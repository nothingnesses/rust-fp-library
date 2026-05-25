//! Count and inspect `#[document_examples]` doctest blocks.
//!
//! Usage:
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] --list [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] <index> [--line-numbers] [--json]

use std::{
	env,
	fs,
	path::{
		Path,
		PathBuf,
	},
	process::{
		Command,
		ExitCode,
	},
};

const DOCUMENT_EXAMPLES_ATTR: &str = "#[document_examples]";
const QUALIFIED_DOCUMENT_EXAMPLES_ATTR: &str = "#[fp_macros::document_examples]";
const DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX: &str = "#[document_examples(";
const QUALIFIED_DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX: &str = "#[fp_macros::document_examples(";
const SKIP_CALL_CHECK: &str = "skip_call_check";
const DOC_FENCE_PREFIX: &str = "/// ```";

#[derive(Debug)]
enum Mode {
	Count,
	List,
	Example(usize),
}

#[derive(Clone, Copy, Debug)]
enum AttributeMode {
	SkipCallCheck,
	Plain,
	Both,
}

impl AttributeMode {
	fn parse(value: &str) -> Result<Self, String> {
		match value {
			"1" => Ok(Self::SkipCallCheck),
			"2" => Ok(Self::Plain),
			"3" => Ok(Self::Both),
			_ => Err(format!("--mode must be 1, 2, or 3: {value}")),
		}
	}

	fn includes(
		self,
		kind: AttributeKind,
	) -> bool {
		matches!(
			(self, kind),
			(Self::SkipCallCheck, AttributeKind::SkipCallCheck)
				| (Self::Plain, AttributeKind::Plain)
				| (Self::Both, _)
		)
	}

	fn description(self) -> &'static str {
		match self {
			Self::SkipCallCheck => "document examples with skip_call_check",
			Self::Plain => "document examples without skip_call_check",
			Self::Both => "document examples",
		}
	}

	fn number(self) -> u8 {
		match self {
			Self::SkipCallCheck => 1,
			Self::Plain => 2,
			Self::Both => 3,
		}
	}
}

#[derive(Debug)]
struct Config {
	path: PathBuf,
	attribute_mode: AttributeMode,
	json: bool,
	line_numbers: bool,
	mode: Mode,
}

#[derive(Clone, Copy, Debug)]
enum AttributeKind {
	SkipCallCheck,
	Plain,
}

impl AttributeKind {
	fn as_json_value(self) -> &'static str {
		match self {
			Self::SkipCallCheck => "skip_call_check",
			Self::Plain => "plain",
		}
	}

	fn as_attribute(self) -> &'static str {
		match self {
			Self::SkipCallCheck => "#[document_examples(skip_call_check, reason = \"...\")]",
			Self::Plain => DOCUMENT_EXAMPLES_ATTR,
		}
	}
}

#[derive(Debug)]
struct Entry {
	index: usize,
	path: PathBuf,
	line: usize,
	kind: AttributeKind,
}

#[derive(Debug)]
struct Example {
	index: usize,
	path: PathBuf,
	line: usize,
	kind: AttributeKind,
	example_start_line: usize,
	example_end_line: usize,
	lines: Vec<String>,
}

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("error: {error}");
			eprintln!();
			eprintln!("{}", usage());
			ExitCode::from(2)
		}
	}
}

