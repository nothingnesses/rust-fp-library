import json
import os
import re
from collections import defaultdict

def remove_clippy_empty_docs_carefully(jsonl_path):
    targets = defaultdict(set)
    # Regex for empty doc comment line: whitespace, then /// or //! followed by nothing but whitespace
    empty_doc_re = re.compile(r"^\s*(///|//!)\s*$")
    
    with open(jsonl_path, "r") as f:
        for line in f:
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                continue
                
            if data.get("reason") != "compiler-message":
                continue
                
            message = data.get("message", {})
            code = message.get("code")
            if not code or code.get("code") != "clippy::empty_docs":
                continue
                
            for span in message.get("spans", []):
                if span.get("is_primary"):
                    file_name = span.get("file_name")
                    line_start = span.get("line_start")
                    line_end = span.get("line_end")
                    if file_name and line_start and line_end:
                        # Add the span to the targets
                        targets[file_name].add((line_start, line_end))
    
    updated_files = 0
    removed_lines_total = 0
    
    for file_name, spans in targets.items():
        if not os.path.exists(file_name):
            print(f"Warning: File {file_name} not found.")
            continue
            
        with open(file_name, "r", encoding="utf-8") as f:
            lines = f.readlines()
            
        new_lines = []
        file_changed = False
        
        # Create a set of all lines that are part of ANY flagged span
        # AND match the empty doc regex
        lines_to_remove = set()
        for start, end in spans:
            for i in range(start, end + 1):
                # i is 1-indexed
                if i <= len(lines):
                    line_content = lines[i-1]
                    if empty_doc_re.match(line_content):
                        lines_to_remove.add(i)
        
        for i, line in enumerate(lines):
            if (i + 1) in lines_to_remove:
                file_changed = True
                removed_lines_total += 1
                continue
            new_lines.append(line)
            
        if file_changed:
            with open(file_name, "w", encoding="utf-8") as f:
                f.writelines(new_lines)
            print(f"Updated {file_name}, removed {len(lines_to_remove)} lines.")
            updated_files += 1
        
    print(f"Finished. Updated {updated_files} files, removed {removed_lines_total} lines total.")

if __name__ == "__main__":
    import sys
    jsonl_path = sys.argv[1] if len(sys.argv) > 1 else "clippy_output.jsonl"
    remove_clippy_empty_docs_carefully(jsonl_path)
