#!/usr/bin/env python3
"""
Migrate /// ### Examples doc comment sections to #[document_examples(...)] attributes.

For each function/method that has:

    /// ### Examples
    ///
    /// ```
    /// some_fn();
    /// assert_eq!(result, expected);
    /// ```
    fn foo() { ... }

this script produces:

    #[document_examples(r#"some_fn();
    assert_eq!(result, expected);"#)]
    fn foo() { ... }

Only sections whose extracted code contains at least one assertion macro invocation
are migrated; the rest are left untouched so that the document_examples attribute
macro does not reject them.
"""

import re
import sys
from pathlib import Path

ASSERTION_MACROS = [
    "assert!",
    "assert_eq!",
    "assert_ne!",
    "debug_assert!",
    "debug_assert_eq!",
    "debug_assert_ne!",
    "assert_matches!",
]

# Code fence opening lines that we recognise as Rust examples.
# Language tags other than these (e.g. "text", "purescript") are left alone.
RUST_FENCE_TAGS = {"", "rust", "rust,no_run", "rust, no_run"}

FN_STARTERS = (
    "fn ",
    "pub fn ",
    "pub(crate) fn ",
    "pub(super) fn ",
    "pub(self) fn ",
    "async fn ",
    "pub async fn ",
    "unsafe fn ",
    "pub unsafe fn ",
    "extern fn ",
    "pub extern fn ",
)


def contains_assertion(code: str) -> bool:
    return any(mac in code for mac in ASSERTION_MACROS)


def make_raw_string(code: str) -> str:
    """Wrap *code* in a Rust raw-string literal with the minimum # count."""
    hashes = 1
    while '"' + "#" * hashes in code:
        hashes += 1
    h = "#" * hashes
    return f'r{h}"{code}"{h}'


def leading_whitespace(line: str) -> str:
    return line[: len(line) - len(line.lstrip("\t "))]


def process_lines(lines: list[str]) -> tuple[list[str], int]:
    """
    Process a list of source lines and return (new_lines, replacements_made).
    """
    result: list[str] = []
    replacements = 0
    i = 0

    while i < len(lines):
        raw = lines[i]
        content = raw.rstrip("\n")
        stripped = content.lstrip("\t ")

        if stripped != "/// ### Examples":
            result.append(raw)
            i += 1
            continue

        # ── Found "/// ### Examples" ──────────────────────────────────────────
        indent = leading_whitespace(content)
        examples_line_idx = i
        j = i + 1

        # Skip any blank `///` lines between "### Examples" and the opening fence.
        while j < len(lines) and lines[j].rstrip("\n").lstrip("\t ") == "///":
            j += 1

        # Expect an opening code fence.
        if j >= len(lines):
            result.append(raw)
            i += 1
            continue

        fence_content = lines[j].rstrip("\n").lstrip("\t ")
        if not fence_content.startswith("/// ```"):
            # Not a code fence – leave as-is.
            result.append(raw)
            i += 1
            continue

        lang_tag = fence_content[len("/// ```"):].strip()
        if lang_tag not in RUST_FENCE_TAGS:
            # Non-Rust fence (e.g. "text", "purescript") – leave as-is.
            result.append(raw)
            i += 1
            continue

        opening_fence_idx = j
        j += 1

        # Collect code lines until the closing fence.
        code_lines: list[str] = []
        found_closing = False
        while j < len(lines):
            code_raw = lines[j].rstrip("\n")
            code_stripped = code_raw.lstrip("\t ")

            if code_stripped == "/// ```":
                # Closing fence.
                found_closing = True
                j += 1
                break
            elif code_stripped.startswith("/// "):
                # Normal doc-comment code line.
                code_lines.append(code_raw[len(indent) + len("/// "):])
            elif code_stripped == "///":
                # Empty doc-comment line inside the code block.
                code_lines.append("")
            else:
                # Something unexpected – abort this replacement.
                break
            j += 1

        if not found_closing:
            result.append(raw)
            i += 1
            continue

        closing_fence_idx = j - 1  # index of the "/// ```" line we just consumed

        code = "\n".join(code_lines)

        # ── Perform the replacement ───────────────────────────────────────────
        # Lines [examples_line_idx .. j-1] (inclusive) are consumed.
        # We emit nothing for them and then, just before the fn definition,
        # we emit the #[document_examples(...)] attribute.

        raw_str = make_raw_string(code)
        attr_line = f"{indent}#[document_examples({raw_str})]\n"

        # Skip from examples_line_idx to j-1 (already consumed above).
        # Now re-emit any non-fn lines between j and the fn definition,
        # then emit the attribute followed by the fn definition.
        while j < len(lines):
            next_content = lines[j].rstrip("\n")
            next_stripped = next_content.lstrip("\t ")
            if any(next_stripped.startswith(s) for s in FN_STARTERS):
                result.append(attr_line)
                break
            else:
                result.append(lines[j])
                j += 1

        i = j
        replacements += 1

    return result, replacements


def add_import(text: str) -> str:
    """
    Ensure `document_examples` is listed inside the `fp_macros::{ ... }` block.
    If it's already there, do nothing. If the block exists, append it.
    """
    import re
    # Find the fp_macros::{ ... }, block.
    block_pat = re.compile(r'(fp_macros::\{[^}]*?\})', re.DOTALL)
    m = block_pat.search(text)
    if not m:
        return text
    block = m.group(1)
    # Already imported inside the block?
    if "document_examples" in block:
        return text
    # Insert document_examples before the closing }.
    new_block = block[:-1] + "\n\t\t\tdocument_examples,\n\t\t}"
    return text[:m.start()] + new_block + text[m.end():]


def process_file(path: Path, dry_run: bool = False) -> int:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)

    new_lines, count = process_lines(lines)

    if count == 0:
        return 0

    new_text = add_import("".join(new_lines))
    if dry_run:
        print(f"  [dry-run] would replace {count} section(s) in {path}")
    else:
        path.write_text(new_text, encoding="utf-8")
        print(f"  replaced {count} section(s) in {path}")

    return count


def main(argv: list[str]) -> int:
    dry_run = "--dry-run" in argv
    targets = [a for a in argv if not a.startswith("--")]

    if not targets:
        print("Usage: migrate_examples.py [--dry-run] <file_or_dir>...", file=sys.stderr)
        return 1

    total = 0
    for target in targets:
        p = Path(target)
        if p.is_file():
            total += process_file(p, dry_run)
        elif p.is_dir():
            for rs in sorted(p.rglob("*.rs")):
                total += process_file(rs, dry_run)
        else:
            print(f"warning: {target!r} does not exist", file=sys.stderr)

    print(f"\nTotal replacements: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