fn run() -> Result<(), String> {
	let config = parse_args(env::args().skip(1))?;

	if !config.path.is_dir() {
		return Err(format!("--path must name an existing directory: {}", config.path.display()));
	}

	let files = rust_files_from_git(&config.path)?;
	let entries = collect_entries(&config.path, &files, config.attribute_mode)?;

	match config.mode {
		Mode::Count => print_count(entries.len(), config.json, config.attribute_mode),
		Mode::List => print_list(&entries, config.json, config.attribute_mode),
		Mode::Example(index) => {
			let entry = entries.get(index).ok_or_else(|| {
				format!("index {index} is out of range; found {} document examples", entries.len())
			})?;
			let example = extract_example(&config.path, entry)?;
			print_example(&example, entries.len(), config.json, config.line_numbers);
		}
	}

	Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
	let mut path = PathBuf::from(".");
	let mut attribute_mode = AttributeMode::SkipCallCheck;
	let mut json = false;
	let mut line_numbers = false;
	let mut list = false;
	let mut index = None;
	let mut args = args.peekable();

	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--" => {}
			"--help" | "-h" => {
				println!("{}", usage());
				std::process::exit(0);
			}
			"--json" => json = true,
			"--line-numbers" => line_numbers = true,
			"--list" => list = true,
			"--mode" => {
				let value = args.next().ok_or_else(|| "--mode requires 1, 2, or 3".to_string())?;
				attribute_mode = AttributeMode::parse(&value)?;
			}
			"--path" => {
				let value = args
					.next()
					.ok_or_else(|| "--path requires a directory argument".to_string())?;
				path = PathBuf::from(value);
			}
			_ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
			_ => {
				if index.is_some() {
					return Err(format!("unexpected extra argument: {arg}"));
				}
				index = Some(
					arg.parse::<usize>()
						.map_err(|_| format!("index must be an unsigned integer: {arg}"))?,
				);
			}
		}
	}

	let mode = match (list, index) {
		(true, Some(_)) => return Err("--list cannot be combined with an index".to_string()),
		(true, None) => Mode::List,
		(false, Some(index)) => Mode::Example(index),
		(false, None) => Mode::Count,
	};

	if line_numbers && !matches!(mode, Mode::Example(_)) {
		return Err("--line-numbers can only be used with an index".to_string());
	}

	Ok(Config {
		path,
		attribute_mode,
		json,
		line_numbers,
		mode,
	})
}

fn usage() -> &'static str {
	"usage:
  rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] [--json]
  rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] --list [--json]
  rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] <index> [--line-numbers] [--json]

modes:
  1  #[document_examples(skip_call_check, reason = \"...\")] only (default)
  2  #[document_examples] without skip_call_check only
  3  both"
}

fn rust_files_from_git(root: &Path) -> Result<Vec<PathBuf>, String> {
	let output = Command::new("git")
		.current_dir(root)
		.args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
		.output()
		.map_err(|error| format!("failed to run git ls-files: {error}"))?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		return Err(format!("git ls-files failed: {}", stderr.trim()));
	}

	let mut files: Vec<PathBuf> = output
		.stdout
		.split(|byte| *byte == 0)
		.filter(|path| !path.is_empty())
		.map(|path| PathBuf::from(String::from_utf8_lossy(path).into_owned()))
		.filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
		.collect();

	files.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));

	Ok(files)
}

fn collect_entries(
	root: &Path,
	files: &[PathBuf],
	attribute_mode: AttributeMode,
) -> Result<Vec<Entry>, String> {
	let mut entries = Vec::new();

	for path in files {
		let contents = read_to_string(root, path)?;
		let lines: Vec<_> = contents.lines().collect();
		let mut line_index = 0;

		while line_index < lines.len() {
			let Some((kind, end_line_index)) = document_examples_attr_kind(&lines, line_index)
			else {
				line_index += 1;
				continue;
			};
			if attribute_mode.includes(kind) {
				entries.push(Entry {
					index: entries.len(),
					path: path.clone(),
					line: line_index + 1,
					kind,
				});
			}
			line_index = end_line_index + 1;
		}
	}

	Ok(entries)
}

fn extract_example(
	root: &Path,
	entry: &Entry,
) -> Result<Example, String> {
	let contents = read_to_string(root, &entry.path)?;
	let mut lines = Vec::new();
	let mut example_start_line = None;

	for (line_index, line) in contents.lines().enumerate().skip(entry.line) {
		let line_number = line_index + 1;

		if example_start_line.is_none() {
			if is_doc_fence(line) {
				example_start_line = Some(line_number);
				lines.push(line.to_string());
			}
			continue;
		}

		lines.push(line.to_string());

		if is_doc_fence(line) {
			return Ok(Example {
				index: entry.index,
				path: entry.path.clone(),
				line: entry.line,
				kind: entry.kind,
				example_start_line: example_start_line.expect("example start line is set"),
				example_end_line: line_number,
				lines,
			});
		}
	}

	if let Some(example_start_line) = example_start_line {
		Err(format!(
			"{}:{} has an unterminated doctest block starting at line {}",
			entry.path.display(),
			entry.line,
			example_start_line
		))
	} else {
		Err(format!(
			"{}:{} has no following doc comment code fence",
			entry.path.display(),
			entry.line
		))
	}
}

