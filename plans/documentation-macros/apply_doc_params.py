import re
import os

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Pattern to find the Parameters section
    # It captures the whole block of parameters
    pattern = re.compile(
        r'(^|\n)([ \t]*)/// ### Parameters[ \t]*\n'
        r'(?:[ \t]*///[ \t]*\n)?'
        r'((?:[ \t]*///\s*\*\s*`[^`]+`:.*\n)+)',
        re.MULTILINE
    )

    def replacement(match):
        prefix = match.group(1)
        indent = match.group(2)
        params_content = match.group(3)
        
        # Parse doc params
        args = []
        for line in params_content.splitlines():
            # Match * `name`: description
            m = re.search(r'///\s*\*\s*`([^`]+)`:\s*(.*)', line)
            if m:
                name = m.group(1)
                desc = m.group(2).strip()
                
                # Skip 'self' parameter if documented
                if name == "self":
                    continue
                
                # Escape quotes in description
                desc = desc.replace('"', '\\"')
                args.append(f'"{desc}"')
        
        if not args:
            return match.group(0)
            
        args_str = ",\n".join([f'{indent}\t{arg}' for arg in args])
        return f'{prefix}{indent}/// ### Parameters\n{indent}///\n{indent}#[doc_params(\n{args_str}\n{indent})]'

    new_content = pattern.sub(replacement, content)

    if new_content != content:
        if "use fp_macros::doc_params;" not in new_content:
            lines = new_content.splitlines(keepends=True)
            insert_idx = 0
            if lines and lines[0].startswith("#!"):
                insert_idx += 1
            while insert_idx < len(lines):
                line = lines[insert_idx].strip()
                if line.startswith("//!") or line.startswith("#!") or not line:
                    insert_idx += 1
                else:
                    break
            lines.insert(insert_idx, "use fp_macros::doc_params;\n")
            new_content = "".join(lines)
            
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Updated {filepath}")

def main():
    import sys
    if len(sys.argv) > 1:
        for arg in sys.argv[1:]:
            if os.path.isfile(arg):
                process_file(arg)
            else:
                for root, dirs, files in os.walk(arg):
                    if 'target' in dirs:
                        dirs.remove('target')
                    if '.git' in dirs:
                        dirs.remove('.git')
                    for file in files:
                        if file.endswith('.rs'):
                            process_file(os.path.join(root, file))
    else:
        for root, dirs, files in os.walk('.'):
            if 'target' in dirs:
                dirs.remove('target')
            if '.git' in dirs:
                dirs.remove('.git')
            for file in files:
                if file.endswith('.rs'):
                    process_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
