#!/usr/bin/env python3
"""Collect effects subsystem items with rust-analyzer's AST symbol parser.

Run from the repository root with:

    direnv exec . python3 docs/plans/effects/review/3-current-effects-system-review/collect_effect_items.py

The script intentionally uses rust-analyzer's `symbols` subcommand instead of
regex extraction. It computes line numbers from rust-analyzer byte ranges and
uses nearby Rust doc comments as the first description pass.
"""

from __future__ import annotations

import html
import argparse
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path


SYMBOL_RE = re.compile(
	r"StructureNode \{ parent: (?P<parent>None|Some\((?P<parent_id>\d+)\)), "
	r'label: "(?P<label>(?:\\.|[^"])*)", '
	r"navigation_range: (?P<nav_start>\d+)\.\.(?P<nav_end>\d+), "
	r"node_range: (?P<node_start>\d+)\.\.(?P<node_end>\d+), "
	r"kind: SymbolKind\((?P<kind>[^)]+)\), "
	r"detail: (?P<detail>None|Some\(\"(?:\\.|[^\"]*)\"\)), "
	r"deprecated: (?P<deprecated>true|false) \}"
)

DETAIL_RE = re.compile(r'detail: Some\("(?P<detail>(?:\\.|[^"])*)"\)')

INCLUDED_KINDS = {
	"Const",
	"Enum",
	"Function",
	"Impl",
	"Macro",
	"Method",
	"Module",
	"Static",
	"Struct",
	"Trait",
	"TypeAlias",
	"Variant",
}

EXAMPLE_PREFIXES = (
	"```",
	"#",
	"use ",
	"let ",
	"assert",
	"type ",
	"struct ",
	"enum ",
	"impl ",
)
EXAMPLE_MARKERS = (
	"```",
	"assert!",
	"assert_eq!",
	"use fp_library",
)


@dataclass
class Symbol:
	index: int
	parent: int | None
	label: str
	kind: str
	detail: str
	nav_start: int
	node_start: int
	line: int
	path: str
	description: str
	description_source: str


def repo_root() -> Path:
	path = Path(__file__).resolve()
	for parent in path.parents:
		if (parent / "justfile").exists():
			return parent
	raise RuntimeError("Could not find repository root")


def module_path_for_file(root: Path, file: Path) -> str:
	rel = file.relative_to(root)
	parts = list(rel.parts)
	if parts[:2] == ["fp-library", "src"]:
		crate = "fp_library"
		module_parts = parts[2:]
	elif parts[:2] == ["fp-macros", "src"]:
		crate = "fp_macros"
		module_parts = parts[2:]
	else:
		crate = rel.parts[0].replace("-", "_")
		module_parts = list(rel.parts[1:])

	if module_parts[-1] == "effects.rs":
		module_parts[-1] = "effects"
	else:
		module_parts[-1] = module_parts[-1][:-3]

	return "::".join([crate, *module_parts])


def target_files(
	root: Path,
	include_tests: bool,
) -> list[Path]:
	files = {
		root / "fp-library/src/brands/effects.rs",
		root / "fp-library/src/types/effects.rs",
		root / "fp-macros/src/effects.rs",
	}
	files.update((root / "fp-library/src/types/effects").rglob("*.rs"))
	files.update((root / "fp-macros/src/effects").rglob("*.rs"))
	result = sorted(files)
	if include_tests:
		return result
	return [
		file
		for file in result
		if file.name != "tests.rs" and not file.name.endswith("_tests.rs")
	]


def line_for_offset(text: str, offset: int) -> int:
	return text.count("\n", 0, offset) + 1


def clean_label(raw: str) -> str:
	return bytes(raw, "utf-8").decode("unicode_escape")


def clean_detail(raw: str) -> str:
	match = DETAIL_RE.match(f"detail: {raw}")
	if not match:
		return ""
	return bytes(match.group("detail"), "utf-8").decode("unicode_escape")


def first_sentence(docs: list[str]) -> str:
	text = " ".join(part for part in docs if part)
	text = re.sub(r"\s+", " ", text).strip()
	if not text:
		return ""
	if text.startswith(EXAMPLE_PREFIXES):
		return ""
	if any(marker in text for marker in EXAMPLE_MARKERS):
		return ""
	for marker in [". ", "? ", "! "]:
		if marker in text:
			return text.split(marker, 1)[0] + marker.strip()
	return text


def doc_blocks(lines: list[str], start_line: int, nav_line: int) -> list[list[str]]:
	window = lines[max(start_line - 1, 0) : max(nav_line - 1, start_line - 1)]
	blocks: list[list[str]] = []
	current: list[str] = []
	for line in window:
		stripped = line.strip()
		if stripped.startswith("///"):
			current.append(stripped[3:].strip())
		elif stripped.startswith("//!"):
			current.append(stripped[3:].strip())
		elif current:
			blocks.append(current)
			current = []
	if current:
		blocks.append(current)
	return blocks


def inline_test_start_line(lines: list[str]) -> int | None:
	for index, line in enumerate(lines, start=1):
		if re.search(r"\bmod\s+tests\b", line):
			return index
	return None


