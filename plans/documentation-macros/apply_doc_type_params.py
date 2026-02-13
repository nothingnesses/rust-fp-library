import re
import os

def extract_generics(sig_text):
    # sig_text is something like "<'a, M: Applicative, A: 'a + Clone>"
    content = sig_text.strip()
    if not content.startswith('<') or not content.endswith('>'):
        return []
    content = content[1:-1]
    params = []
    current = ""
    depth = 0
    for char in content:
        if char == '<':
            depth += 1
            current += char
        elif char == '>':
            depth -= 1
            current += char
        elif char == ',' and depth == 0:
            params.append(current.strip())
            current = ""
        else:
            current += char
    if current.strip():
        params.append(current.strip())
    
    names = []
    for p in params:
        p = re.sub(r'^const\s+', '', p)
        name = re.split(r'[:\s=]', p)[0]
        if name:
            names.append(name)
    return names

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Pattern to find the Type Parameters section
    pattern = re.compile(
        r'(^|\n)([ \t]*)/// ### Type Parameters[ \t]*\n'
        r'(?:[ \t]*///[ \t]*\n)?'
        r'((?:[ \t]*///\s*\*\s*`\w+`:.*\n)+)',
        re.MULTILINE
    )

    def replacement(match):
        prefix = match.group(1)
        indent = match.group(2)
        params_content = match.group(3)
        
        # Parse existing doc params
        doc_params = {}
        ordered_doc_params = []
        for line in params_content.splitlines():
            m = re.search(r'///\s*\*\s*`(\w+)`:\s*(.*)', line)
            if m:
                name = m.group(1)
                desc = m.group(2).strip()
                doc_params[name] = desc
                ordered_doc_params.append((name, desc))

        # Look ahead for the function signature
        start_idx = match.end()
        lookahead = content[start_idx:start_idx+5000]
        
        # Find the next non-comment, non-attribute line
        lines = lookahead.splitlines()
        next_item_line = ""
        for line in lines:
            stripped = line.strip()
            if not stripped or stripped.startswith("///") or stripped.startswith("#[") or stripped.startswith("//"):
                continue
            next_item_line = stripped
            break
        
        # Check if the next item is a function
        # Matches: fn name, pub fn name, pub(crate) fn name, etc.
        if not re.search(r'\bfn\b', next_item_line):
             # Not a function (e.g., a struct, enum, or impl)
             return match.group(0)

        # Re-parse lookahead without comments to extract generics accurately
        lookahead_no_comments = re.sub(r'//.*', '', lookahead)
        lookahead_no_comments = re.sub(r'/\*.*?\*/', '', lookahead_no_comments, flags=re.DOTALL)
        
        fn_match = re.search(r'\bfn\s+(\w+)', lookahead_no_comments)
        if not fn_match:
            return match.group(0)
            
        # Find generics part <...>
        after_fn_name = lookahead_no_comments[fn_match.end():]
        generics_match = re.search(r'^\s*<', after_fn_name)
        
        generic_names = []
        if generics_match:
            depth = 0
            generics_text = ""
            actual_start = generics_match.start()
            for char in after_fn_name[actual_start:]:
                if char == '<':
                    depth += 1
                elif char == '>':
                    depth -= 1
                generics_text += char
                if depth == 0:
                    break
            generic_names = extract_generics(generics_text)
        
        if not generic_names:
            return match.group(0)

        # Map documentation to generic names
        final_args = []
        doc_ptr = 0
        for g_name in generic_names:
            clean_g = g_name.lstrip("'")
            matched = False
            if doc_ptr < len(ordered_doc_params):
                d_name, d_desc = ordered_doc_params[doc_ptr]
                if d_name == g_name or d_name == clean_g:
                    final_args.append(f'"{d_desc}"')
                    doc_ptr += 1
                    matched = True
            
            if not matched:
                if g_name in doc_params:
                    final_args.append(f'"{doc_params[g_name]}"')
                elif clean_g in doc_params:
                    final_args.append(f'"{doc_params[clean_g]}"')
                else:
                    if not g_name.startswith("'") and doc_ptr < len(ordered_doc_params):
                        d_name, d_desc = ordered_doc_params[doc_ptr]
                        final_args.append(f'("{d_name}", "{d_desc}")')
                        doc_ptr += 1
                    else:
                        final_args.append('"Undocumented"')
        
        args_str = ",\n".join([f'{indent}\t{arg}' for arg in final_args])
        return f'{prefix}{indent}/// ### Type Parameters\n{indent}///\n{indent}#[doc_type_params(\n{args_str}\n{indent})]'

    new_content = pattern.sub(replacement, content)

    if new_content != content:
        if "use fp_macros::doc_type_params;" not in new_content:
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
            lines.insert(insert_idx, "use fp_macros::doc_type_params;\n")
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
