import re
import os

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    pattern = re.compile(
        r'(^|\n)([ \t]*)/// ### Type Signature[ \t]*\n'
        r'[ \t]*///[ \t]*\n'
        r'[ \t]*///\s*`([^`\n]+)`',
        re.MULTILINE
    )

    def replacement(match):
        prefix = match.group(1)
        indent = match.group(2)
        signature_content = match.group(3)
        
        trait_match = re.search(r'.*?\.\s*(?:[(]\s*)?(\w+).*?=>', signature_content)
        if trait_match:
            trait = trait_match.group(1)
            attr = f'({trait})'
        else:
            attr = ''

        start_idx = match.end()
        lookahead = content[start_idx:start_idx+5000]
        
        lookahead = re.sub(r'//.*', '', lookahead)
        lookahead = re.sub(r'/\*.*?\*/', '', lookahead, flags=re.DOTALL)
        
        fn_match = re.search(r'\bfn\b', lookahead)
        if not fn_match:
            return match.group(0) 
            
        after_fn = lookahead[fn_match.end():]
        
        level = 0
        found_body = False
        found_decl = False
        
        for char in after_fn:
            if char == '{':
                if level == 0:
                    found_body = True
                    break
                level += 1
            elif char == '}':
                level -= 1
            elif char == '(':
                level += 1
            elif char == ')':
                level -= 1
            elif char == '[':
                level += 1
            elif char == ']':
                level -= 1
            elif char == ';':
                if level == 0:
                    found_decl = True
                    break
        
        return f'{prefix}{indent}/// ### Type Signature\n{indent}///\n{indent}#[hm_signature{attr}]'

    new_content = pattern.sub(replacement, content)

    if new_content != content:
        if "use fp_macros::hm_signature;" not in new_content:
            lines = new_content.splitlines(keepends=True)
            insert_idx = 0
            if lines and lines[0].startswith("#!"):
                insert_idx += 1
            while insert_idx < len(lines):
                line = lines[insert_idx].strip()
                if line.startswith("//!"):
                    insert_idx += 1
                elif line.startswith("#!"):
                    insert_idx += 1
                elif not line:
                    insert_idx += 1
                else:
                    break
            
            lines.insert(insert_idx, "use fp_macros::hm_signature;\n")
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
