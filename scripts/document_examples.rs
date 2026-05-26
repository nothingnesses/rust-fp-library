//! Count and inspect `#[document_examples]` doctest blocks.
//!
//! Usage:
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] --list [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] [--mode <1|2|3>] <index> [--line-numbers] [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] --invalid-reasons [--json]
//!   rust-script scripts/document_examples.rs -- [--path <dir>] --suspicious-reasons [--json]

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
const REASON: &str = "reason";
const SKIP_CALL_CHECK: &str = "skip_call_check";
const STALE_PLACEHOLDER_REASON: &str = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical.";
const DOC_FENCE_PREFIX: &str = "/// ```";
const RUST_CODE_TAGS: &[&str] = &["", "rust", "no_run", "rust,no_run"];

#[derive(Debug)]
enum Mode {
	Count,
	List,
	Example(usize),
	InvalidReasons,
	SuspiciousReasons,
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
}

#[derive(Debug)]
struct Entry {
	index: usize,
	path: PathBuf,
	line: usize,
	attribute_end_line: usize,
	kind: AttributeKind,
	attribute: String,
	has_skip_call_check: bool,
	has_reason: bool,
	reason: Option<String>,
	option_errors: Vec<String>,
}

#[derive(Debug)]
struct Example {
	index: usize,
	path: PathBuf,
	line: usize,
	kind: AttributeKind,
	attribute: String,
	example_start_line: usize,
	example_end_line: usize,
	lines: Vec<String>,
}

#[derive(Debug)]
struct AuditIssue {
	index: usize,
	path: PathBuf,
	line: usize,
	kind: AttributeKind,
	issue: &'static str,
	message: String,
	attribute: String,
	reason: Option<String>,
	item_kind: String,
	item_name: Option<String>,
}

#[derive(Debug)]
struct DocumentExamplesAttribute {
	end_line_index: usize,
	attribute: String,
	has_skip_call_check: bool,
	has_reason: bool,
	reason: Option<String>,
	option_errors: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DocumentedItemKind {
	FunctionLike,
	Other(String),
	Unknown,
}

impl DocumentedItemKind {
	fn as_str(&self) -> &str {
		match self {
			Self::FunctionLike => "function_like",
			Self::Other(kind) => kind,
			Self::Unknown => "unknown",
		}
	}
}

#[derive(Debug)]
struct DocumentedItem {
	kind: DocumentedItemKind,
	name: Option<String>,
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
	let collection_mode = match config.mode {
		Mode::InvalidReasons | Mode::SuspiciousReasons => AttributeMode::Both,
		Mode::Count | Mode::List | Mode::Example(_) => config.attribute_mode,
	};
	let entries = collect_entries(&config.path, &files, collection_mode)?;

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
		Mode::InvalidReasons => {
			let issues = collect_invalid_reason_issues(&config.path, &entries)?;
			print_audit_issues("invalid", &issues, config.json);
		}
		Mode::SuspiciousReasons => {
			let issues = collect_suspicious_reason_issues(&config.path, &entries)?;
			print_audit_issues("suspicious", &issues, config.json);
		}
	}

	Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
	let mut path = PathBuf::from(".");
	let mut attribute_mode = AttributeMode::SkipCallCheck;
	let mut json = false;
	let mut invalid_reasons = false;
	let mut line_numbers = false;
	let mut list = false;
	let mut suspicious_reasons = false;
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
			"--invalid-reasons" => invalid_reasons = true,
			"--line-numbers" => line_numbers = true,
			"--list" => list = true,
			"--suspicious-reasons" => suspicious_reasons = true,
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

	let audit_modes = usize::from(invalid_reasons) + usize::from(suspicious_reasons);
	if audit_modes > 1 {
		return Err("--invalid-reasons and --suspicious-reasons cannot be combined".to_string());
	}
	if audit_modes > 0 && (list || index.is_some() || line_numbers) {
		return Err(
			"--invalid-reasons and --suspicious-reasons cannot be combined with --list, --line-numbers, or an index"
				.to_string(),
		);
	}

