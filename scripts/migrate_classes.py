#!/usr/bin/env python3
"""
Migrate classes/ modules to use #[fp_macros::document_module] mod inner { ... } pattern.

Transformations:
1. Wrap module contents in: #[fp_macros::document_module] mod inner { use super::*; ... } pub use inner::*;
   - `pub mod` declarations and `pub use` re-exports stay OUTSIDE mod inner
2. Convert `/// ### Returns\n///\n/// Description.` to `#[document_returns("Description.")]`
3. Convert `/// ### Examples` to `#[document_examples]`
4. All doc attrs inside mod inner use fully-qualified paths (fp_macros::X)
5. Remove fp_macros doc attr imports from the outer level (they're now qualified)
"""

import re
import sys
from pathlib import Path

ALREADY_MIGRATED_MARKER = "#[fp_macros::document_module]"

# All documentation attributes — will be fully qualified inside mod inner
ALL_DOC_ATTRS = [
    "document_signature",
    "document_type_parameters",
    "document_parameters",
    "document_returns",
    "document_examples",
]


def process_returns_sections(text: str) -> str:
    """Convert /// ### Returns sections to #[document_returns("...")] attributes."""
    lines = text.split("\n")
    result = []
    i = 0

    while i < len(lines):
        stripped = lines[i].lstrip("\t ")

        if stripped == "/// ### Returns":
            indent = lines[i][: len(lines[i]) - len(lines[i].lstrip("\t "))]
            j = i + 1

            # Skip blank /// lines
            while j < len(lines) and lines[j].lstrip("\t ") == "///":
                j += 1

            # Collect description lines until another ### or non-doc line
            desc_lines = []
            while j < len(lines):
                s = lines[j].lstrip("\t ")
                if s.startswith("/// ### ") or s.startswith("/// ```") or not s.startswith("///"):
                    break
                if s == "///":
                    desc_lines.append("")
                elif s.startswith("/// "):
                    desc_lines.append(s[4:])
                else:
                    break
                j += 1

            desc = " ".join(line for line in desc_lines if line).strip()

            if desc:
                desc_escaped = desc.replace("\\", "\\\\").replace('"', '\\"')
                result.append(f'{indent}#[document_returns("{desc_escaped}")]')
                # Skip blank /// line after Returns section
                if j < len(lines) and lines[j].lstrip("\t ") == "///":
                    j += 1
                i = j
                continue

        result.append(lines[i])
        i += 1

    return "\n".join(result)


def process_examples_sections(text: str) -> str:
    """Convert /// ### Examples to #[document_examples]."""
    lines = text.split("\n")
    result = []

    for line in lines:
        stripped = line.lstrip("\t ")
        if stripped == "/// ### Examples":
            indent = line[: len(line) - len(line.lstrip("\t "))]
            result.append(f"{indent}#[document_examples]")
        else:
            result.append(line)

    return "\n".join(result)


def qualify_all_doc_attrs(text: str) -> str:
    """Replace bare #[document_*] with #[fp_macros::document_*] for all doc attrs."""
    for attr in ALL_DOC_ATTRS:
        text = re.sub(
            rf'#\[(?!fp_macros::){attr}\b',
            f'#[fp_macros::{attr}',
            text,
        )
    return text


def remove_doc_attr_imports(text: str) -> str:
    """Remove doc attr imports from fp_macros use blocks since we use qualified paths."""
    for attr in ALL_DOC_ATTRS:
        text = re.sub(rf'\n\s*{attr},', '', text)

    # Clean up empty fp_macros blocks
    # Pattern: fp_macros::{\n\t\t} or fp_macros::{  }
    text = re.sub(r'\n\s*fp_macros::\{\s*\},?', '', text)

    # Clean up resulting empty use blocks: use {\n};\n or use { };\n
    text = re.sub(r'use\s*\{\s*\}\s*;\n?', '', text)

    # Clean up lines that are just "use {" followed by "};" with nothing in between
    text = re.sub(r'use\s+\{\n\s*\};\n?', '', text)

    return text


def is_mod_or_reexport_line(line: str) -> bool:
    """Check if a line is a pub mod or pub use re-export that should stay outside mod inner."""
    stripped = line.strip()
    if re.match(r'^pub mod \w+;', stripped):
        return True
    if re.match(r'^pub use\s', stripped):
        return True
    return False


