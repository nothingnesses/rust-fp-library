#!/usr/bin/env python3
"""
Migrate #[document_examples(r#"..."#)] attributes to doc comment code blocks.

For each function/method that has:

    #[document_examples(r#"some_fn();
    assert_eq!(result, expected);"#)]
    fn foo() { ... }

this script produces:

    #[document_examples]
    ///
    /// ```
    /// some_fn();
    /// assert_eq!(result, expected);
    /// ```
    fn foo() { ... }

This is the reverse of migrate_examples.py.
"""

import re
import sys
from pathlib import Path


# Match #[document_examples(r#"..."#)] including multiline raw strings.
# Captures: indent, hashes, code content.
ATTR_PATTERN = re.compile(
    r'^(?P<indent>[ \t]*)#\[document_examples\(\s*r(?P<hashes>#+)"(?P<code>.*?)"(?P=hashes)\s*\)\]',
    re.DOTALL | re.MULTILINE,
)


def replace_attr(m: re.Match) -> str:
    indent = m.group("indent")
    code = m.group("code")

    lines = [f"{indent}#[document_examples]"]
    lines.append(f"{indent}///")
    lines.append(f"{indent}/// ```")

    for code_line in code.split("\n"):
        if code_line == "":
            lines.append(f"{indent}///")
        else:
            lines.append(f"{indent}/// {code_line}")

    lines.append(f"{indent}/// ```")

    return "\n".join(lines)


def process_file(path: Path, dry_run: bool = False) -> int:
    text = path.read_text(encoding="utf-8")

    matches = list(ATTR_PATTERN.finditer(text))
    if not matches:
        return 0

    new_text = ATTR_PATTERN.sub(replace_attr, text)
    count = len(matches)

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
        print(
            "Usage: reverse_migrate_examples.py [--dry-run] <file_or_dir>...",
            file=sys.stderr,
        )
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