	let mode = match (invalid_reasons, suspicious_reasons, list, index) {
		(true, false, false, None) => Mode::InvalidReasons,
		(false, true, false, None) => Mode::SuspiciousReasons,
		(false, false, true, Some(_)) => {
			return Err("--list cannot be combined with an index".to_string());
		}
		(false, false, true, None) => Mode::List,
		(false, false, false, Some(index)) => Mode::Example(index),
		(false, false, false, None) => Mode::Count,
		_ => unreachable!("invalid audit combinations were rejected above"),
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
  rust-script scripts/document_examples.rs -- [--path <dir>] --invalid-reasons [--json]
  rust-script scripts/document_examples.rs -- [--path <dir>] --suspicious-reasons [--json]

modes:
  1  #[document_examples(skip_call_check, reason = \"...\")] only (default)
  2  #[document_examples] without skip_call_check only
  3  both

audits:
  --invalid-reasons     report objective skip_call_check reason problems
  --suspicious-reasons  report subjective reason and example cleanup signals"
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
			let Some(attribute) = document_examples_attr(&lines, line_index) else {
				line_index += 1;
				continue;
			};
			let kind = if attribute.has_skip_call_check {
				AttributeKind::SkipCallCheck
			} else {
				AttributeKind::Plain
			};
			if attribute_mode.includes(kind) {
				entries.push(Entry {
					index: entries.len(),
					path: path.clone(),
					line: line_index + 1,
					attribute_end_line: attribute.end_line_index + 1,
					kind,
					attribute: attribute.attribute.clone(),
					has_skip_call_check: attribute.has_skip_call_check,
					has_reason: attribute.has_reason,
					reason: attribute.reason.clone(),
					option_errors: attribute.option_errors.clone(),
				});
			}
			line_index = attribute.end_line_index + 1;
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
				attribute: entry.attribute.clone(),
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

fn document_examples_attr(
	lines: &[&str],
	start_line_index: usize,
) -> Option<DocumentExamplesAttribute> {
	let line = lines.get(start_line_index)?.trim();
	if line == DOCUMENT_EXAMPLES_ATTR || line == QUALIFIED_DOCUMENT_EXAMPLES_ATTR {
		return Some(DocumentExamplesAttribute {
			end_line_index: start_line_index,
			attribute: line.to_string(),
			has_skip_call_check: false,
			has_reason: false,
			reason: None,
			option_errors: Vec::new(),
		});
	}

	if line.starts_with(DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX)
		|| line.starts_with(QUALIFIED_DOCUMENT_EXAMPLES_SKIP_CALL_CHECK_PREFIX)
	{
		let mut attr_lines = Vec::new();
		for (line_index, line) in lines.iter().enumerate().skip(start_line_index) {
			attr_lines.push(line.trim().to_string());
			if line.trim_end().ends_with(']') {
				let attribute = attr_lines.join(" ");
				let options = parse_document_examples_attribute_options(&attribute);
				return Some(DocumentExamplesAttribute {
					end_line_index: line_index,
					attribute,
					has_skip_call_check: options.has_skip_call_check,
					has_reason: options.has_reason,
					reason: options.reason,
					option_errors: options.errors,
				});
			}
		}
	}

	None
}

#[derive(Debug)]
struct ParsedAttributeOptions {
	has_skip_call_check: bool,
	has_reason: bool,
	reason: Option<String>,
	errors: Vec<String>,
}

fn parse_document_examples_attribute_options(attribute: &str) -> ParsedAttributeOptions {
	let mut parsed = ParsedAttributeOptions {
		has_skip_call_check: false,
		has_reason: false,
		reason: None,
		errors: Vec::new(),
	};

	let Some(inner) = attribute_inner(attribute) else {
		return parsed;
	};

	let mut skip_count = 0usize;
	let mut reason_count = 0usize;

	for argument in split_attribute_arguments(inner) {
		let argument = argument.trim();
		if argument.is_empty() {
			continue;
		}

		if argument == SKIP_CALL_CHECK {
			skip_count += 1;
			parsed.has_skip_call_check = true;
			continue;
		}

		if let Some(result) = parse_reason_argument(argument) {
			reason_count += 1;
			parsed.has_reason = true;
			match result {
				Ok(reason) => parsed.reason = Some(reason),
				Err(error) => parsed.errors.push(error),
			}
			continue;
		}

		parsed.errors.push(format!("unsupported document_examples argument `{argument}`"));
	}

	if skip_count > 1 {
		parsed.errors.push(format!("duplicate `{SKIP_CALL_CHECK}` argument"));
	}
	if reason_count > 1 {
		parsed.errors.push(format!("duplicate `{REASON}` argument"));
	}

	parsed
}

fn attribute_inner(attribute: &str) -> Option<&str> {
	let start = attribute.find('(')?;
	let end = attribute.rfind(')')?;
	if end <= start {
		return None;
	}
	Some(&attribute[start + 1 .. end])
}

fn split_attribute_arguments(inner: &str) -> Vec<String> {
	let mut arguments = Vec::new();
	let mut current = String::new();
	let mut in_string = false;
	let mut escaped = false;

	for character in inner.chars() {
		if in_string {
			current.push(character);
			if escaped {
				escaped = false;
			} else if character == '\\' {
				escaped = true;
			} else if character == '"' {
				in_string = false;
			}
			continue;
		}

		match character {
			'"' => {
				in_string = true;
				current.push(character);
			}
			',' => {
				arguments.push(current);
				current = String::new();
			}
			_ => current.push(character),
		}
	}

	arguments.push(current);
	arguments
}

fn parse_reason_argument(argument: &str) -> Option<Result<String, String>> {
	let Some(rest) = argument.strip_prefix(REASON) else {
		return None;
	};
	let rest = rest.trim_start();
	if !rest.starts_with('=') {
		return None;
	}
	let value = rest[1 ..].trim_start();
	Some(parse_string_literal(value).map_err(|error| format!("invalid `{REASON}` value: {error}")))
}

fn parse_string_literal(value: &str) -> Result<String, String> {
	let mut chars = value.chars();
	if chars.next() != Some('"') {
		return Err("expected a string literal".to_string());
	}

	let mut parsed = String::new();
	let mut escaped = false;
	let mut closed = false;
	let mut rest = String::new();

	for character in chars {
		if closed {
			rest.push(character);
			continue;
		}

		if escaped {
			match character {
				'n' => parsed.push('\n'),
				'r' => parsed.push('\r'),
				't' => parsed.push('\t'),
				'\\' => parsed.push('\\'),
				'"' => parsed.push('"'),
				other => parsed.push(other),
			}
			escaped = false;
		} else if character == '\\' {
			escaped = true;
		} else if character == '"' {
			closed = true;
		} else {
			parsed.push(character);
		}
	}

	if escaped {
		return Err("unterminated escape sequence".to_string());
	}
	if !closed {
		return Err("unterminated string literal".to_string());
	}
	if !rest.trim().is_empty() {
		return Err("unexpected tokens after string literal".to_string());
	}

	Ok(parsed)
}

fn is_doc_fence(line: &str) -> bool {
	line.trim_start().starts_with(DOC_FENCE_PREFIX)
}

fn collect_invalid_reason_issues(
	root: &Path,
	entries: &[Entry],
) -> Result<Vec<AuditIssue>, String> {
	let mut issues = Vec::new();

	for entry in entries {
		if is_compile_fail_fixture(entry) {
			continue;
		}

		let item = documented_item(root, entry)?;

		for error in &entry.option_errors {
			issues.push(audit_issue(entry, "invalid_option", error.clone(), &item));
		}

		if entry.has_skip_call_check && !entry.has_reason {
			issues.push(audit_issue(
				entry,
				"missing_reason",
				format!("`{SKIP_CALL_CHECK}` must include `{REASON} = \"...\"`"),
				&item,
			));
		}

		if let Some(reason) = &entry.reason {
			if reason.trim().is_empty() {
				issues.push(audit_issue(
					entry,
					"empty_reason",
					format!("`{REASON}` cannot be empty"),
					&item,
				));
			}
			if reason == STALE_PLACEHOLDER_REASON {
				issues.push(audit_issue(
					entry,
					"stale_placeholder_reason",
					"reason still uses the migration placeholder".to_string(),
					&item,
				));
			}
		}

		if entry.has_reason && !entry.has_skip_call_check {
			issues.push(audit_issue(
				entry,
				"reason_without_skip",
				format!("`{REASON}` is only meaningful with `{SKIP_CALL_CHECK}`"),
				&item,
			));
		}

		if entry.has_skip_call_check {
			if item.kind != DocumentedItemKind::FunctionLike {
				issues.push(audit_issue(
					entry,
					"skip_on_non_function_item",
					"`skip_call_check` is present, but direct-call validation has no function or method target"
						.to_string(),
					&item,
				));
			} else if let Some(item_name) = &item.name {
				let code_blocks = rust_code_blocks_for_entry(root, entry)?;
				if !code_blocks.is_empty()
					&& code_blocks.iter().all(|code| contains_call_to_item(code, item_name))
				{
					issues.push(audit_issue(
						entry,
						"unnecessary_skip",
						format!("every Rust code block appears to call `{item_name}` directly"),
						&item,
					));
				}
			}
		}
	}

	Ok(issues)
}

fn collect_suspicious_reason_issues(
	root: &Path,
	entries: &[Entry],
) -> Result<Vec<AuditIssue>, String> {
	let mut issues = Vec::new();
	let mut reason_counts = std::collections::BTreeMap::<String, usize>::new();

	for entry in entries {
		if is_compile_fail_fixture(entry) {
			continue;
		}

		if let Some(reason) = normalized_reason(entry) {
			if reason != STALE_PLACEHOLDER_REASON {
				*reason_counts.entry(reason).or_default() += 1;
			}
		}
	}

	for entry in entries {
		if is_compile_fail_fixture(entry) {
			continue;
		}

		let item = documented_item(root, entry)?;

		if let Some(reason) = normalized_reason(entry) {
			let lower_reason = reason.to_ascii_lowercase();
			if reason.len() < 24 {
				issues.push(audit_issue(
					entry,
					"short_reason",
					"reason is very short and may not explain why direct-call validation is skipped"
						.to_string(),
					&item,
				));
			}
			if lower_reason.contains("todo")
				|| lower_reason.contains("fixme")
				|| lower_reason.contains("tbd")
				|| lower_reason.contains("audit")
			{
				issues.push(audit_issue(
					entry,
					"todo_reason",
					"reason contains TODO-style cleanup wording".to_string(),
					&item,
				));
			}
			if reason_counts.get(&reason).copied().unwrap_or_default() > 3 {
				issues.push(audit_issue(
					entry,
					"repeated_reason",
					"reason text is repeated more than three times".to_string(),
					&item,
				));
			}
		}

		for code in rust_code_blocks_for_entry(root, entry)? {
			if has_weak_assertion_signal(&code) {
				issues.push(audit_issue(
					entry,
					"weak_assertion_signal",
					"example contains an assertion pattern that may be too weak".to_string(),
					&item,
				));
				break;
			}
		}
	}

	Ok(issues)
}

fn is_compile_fail_fixture(entry: &Entry) -> bool {
	let path = entry.path.to_string_lossy();
	path.contains("/tests/ui/") || path.starts_with("tests/ui/")
}

fn audit_issue(
	entry: &Entry,
	issue: &'static str,
	message: String,
	item: &DocumentedItem,
) -> AuditIssue {
	AuditIssue {
		index: entry.index,
		path: entry.path.clone(),
		line: entry.line,
		kind: entry.kind,
		issue,
		message,
		attribute: entry.attribute.clone(),
		reason: entry.reason.clone(),
		item_kind: item.kind.as_str().to_string(),
		item_name: item.name.clone(),
	}
}

fn normalized_reason(entry: &Entry) -> Option<String> {
	entry
		.reason
		.as_ref()
		.map(|reason| reason.trim().to_string())
		.filter(|reason| !reason.is_empty())
}

fn documented_item(
	root: &Path,
	entry: &Entry,
) -> Result<DocumentedItem, String> {
	let contents = read_to_string(root, &entry.path)?;
	let lines: Vec<_> = contents.lines().collect();
	let mut line_index = entry.attribute_end_line;

	while line_index < lines.len() {
		let line = lines[line_index];
		let trimmed = line.trim_start();
		if trimmed.is_empty() || trimmed.starts_with("///") {
			line_index += 1;
			continue;
		}
		if trimmed.starts_with("#[") {
			line_index = skip_attribute_block(&lines, line_index);
			continue;
		}
		if let Some(name) = function_name_from_line(trimmed) {
			return Ok(DocumentedItem {
				kind: DocumentedItemKind::FunctionLike,
				name: Some(name),
			});
		}
		return Ok(DocumentedItem {
			kind: DocumentedItemKind::Other(item_kind_from_line(trimmed)),
			name: None,
		});
	}

	Ok(DocumentedItem {
		kind: DocumentedItemKind::Unknown,
		name: None,
	})
}

fn skip_attribute_block(
	lines: &[&str],
	start_line_index: usize,
) -> usize {
	for (line_index, line) in lines.iter().enumerate().skip(start_line_index) {
		if line.trim_end().ends_with(']') {
			return line_index + 1;
		}
	}
	lines.len()
}

fn function_name_from_line(line: &str) -> Option<String> {
	let tokens = identifier_tokens(line);
	for (index, token) in tokens.iter().enumerate() {
		if token == "fn" {
			return tokens.get(index + 1).cloned();
		}
	}
	None
}

fn item_kind_from_line(line: &str) -> String {
	let tokens = identifier_tokens(line);
	for token in tokens {
		match token.as_str() {
			"fn" => return "function_like".to_string(),
			"struct" | "enum" | "union" | "trait" | "impl" | "type" | "const" | "static"
			| "mod" => return token,
			_ => {}
		}
	}
	"other".to_string()
}

fn identifier_tokens(line: &str) -> Vec<String> {
	let mut tokens = Vec::new();
	let mut current = String::new();

	for character in line.chars() {
		if character == '_' || character.is_ascii_alphanumeric() {
			current.push(character);
		} else if !current.is_empty() {
			tokens.push(std::mem::take(&mut current));
		}
	}

	if !current.is_empty() {
		tokens.push(current);
	}

	tokens
}

fn rust_code_blocks_for_entry(
	root: &Path,
	entry: &Entry,
) -> Result<Vec<String>, String> {
	let contents = read_to_string(root, &entry.path)?;
	let mut doc_lines = Vec::new();

	for line in contents.lines().skip(entry.attribute_end_line) {
		let trimmed = line.trim_start();
		if let Some(doc_line) = trimmed.strip_prefix("///") {
			doc_lines.push(doc_line.strip_prefix(' ').unwrap_or(doc_line).to_string());
		} else if trimmed.is_empty() {
			continue;
		} else {
			break;
		}
	}

	Ok(extract_rust_code_blocks(&doc_lines))
}

fn extract_rust_code_blocks(doc_lines: &[String]) -> Vec<String> {
	let mut blocks = Vec::new();
	let mut current = Vec::new();
	let mut in_rust_block = false;
	let mut in_skipped_block = false;

	for line in doc_lines {
		let trimmed = line.trim();
		if in_rust_block {
			if trimmed == "```" {
				blocks.push(current.join("\n"));
				current.clear();
				in_rust_block = false;
			} else {
				current.push(line.clone());
			}
			continue;
		}
		if in_skipped_block {
			if trimmed == "```" {
				in_skipped_block = false;
			}
			continue;
		}
		if let Some(tag) = trimmed.strip_prefix("```") {
			if RUST_CODE_TAGS.contains(&tag.trim()) {
				in_rust_block = true;
			} else {
				in_skipped_block = true;
			}
		}
	}

	blocks
}

fn contains_call_to_item(
	code: &str,
	item_name: &str,
) -> bool {
	let needles = [format!("{item_name}("), format!(".{item_name}("), format!("::{item_name}(")];
	needles.iter().any(|needle| code.contains(needle))
}

fn has_weak_assertion_signal(code: &str) -> bool {
	let compact: String = code.chars().filter(|character| !character.is_whitespace()).collect();
	compact.contains("assert!(true)")
		|| compact.contains("assert_eq!(true,true)")
		|| compact.contains("assert_eq!(1,1)")
		|| compact.contains("assert_eq!(value,value)")
		|| compact.contains("assert!(matches!(") && compact.contains(",_)")
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
				escape_json(&entry.attribute),
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
				entry.attribute
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
		println!("  \"attribute\": \"{}\",", escape_json(&example.attribute));
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
			example.attribute,
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

fn print_audit_issues(
	label: &str,
	issues: &[AuditIssue],
	json: bool,
) {
	if json {
		println!("{{");
		println!("  \"count\": {},", issues.len());
		println!("  \"kind\": \"{}\",", escape_json(label));
		println!("  \"issues\": [");
		for (position, issue) in issues.iter().enumerate() {
			let comma = if position + 1 == issues.len() { "" } else { "," };
			let item_name = issue.item_name.as_deref().unwrap_or("");
			let reason = issue.reason.as_deref().unwrap_or("");
			println!(
				"    {{\"index\":{},\"path\":\"{}\",\"line\":{},\"kind\":\"{}\",\"issue\":\"{}\",\"message\":\"{}\",\"reason\":\"{}\",\"item_kind\":\"{}\",\"item_name\":\"{}\",\"attribute\":\"{}\"}}{}",
				issue.index,
				escape_json(&issue.path.display().to_string()),
				issue.line,
				issue.kind.as_json_value(),
				escape_json(issue.issue),
				escape_json(&issue.message),
				escape_json(reason),
				escape_json(&issue.item_kind),
				escape_json(item_name),
				escape_json(&issue.attribute),
				comma
			);
		}
		println!("  ]");
		println!("}}");
	} else {
		println!("{} {label} document_examples entries", issues.len());
		for issue in issues {
			let item = issue
				.item_name
				.as_ref()
				.map(|name| format!("{} `{name}`", issue.item_kind))
				.unwrap_or_else(|| issue.item_kind.clone());
			println!(
				"{}\t{}:{}\t{}\t{}\t{}",
				issue.index,
				issue.path.display(),
				issue.line,
				issue.issue,
				item,
				issue.message
			);
			if let Some(reason) = &issue.reason {
				println!("\treason: {reason}");
			}
			println!("\tattribute: {}", issue.attribute);
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
