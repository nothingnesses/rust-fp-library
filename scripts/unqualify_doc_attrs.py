#!/usr/bin/env python3
"""Replace fp_macros:: qualified doc attributes with unqualified versions
and ensure fp_macros::* imports are present.

- #[fp_macros::document_signature] -> #[document_signature] (etc.)
- #[fp_macros::document_module] is kept qualified (outer proc macro attribute)
- fp_macros::{document_X, ...} imports -> fp_macros::*
- Adds use fp_macros::*; inside mod inner blocks that need it
"""
import re
import glob


def process_file(filepath):
	with open(filepath) as f:
		content = f.read()

	if 'fp_macros::' not in content:
		return False

	original = content

	# Step 1: Replace #[fp_macros::document_X] with #[document_X]
	# Keep fp_macros::document_module since it's the outer proc macro attribute
	content = re.sub(r'fp_macros::(document_(?!module\b)\w+)', r'\1', content)

	# Step 2: Replace specific fp_macros::{...} imports with fp_macros::*
	content = re.sub(r'fp_macros::\{[^}]*\}', 'fp_macros::*', content, flags=re.DOTALL)

	# Step 3: Add use fp_macros::*; if file uses doc attrs but has no fp_macros import
	needs_import = bool(re.search(
		r'#\[document_(?:signature|type_parameters|parameters|returns|examples)',
		content
	)) and 'fp_macros::*' not in content

	if needs_import:
		if '\tuse super::*;' in content:
			content = content.replace(
				'\tuse super::*;',
				'\tuse super::*;\n\tuse fp_macros::*;',
				1
			)
		elif 'mod inner {' in content:
			content = re.sub(
				r'(mod inner \{)\n',
				r'\1\n\tuse fp_macros::*;\n',
				content,
				count=1
			)

	if content != original:
		with open(filepath, 'w') as f:
			f.write(content)
		return True
	return False


count = 0
for path in sorted(glob.glob('fp-library/src/**/*.rs', recursive=True)):
	if process_file(path):
		count += 1
		print(f'  {path}')

print(f'\n{count} files updated')