fn read_to_string(
	root: &Path,
	path: &Path,
) -> Result<String, String> {
	fs::read_to_string(root.join(path))
		.map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn document_examples_attr_kind(
	lines: &[&str],
	start_line_index: usize,
) -> Option<(AttributeKind, usize)> {
	let line = lines.get(start_line_index)?.trim();
	if line == DOCUMENT_EXAMPLES_ATTR || line == QUALIFIED_DOCUMENT_EXAMPLES_ATTR {
		return Some((AttributeKind::Plain, start_line_index));
	}

	if line.starts_with(DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX)
		|| line.starts_with(QUALIFIED_DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX)
	{
		let mut attr = String::new();
		for (line_index, line) in lines.iter().enumerate().skip(start_line_index) {
			attr.push_str(line.trim());
			if line.trim_end().ends_with(']') {
				let kind = if attr.contains(SKIP_CALL_CHECK) {
					AttributeKind::SkipCallCheck
				} else {
					AttributeKind::Plain
				};
				return Some((kind, line_index));
			}
		}
	}

	None
}

fn is_doc_fence(line: &str) -> bool {
	line.trim_start().starts_with(DOC_FENCE_PREFIX)
}

fn print_count(
	count: usize,
	json: bool,
	attribute_mode: AttributeMode,
) {
	if json {
		println!(
			"{{\"count\":{count},\"mode\":{},\"description\":\"{}\"}}",
			attribute_mode.number(),
			attribute_mode.description()
		);
	} else {
		println!("{count}");
	}
}

fn print_list(
	entries: &[Entry],
	json: bool,
	attribute_mode: AttributeMode,
) {
	if json {
		println!("{{");
		println!("  \"count\": {},", entries.len());
		println!("  \"mode\": {},", attribute_mode.number());
		println!("  \"examples\": [");
		for (position, entry) in entries.iter().enumerate() {
			let comma = if position + 1 == entries.len() { "" } else { "," };
			println!(
				"    {{\"index\":{},\"path\":\"{}\",\"line\":{},\"kind\":\"{}\",\"attribute\":\"{}\"}}{}",
				entry.index,
				escape_json(&entry.path.display().to_string()),
				entry.line,
				entry.kind.as_json_value(),
				escape_json(entry.kind.as_attribute()),
				comma
			);
		}
		println!("  ]");
		println!("}}");
	} else {
		println!("{} {}", entries.len(), attribute_mode.description());
		for entry in entries {
			println!(
				"{}\t{}:{}\t{}",
				entry.index,
				entry.path.display(),
				entry.line,
				entry.kind.as_attribute()
			);
		}
	}
}

fn print_example(
	example: &Example,
	count: usize,
	json: bool,
	line_numbers: bool,
) {
	if json {
		println!("{{");
		println!("  \"count\": {count},");
		println!("  \"index\": {},", example.index);
		println!("  \"path\": \"{}\",", escape_json(&example.path.display().to_string()));
		println!("  \"line\": {},", example.line);
		println!("  \"kind\": \"{}\",", example.kind.as_json_value());
		println!("  \"attribute\": \"{}\",", escape_json(example.kind.as_attribute()));
		println!("  \"example_start_line\": {},", example.example_start_line);
		println!("  \"example_end_line\": {},", example.example_end_line);
		println!("  \"example\": \"{}\"", escape_json(&example.lines.join("\n")));
		println!("}}");
	} else {
		println!(
			"{}:{}-{} ({} at line {})",
			example.path.display(),
			example.example_start_line,
			example.example_end_line,
			example.kind.as_attribute(),
			example.line
		);
		if line_numbers {
			let width = example.example_end_line.to_string().len();
			for (offset, line) in example.lines.iter().enumerate() {
				let line_number = example.example_start_line + offset;
				println!("{line_number:>width$}\t{line}");
			}
		} else {
			for line in &example.lines {
				println!("{line}");
			}
		}
	}
}

fn escape_json(value: &str) -> String {
	let mut escaped = String::with_capacity(value.len());

	for character in value.chars() {
		match character {
			'"' => escaped.push_str("\\\""),
			'\\' => escaped.push_str("\\\\"),
			'\n' => escaped.push_str("\\n"),
			'\r' => escaped.push_str("\\r"),
			'\t' => escaped.push_str("\\t"),
			'\u{08}' => escaped.push_str("\\b"),
			'\u{0c}' => escaped.push_str("\\f"),
			character if character.is_control() => {
				escaped.push_str(&format!("\\u{:04x}", character as u32));
			}
			character => escaped.push(character),
		}
	}

	escaped
}