def doc_description(lines: list[str], node_line: int, nav_line: int) -> str:
	# rust-analyzer often starts function nodes at document_* attributes, so
	# scan a bounded window above the identifier and pick the closest prose doc
	# block that is not an example snippet.
	for block in reversed(doc_blocks(lines, max(node_line - 80, 1), nav_line)):
		sentence = first_sentence(block)
		if sentence:
			return sentence
	return ""


def fallback_description(symbol: Symbol, parent: Symbol | None) -> str:
	if symbol.kind == "Impl":
		label = symbol.label
		if " for " in label:
			trait_name, target = label.removeprefix("impl ").split(" for ", 1)
			return f"Implements `{trait_name}` for `{target}`."
		return f"Inherent implementation block for `{label.removeprefix('impl ')}`."
	if symbol.kind == "Module":
		return "Module namespace or submodule declaration in the effects subsystem."
	if symbol.kind == "Variant" and parent is not None:
		return f"Variant of `{parent.label}`."
	if symbol.kind in {"Function", "Method"} and parent is not None:
		return f"{symbol.kind} defined under `{parent.label}`."
	if symbol.detail:
		return f"{symbol.kind} with signature `{symbol.detail}`."
	return f"{symbol.kind} in the effects subsystem."


def collect_symbols_for_file(
	root: Path,
	file: Path,
	include_tests: bool,
) -> list[Symbol]:
	text = file.read_text()
	lines = text.splitlines()
	test_start = inline_test_start_line(lines)
	result = subprocess.run(
		["rust-analyzer", "symbols"],
		input=text,
		text=True,
		capture_output=True,
		check=True,
	)

	base = module_path_for_file(root, file)
	symbols: list[Symbol] = []
	by_id: dict[int, Symbol] = {}
	for raw_line in result.stdout.splitlines():
		match = SYMBOL_RE.match(raw_line)
		if not match:
			continue
		kind = match.group("kind")
		if kind not in INCLUDED_KINDS:
			continue

		index = len(by_id)
		parent_id = match.group("parent_id")
		parent = int(parent_id) if parent_id is not None else None
		label = clean_label(match.group("label"))
		nav_start = int(match.group("nav_start"))
		node_start = int(match.group("node_start"))
		nav_line = line_for_offset(text, nav_start)
		node_line = line_for_offset(text, node_start)
		if not include_tests and test_start is not None and nav_line >= test_start:
			continue
		parent_symbol = by_id.get(parent) if parent is not None else None

		if parent_symbol is None:
			item_path = f"{base}::{label}"
		else:
			item_path = f"{parent_symbol.path}::{label}"

		if not include_tests and ("::tests" in item_path or "::test_" in item_path):
			continue

		symbol = Symbol(
			index=index,
			parent=parent,
			label=label,
			kind=kind,
			detail=clean_detail(match.group("detail")),
			nav_start=nav_start,
			node_start=node_start,
			line=nav_line,
			path=item_path,
			description="",
			description_source="doc",
		)
		symbol.description = doc_description(lines, node_line, nav_line)
		if not symbol.description:
			symbol.description = fallback_description(symbol, parent_symbol)
			symbol.description_source = "fallback"
		by_id[index] = symbol
		symbols.append(symbol)
	return symbols


def markdown_escape(value: str) -> str:
	value = value.replace("\n", " ")
	value = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", value)
	value = value.replace("|", "\\|")
	value = value.replace("`", "\\`")
	return html.escape(value, quote=False)


def compact(
	value: str,
	limit: int,
) -> str:
	value = re.sub(r"\s+", " ", value).strip()
	if len(value) <= limit:
		return value
	return value[: limit - 3].rstrip() + "..."


def main() -> None:
	parser = argparse.ArgumentParser()
	parser.add_argument(
		"--include-tests",
		action="store_true",
		help="Include test-only files and symbols in the inventory.",
	)
	args = parser.parse_args()
	root = repo_root()
	print("# Effects System Item Inventory")
	print()
	print(
		"Generated from `rust-analyzer symbols` over `fp-library/src/brands/effects.rs`, "
		"`fp-library/src/types/effects.rs` and submodules, and "
		"`fp-macros/src/effects.rs` and submodules."
	)
	print()
	print("| File | Line | Kind | Item path | Detail | Description | Description source |")
	print("| --- | ---: | --- | --- | --- | --- | --- |")
	for file in target_files(root, args.include_tests):
		rel = file.relative_to(root)
		for symbol in collect_symbols_for_file(root, file, args.include_tests):
			print(
				"| "
				f"`{markdown_escape(str(rel))}` | "
				f"{symbol.line} | "
				f"{markdown_escape(symbol.kind)} | "
				f"`{markdown_escape(symbol.path)}` | "
				f"{markdown_escape(compact(symbol.detail, 160))} | "
				f"{markdown_escape(compact(symbol.description, 220))} | "
				f"{markdown_escape(symbol.description_source)} |"
			)


if __name__ == "__main__":
	main()