def wrap_in_inner_module(text: str) -> str:
    """Wrap module body in #[fp_macros::document_module] mod inner { ... } pub use inner::*;"""
    lines = text.split("\n")

    # Phase 1: Find module doc comment (//! lines) at the top
    module_doc_end = 0
    for i, line in enumerate(lines):
        if line.lstrip("\t ").startswith("//!"):
            module_doc_end = i + 1
        elif line.strip() == "":
            if module_doc_end > 0:
                module_doc_end = i + 1
                if i + 1 < len(lines) and not lines[i + 1].lstrip("\t ").startswith("//!"):
                    break
        else:
            break

    # Phase 2: Find imports section — use/pub use blocks and their continuations
    imports_end = module_doc_end
    brace_depth = 0
    for i in range(module_doc_end, len(lines)):
        stripped = lines[i].strip()

        if stripped == "":
            imports_end = i + 1
            continue

        # Track brace depth for multi-line use blocks
        brace_depth += stripped.count("{") - stripped.count("}")

        # Inside a multi-line use block
        if brace_depth > 0:
            imports_end = i + 1
            continue

        # Just closed a multi-line use block (e.g. "};")
        if brace_depth == 0 and (stripped == "};" or stripped.endswith("};")):
            imports_end = i + 1
            continue

        if stripped.startswith("use ") or stripped.startswith("pub use "):
            imports_end = i + 1
            continue

        # Also catch pub mod declarations at import level
        if re.match(r'^pub mod \w+;', stripped):
            imports_end = i + 1
            continue

        break

    module_doc = "\n".join(lines[:module_doc_end])
    imports = "\n".join(lines[module_doc_end:imports_end])
    rest_lines = lines[imports_end:]

    # Phase 3: Separate pub mod/pub use re-exports from body
    pre_inner = []
    body_lines = []

    for line in rest_lines:
        if is_mod_or_reexport_line(line):
            pre_inner.append(line)
        else:
            body_lines.append(line)

    body = "\n".join(body_lines).strip()

    # Build the new file
    parts = []
    if module_doc.strip():
        parts.append(module_doc.rstrip())

    parts.append("")

    if imports.strip():
        parts.append(imports.rstrip())

    parts.append("")

    # pub mod / pub use re-exports outside mod inner
    if pre_inner:
        for line in pre_inner:
            parts.append(line)
        parts.append("")

    parts.append("#[fp_macros::document_module(no_validation)]")
    parts.append("mod inner {")
    parts.append("\tuse super::*;")
    parts.append("")

    # Indent the body by one tab
    for line in body.split("\n"):
        if line.strip() == "":
            parts.append("")
        else:
            parts.append("\t" + line)

    parts.append("}")
    parts.append("")
    parts.append("pub use inner::*;")
    parts.append("")

    return "\n".join(parts)


def process_file(path: Path, dry_run: bool = False) -> bool:
    text = path.read_text(encoding="utf-8")

    if ALREADY_MIGRATED_MARKER in text:
        return False

    has_trait = "pub trait " in text
    has_fn = "pub fn " in text
    has_impl = "impl " in text
    if not (has_trait or has_fn or has_impl):
        if dry_run:
            print(f"  [skip] {path.name} — no traits/fns/impls")
        return False

    # Step 1: Convert ### Returns to #[document_returns]
    text = process_returns_sections(text)

    # Step 2: Convert ### Examples to #[document_examples]
    text = process_examples_sections(text)

    # Step 3: Qualify all doc attributes with fp_macros::
    text = qualify_all_doc_attrs(text)

    # Step 4: Remove doc attr imports (now using qualified paths)
    text = remove_doc_attr_imports(text)

    # Step 5: Wrap in mod inner
    text = wrap_in_inner_module(text)

    if dry_run:
        print(f"  [dry-run] would migrate {path.name}")
        return True

    path.write_text(text, encoding="utf-8")
    print(f"  migrated {path.name}")
    return True


def main(argv: list[str]) -> int:
    dry_run = "--dry-run" in argv
    targets = [a for a in argv if not a.startswith("--")]

    if not targets:
        print("Usage: migrate_classes.py [--dry-run] <file_or_dir>...", file=sys.stderr)
        return 1

    total = 0
    for target in targets:
        p = Path(target)
        if p.is_file():
            if process_file(p, dry_run):
                total += 1
        elif p.is_dir():
            for rs in sorted(p.rglob("*.rs")):
                if process_file(rs, dry_run):
                    total += 1
        else:
            print(f"warning: {target!r} does not exist", file=sys.stderr)

    print(f"\nTotal files migrated: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
