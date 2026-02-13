import os
import re
import sys

def process_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        print(f"Skipping binary file: {filepath}")
        return

    # Regex to find #[hm_signature(Something)] or #[hm_signature()] 
    # and replace with #[hm_signature].
    # Matches:
    # #[hm_signature(Trait)]
    # #[hm_signature(  Trait  )]
    # #[hm_signature()]
    pattern = re.compile(r'#\[hm_signature\([^)]*\)\]')
    
    new_content = pattern.sub('#[hm_signature]', content)

    if new_content != content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"Updated {filepath}")

def main():
    paths = sys.argv[1:] if len(sys.argv) > 1 else ['.']
    
    for path in paths:
        if os.path.isfile(path):
            process_file(path)
        else:
            for root, dirs, files in os.walk(path):
                if 'target' in dirs:
                    dirs.remove('target')
                if '.git' in dirs:
                    dirs.remove('.git')
                
                for file in files:
                    if file.endswith('.rs'):
                        process_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
