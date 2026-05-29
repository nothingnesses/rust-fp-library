//! Collect a source item inventory using rust-analyzer's LSP document symbols.
//!
//! Usage:
//!   rust-script scripts/item_inventory.rs -- --path <file-or-dir> [--path <file-or-dir>] [--format markdown|json] [--include-tests]
//!   rust-script scripts/item_inventory.rs -- --self-check
//!
//! ```cargo
//! [workspace]
//!
//! [dependencies]
//! serde_json = "1.0"
//! ```

use {
	serde_json::{
		Value,
		json,
	},
	std::{
		env,
		fs,
		io::{
			BufRead,
			BufReader,
			BufWriter,
			Read,
			Write,
		},
		path::{
			Component,
			Path,
			PathBuf,
		},
		process::{
			Child,
			ChildStdin,
			ChildStdout,
			Command,
			ExitCode,
			Stdio,
		},
	},
};

const INCLUDED_KIND_NAMES: &[&str] = &[
	"Class",
	"Constant",
	"Constructor",
	"Enum",
	"EnumMember",
	"Function",
	"Interface",
	"Method",
	"Module",
	"Namespace",
	"Object",
	"Struct",
	"TypeParameter",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
	Json,
	Markdown,
}

#[derive(Debug)]
struct Config {
	format: OutputFormat,
	include_tests: bool,
	paths: Vec<PathBuf>,
	self_check: bool,
	title: String,
}

#[derive(Debug)]
struct InventoryItem {
	file: PathBuf,
	line: usize,
	kind: String,
	item_path: String,
	detail: String,
	description: String,
	description_source: String,
}

#[derive(Debug)]
struct SourceFile {
	abs_path: PathBuf,
	rel_path: PathBuf,
	module_path: String,
	text: String,
	lines: Vec<String>,
}

struct LspClient {
	child: Child,
	next_id: i64,
	reader: BufReader<ChildStdout>,
	writer: BufWriter<ChildStdin>,
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
	if config.self_check {
		run_self_check()?;
		return Ok(());
	}

	let root = repo_root()?;
	let files = collect_source_files(&root, &config.paths, config.include_tests)?;
	let mut client = LspClient::start(&root)?;
	client.initialize(&root)?;

	let mut items = Vec::new();
	for file in &files {
		let symbols = client.document_symbols(file)?;
		flatten_document_symbols(file, &symbols, config.include_tests, &mut items, None, false)?;
	}

	client.shutdown();

	match config.format {
		OutputFormat::Markdown => print_markdown(&config.title, &config.paths, &items),
		OutputFormat::Json => print_json(&config.paths, &items)?,
	}

	Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
	let mut format = OutputFormat::Markdown;
	let mut include_tests = false;
	let mut paths = Vec::new();
	let mut self_check = false;
	let mut title = "Item Inventory".to_string();
	let mut args = args.peekable();

	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--" => {}
			"--help" | "-h" => {
				println!("{}", usage());
				std::process::exit(0);
			}
			"--format" => {
				let value =
					args.next().ok_or_else(|| "--format requires markdown or json".to_string())?;
				format = match value.as_str() {
					"json" => OutputFormat::Json,
					"markdown" => OutputFormat::Markdown,
					_ => return Err(format!("--format requires markdown or json: {value}")),
				};
			}
			"--include-tests" => include_tests = true,
			"--path" => {
				let value = args
					.next()
					.ok_or_else(|| "--path requires a file or directory argument".to_string())?;
				paths.push(PathBuf::from(value));
			}
			"--self-check" => self_check = true,
			"--title" => {
				title = args.next().ok_or_else(|| "--title requires a value".to_string())?;
			}
			_ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
			_ => return Err(format!("unexpected argument: {arg}")),
		}
	}

	if paths.is_empty() {
		paths.push(PathBuf::from("."));
	}

	if self_check
		&& (include_tests || format != OutputFormat::Markdown || paths != [PathBuf::from(".")])
	{
		return Err("--self-check cannot be combined with other options".to_string());
	}

	Ok(Config {
		format,
		include_tests,
		paths,
		self_check,
		title,
	})
}

fn usage() -> &'static str {
	"usage:
  rust-script scripts/item_inventory.rs -- --path <file-or-dir> [--path <file-or-dir>] [--format markdown|json] [--include-tests] [--title <title>]
  rust-script scripts/item_inventory.rs -- --self-check

options:
  --path <file-or-dir>  include tracked and untracked Rust files under this path
  --format <format>     output markdown or json, default markdown
  --include-tests       include test files and symbols under test modules
  --title <title>       markdown heading, default Item Inventory
  --self-check          run internal parser and formatting checks"
}

fn run_self_check() -> Result<(), String> {
	let uri = path_to_file_uri(Path::new("/tmp/a b.rs"))?;
	if uri != "file:///tmp/a%20b.rs" {
		return Err(format!("file URI encoding failed: {uri}"));
	}
	if markdown_escape("a|b`c") != "a\\|b\\`c" {
		return Err("markdown escaping failed".to_string());
	}
	let lines = vec![
		"/// Short sentence. More details.".to_string(),
		"#[document_signature]".to_string(),
		"fn item() {}".to_string(),
	];
	let description = doc_description(&lines, 3, "Function");
	if description != "Short sentence." {
		return Err(format!("doc description extraction failed: {description}"));
	}
	let lines = vec![
		"/// Variant sentence.".to_string(),
		"Variant,".to_string(),
		"".to_string(),
		"impl Clone for Item {}".to_string(),
	];
	let description = doc_description(&lines, 4, "Impl");
	if !description.is_empty() {
		return Err(format!("doc description leaked across item boundary: {description}"));
	}
	let lines = vec![
		"//! Module sentence. More details.".to_string(),
		"".to_string(),
		"#[fp_macros::document_module]".to_string(),
		"mod inner {}".to_string(),
	];
	let description = doc_description(&lines, 4, "Module");
	if description != "Module sentence." {
		return Err(format!("module doc extraction failed: {description}"));
	}

	println!("item_inventory self-check passed");
	Ok(())
}

fn repo_root() -> Result<PathBuf, String> {
	let mut current = env::current_dir().map_err(|error| format!("failed to get cwd: {error}"))?;
	loop {
		if current.join("justfile").is_file() {
			return Ok(current);
		}
		if !current.pop() {
			return Err("could not find repository root containing justfile".to_string());
		}
	}
}

fn collect_source_files(
	root: &Path,
	paths: &[PathBuf],
	include_tests: bool,
) -> Result<Vec<SourceFile>, String> {
	let output = Command::new("git")
		.current_dir(root)
		.args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
		.output()
		.map_err(|error| format!("failed to run git ls-files: {error}"))?;

	if !output.status.success() {
		return Err(format!(
			"git ls-files failed: {}",
			String::from_utf8_lossy(&output.stderr).trim()
		));
	}

	let selected_roots = normalize_selected_paths(root, paths)?;
	let mut files = Vec::new();
	for raw_path in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
		let rel_path = PathBuf::from(String::from_utf8_lossy(raw_path).into_owned());
		if rel_path.extension().is_none_or(|extension| extension != "rs") {
			continue;
		}
		if !include_tests && is_test_file(&rel_path) {
			continue;
		}
		let abs_path = root.join(&rel_path);
		if !selected_roots.iter().any(|selected| selected.matches(root, &abs_path, &rel_path)) {
			continue;
		}
		let text = fs::read_to_string(&abs_path)
			.map_err(|error| format!("failed to read {}: {error}", rel_path.display()))?;
		let lines = text.lines().map(str::to_string).collect();
		let module_path = module_path_for_file(&rel_path);
		files.push(SourceFile {
			abs_path,
			rel_path,
			module_path,
			text,
			lines,
		});
	}

	files.sort_by(|left, right| left.rel_path.cmp(&right.rel_path));
	Ok(files)
}

#[derive(Debug)]
enum SelectedPath {
	File(PathBuf),
	Dir(PathBuf),
}

impl SelectedPath {
	fn matches(
		&self,
		root: &Path,
		abs_path: &Path,
		rel_path: &Path,
	) -> bool {
		match self {
			Self::File(path) => path == abs_path || path == rel_path || root.join(path) == abs_path,
			Self::Dir(path) =>
				abs_path.starts_with(path)
					|| rel_path.starts_with(path)
					|| abs_path.starts_with(root.join(path)),
		}
	}
}

fn normalize_selected_paths(
	root: &Path,
	paths: &[PathBuf],
) -> Result<Vec<SelectedPath>, String> {
	let mut selected = Vec::new();
	for path in paths {
		let abs = if path.is_absolute() { path.clone() } else { root.join(path) };
		if abs.is_file() {
			selected.push(SelectedPath::File(abs.canonicalize().unwrap_or(abs)));
		} else if abs.is_dir() {
			selected.push(SelectedPath::Dir(abs.canonicalize().unwrap_or(abs)));
		} else {
			return Err(format!("--path does not exist: {}", path.display()));
		}
	}
	Ok(selected)
}

fn is_test_file(path: &Path) -> bool {
	let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
	file_name == "tests.rs"
		|| file_name.ends_with("_tests.rs")
		|| path
			.components()
			.any(|component| matches!(component, Component::Normal(value) if value == "tests"))
}

fn module_path_for_file(path: &Path) -> String {
	let parts: Vec<String> = path
		.components()
		.filter_map(|component| match component {
			Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
			_ => None,
		})
		.collect();

	let (crate_name, mut module_parts) = if parts.get(0).is_some_and(|part| part == "fp-library")
		&& parts.get(1).is_some_and(|part| part == "src")
	{
		("fp_library".to_string(), parts[2 ..].to_vec())
	} else if parts.get(0).is_some_and(|part| part == "fp-macros")
		&& parts.get(1).is_some_and(|part| part == "src")
	{
		("fp_macros".to_string(), parts[2 ..].to_vec())
	} else {
		let crate_name = parts.first().map_or("crate".to_string(), |part| part.replace('-', "_"));
		(crate_name, parts.get(1 ..).unwrap_or_default().to_vec())
	};

	if module_parts.is_empty() {
		return crate_name;
	}

	if let Some(last) = module_parts.last_mut() {
		if last == "lib.rs" || last == "main.rs" || last == "mod.rs" {
			module_parts.pop();
		} else if let Some(stripped) = last.strip_suffix(".rs") {
			*last = stripped.to_string();
		}
	}

	let mut result = vec![crate_name];
	result.extend(module_parts.into_iter().filter(|part| !part.is_empty()));
	result.join("::")
}

impl LspClient {
	fn start(root: &Path) -> Result<Self, String> {
		let mut child = Command::new("rust-analyzer")
			.current_dir(root)
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()
			.map_err(|error| format!("failed to spawn rust-analyzer: {error}"))?;

		let reader = BufReader::new(
			child
				.stdout
				.take()
				.ok_or_else(|| "failed to capture rust-analyzer stdout".to_string())?,
		);
		let writer = BufWriter::new(
			child
				.stdin
				.take()
				.ok_or_else(|| "failed to capture rust-analyzer stdin".to_string())?,
		);

		Ok(Self {
			child,
			next_id: 1,
			reader,
			writer,
		})
	}

	fn initialize(
		&mut self,
		root: &Path,
	) -> Result<(), String> {
		let root_uri = path_to_file_uri(root)?;
		let params = json!({
			"processId": Value::Null,
			"rootUri": root_uri,
			"capabilities": {
				"textDocument": {
					"documentSymbol": {
						"hierarchicalDocumentSymbolSupport": true
					},
					"hover": {
						"contentFormat": ["markdown", "plaintext"]
					}
				},
				"workspace": {
					"workspaceFolders": true
				},
				"window": {
					"workDoneProgress": false
				}
			},
			"workspaceFolders": [{
				"uri": root_uri,
				"name": root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace")
			}]
		});
		self.request("initialize", params)?;
		self.notify("initialized", json!({}))?;
		Ok(())
	}

	fn document_symbols(
		&mut self,
		file: &SourceFile,
	) -> Result<Value, String> {
		let uri = path_to_file_uri(&file.abs_path)?;
		self.notify(
			"textDocument/didOpen",
			json!({
				"textDocument": {
					"uri": uri,
					"languageId": "rust",
					"version": 1,
					"text": file.text
				}
			}),
		)?;
		let result = self.request(
			"textDocument/documentSymbol",
			json!({
				"textDocument": {
					"uri": uri
				}
			}),
		)?;
		self.notify(
			"textDocument/didClose",
			json!({
				"textDocument": {
					"uri": uri
				}
			}),
		)?;
		Ok(result)
	}

	fn shutdown(&mut self) {
		let _ = self.request("shutdown", Value::Null);
		let _ = self.notify("exit", Value::Null);
		let _ = self.child.wait();
	}

	fn request(
		&mut self,
		method: &str,
		params: Value,
	) -> Result<Value, String> {
		let id = self.next_id;
		self.next_id += 1;
		self.send(json!({
			"jsonrpc": "2.0",
			"id": id,
			"method": method,
			"params": params
		}))?;

		loop {
			let message = self.read()?;
			if message.get("id").and_then(Value::as_i64) == Some(id) {
				if let Some(error) = message.get("error") {
					return Err(format!("LSP request `{method}` failed: {error}"));
				}
				return Ok(message.get("result").cloned().unwrap_or(Value::Null));
			}
			if message.get("id").is_some() && message.get("method").is_some() {
				let response_id = message.get("id").cloned().unwrap_or(Value::Null);
				self.send(json!({
					"jsonrpc": "2.0",
					"id": response_id,
					"result": Value::Null
				}))?;
			}
		}
	}

	fn notify(
		&mut self,
		method: &str,
		params: Value,
	) -> Result<(), String> {
		self.send(json!({
			"jsonrpc": "2.0",
			"method": method,
			"params": params
		}))
	}

	fn send(
		&mut self,
		message: Value,
	) -> Result<(), String> {
		let body =
			serde_json::to_vec(&message).map_err(|error| format!("json encode failed: {error}"))?;
		write!(self.writer, "Content-Length: {}\r\n\r\n", body.len())
			.map_err(|error| format!("failed to write LSP header: {error}"))?;
		self.writer
			.write_all(&body)
			.map_err(|error| format!("failed to write LSP body: {error}"))?;
		self.writer.flush().map_err(|error| format!("failed to flush LSP message: {error}"))?;
		Ok(())
	}

	fn read(&mut self) -> Result<Value, String> {
		let mut content_length = None;
		loop {
			let mut line = String::new();
			let bytes = self
				.reader
				.read_line(&mut line)
				.map_err(|error| format!("failed to read LSP header: {error}"))?;
			if bytes == 0 {
				return Err("rust-analyzer closed stdout".to_string());
			}
			let line = line.trim_end_matches(['\r', '\n']);
			if line.is_empty() {
				break;
			}
			if let Some(value) = line.strip_prefix("Content-Length: ") {
				content_length =
					Some(value.parse::<usize>().map_err(|error| {
						format!("invalid LSP content length `{value}`: {error}")
					})?);
			}
		}

		let length =
			content_length.ok_or_else(|| "LSP message missing Content-Length".to_string())?;
		let mut body = vec![0; length];
		self.reader
			.read_exact(&mut body)
			.map_err(|error| format!("failed to read LSP body: {error}"))?;
		serde_json::from_slice(&body).map_err(|error| format!("failed to parse LSP JSON: {error}"))
	}
}

fn flatten_document_symbols(
	file: &SourceFile,
	symbols: &Value,
	include_tests: bool,
	items: &mut Vec<InventoryItem>,
	parent_path: Option<&str>,
	parent_skipped: bool,
) -> Result<(), String> {
	let Some(array) = symbols.as_array() else {
		return Ok(());
	};

	for symbol in array {
		if symbol.get("location").is_some() {
			flatten_symbol_information(
				file,
				symbol,
				include_tests,
				items,
				parent_path,
				parent_skipped,
			);
			continue;
		}

		let name = symbol.get("name").and_then(Value::as_str).unwrap_or("<unnamed>");
		let raw_kind = symbol.get("kind").and_then(Value::as_u64).unwrap_or(0);
		let kind = display_kind(raw_kind, name);
		let line = symbol
			.get("selectionRange")
			.and_then(|range| range.get("start"))
			.and_then(|start| start.get("line"))
			.and_then(Value::as_u64)
			.map_or(1, |line| line as usize + 1);
		let detail = symbol.get("detail").and_then(Value::as_str).unwrap_or_default().to_string();
		let item_path = make_item_path(&file.module_path, parent_path, name);
		let skip_subtree =
			parent_skipped || (!include_tests && is_test_symbol(name, &item_path, &kind));

		if !skip_subtree && included_kind(&kind, name) {
			let (description, description_source) =
				description_for_symbol(file, line, &kind, name, parent_path, &detail);
			items.push(InventoryItem {
				file: file.rel_path.clone(),
				line,
				kind,
				item_path: item_path.clone(),
				detail,
				description,
				description_source,
			});
		}

		if let Some(children) = symbol.get("children") {
			flatten_document_symbols(
				file,
				children,
				include_tests,
				items,
				Some(&item_path),
				skip_subtree,
			)?;
		}
	}

	Ok(())
}

fn flatten_symbol_information(
	file: &SourceFile,
	symbol: &Value,
	include_tests: bool,
	items: &mut Vec<InventoryItem>,
	parent_path: Option<&str>,
	parent_skipped: bool,
) {
	let name = symbol.get("name").and_then(Value::as_str).unwrap_or("<unnamed>");
	let raw_kind = symbol.get("kind").and_then(Value::as_u64).unwrap_or(0);
	let kind = display_kind(raw_kind, name);
	let line = symbol
		.get("location")
		.and_then(|location| location.get("range"))
		.and_then(|range| range.get("start"))
		.and_then(|start| start.get("line"))
		.and_then(Value::as_u64)
		.map_or(1, |line| line as usize + 1);
	let item_path = make_item_path(&file.module_path, parent_path, name);
	if parent_skipped
		|| (!include_tests && is_test_symbol(name, &item_path, &kind))
		|| !included_kind(&kind, name)
	{
		return;
	}
	let (description, description_source) =
		description_for_symbol(file, line, &kind, name, parent_path, "");
	items.push(InventoryItem {
		file: file.rel_path.clone(),
		line,
		kind,
		item_path,
		detail: String::new(),
		description,
		description_source,
	});
}

fn included_kind(
	kind: &str,
	name: &str,
) -> bool {
	INCLUDED_KIND_NAMES.contains(&kind)
		|| kind == "Trait"
		|| kind == "Variant"
		|| kind == "Impl"
		|| (kind == "Object" && name.starts_with("impl "))
}

fn is_test_symbol(
	name: &str,
	item_path: &str,
	kind: &str,
) -> bool {
	(kind == "Module" && name == "tests")
		|| item_path.contains("::tests::")
		|| name.starts_with("test_")
}

fn display_kind(
	raw_kind: u64,
	name: &str,
) -> String {
	let kind = match raw_kind {
		2 => "Module",
		3 => "Namespace",
		5 => "Class",
		6 => "Method",
		9 => "Constructor",
		10 => "Enum",
		11 => "Trait",
		12 => "Function",
		14 => "Const",
		19 if name.starts_with("impl ") => "Impl",
		19 => "Object",
		22 => "Variant",
		23 => "Struct",
		26 => "TypeParameter",
		_ => "Other",
	};
	kind.to_string()
}

fn make_item_path(
	module_path: &str,
	parent_path: Option<&str>,
	name: &str,
) -> String {
	if let Some(parent) = parent_path {
		format!("{parent}::{name}")
	} else {
		format!("{module_path}::{name}")
	}
}

fn description_for_symbol(
	file: &SourceFile,
	line: usize,
	kind: &str,
	name: &str,
	parent_path: Option<&str>,
	detail: &str,
) -> (String, String) {
	let description = doc_description(&file.lines, line, kind);
	if !description.is_empty() {
		return (description, "doc".to_string());
	}

	(fallback_description(kind, name, parent_path, detail), "fallback".to_string())
}

fn doc_description(
	lines: &[String],
	line: usize,
	kind: &str,
) -> String {
	for block in adjacent_doc_blocks(lines, line) {
		let sentence = first_sentence(&block);
		if !sentence.is_empty() {
			return sentence;
		}
	}
	if kind == "Module" {
		return first_sentence(&leading_module_doc_block(lines));
	}
	String::new()
}

fn adjacent_doc_blocks(
	lines: &[String],
	line: usize,
) -> Vec<Vec<String>> {
	let Some(mut index) = line.checked_sub(2) else {
		return Vec::new();
	};
	let mut blocks = Vec::new();
	loop {
		let Some(source_line) = lines.get(index) else {
			break;
		};
		let trimmed = source_line.trim();
		if doc_line_text(source_line).is_some() {
			let mut block = Vec::new();
			loop {
				let Some(source_line) = lines.get(index) else {
					break;
				};
				let Some(doc) = doc_line_text(source_line) else {
					break;
				};
				block.push(doc);
				if index == 0 {
					break;
				}
				index -= 1;
			}
			block.reverse();
			blocks.push(block);
			if index == 0 || doc_line_text(lines.get(index).map_or("", String::as_str)).is_some() {
				break;
			}
			continue;
		}
		if trimmed.is_empty() {
			break;
		}
		if let Some(next_index) = skip_attribute_block_backwards(lines, index) {
			index = next_index;
		} else {
			break;
		}
	}
	blocks
}

fn skip_attribute_block_backwards(
	lines: &[String],
	mut index: usize,
) -> Option<usize> {
	loop {
		let trimmed = lines.get(index)?.trim();
		if !is_attribute_line(trimmed) {
			return None;
		}
		if trimmed.starts_with("#[") {
			return index.checked_sub(1);
		}
		index = index.checked_sub(1)?;
	}
}

fn is_attribute_line(line: &str) -> bool {
	line.starts_with("#[")
		|| line.starts_with("]")
		|| line.starts_with(")]")
		|| line.ends_with("]")
		|| line.ends_with("],")
		|| line.ends_with(")")
		|| line.ends_with("),")
		|| line.starts_with('"')
}

fn leading_module_doc_block(lines: &[String]) -> Vec<String> {
	let mut docs = Vec::new();
	for line in lines {
		let trimmed = line.trim();
		if let Some(doc) = doc_line_text(trimmed) {
			docs.push(doc);
		} else if !trimmed.is_empty() {
			break;
		}
	}
	docs
}

fn doc_line_text(line: &str) -> Option<String> {
	let trimmed = line.trim();
	if let Some(rest) = trimmed.strip_prefix("///") {
		Some(rest.trim().to_string())
	} else if let Some(rest) = trimmed.strip_prefix("//!") {
		Some(rest.trim().to_string())
	} else if let Some(rest) = trimmed.strip_prefix("#[doc = \"") {
		let value = rest.strip_suffix("\"]")?;
		Some(value.replace("\\\"", "\"").replace("\\n", "\n").replace("\\\\", "\\"))
	} else {
		None
	}
}

fn first_sentence(docs: &[String]) -> String {
	let text = docs.iter().filter(|line| !line.is_empty()).cloned().collect::<Vec<_>>().join(" ");
	let text = compact_whitespace(&text);
	if text.is_empty() || looks_like_example(&text) {
		return String::new();
	}
	for marker in [". ", "? ", "! "] {
		if let Some(index) = text.find(marker) {
			return text[.. index + 1].to_string();
		}
	}
	text
}

fn looks_like_example(text: &str) -> bool {
	let trimmed = text.trim_start();
	trimmed.starts_with("```")
		|| trimmed.starts_with("#")
		|| trimmed.starts_with("use ")
		|| trimmed.starts_with("let ")
		|| trimmed.starts_with("assert")
		|| trimmed.contains("assert_eq!")
		|| trimmed.contains("use fp_library")
}

fn fallback_description(
	kind: &str,
	name: &str,
	parent_path: Option<&str>,
	detail: &str,
) -> String {
	if kind == "Impl" {
		if let Some((trait_name, target)) =
			name.strip_prefix("impl ").and_then(|label| label.split_once(" for "))
		{
			return format!("Implements `{trait_name}` for `{target}`.");
		}
		return format!(
			"Inherent implementation block for `{}`.",
			name.strip_prefix("impl ").unwrap_or(name)
		);
	}
	if kind == "Module" {
		return "Module namespace or submodule declaration.".to_string();
	}
	if let Some(parent) = parent_path {
		if kind == "Variant" {
			return format!("Variant of `{parent}`.");
		}
		if matches!(kind, "Function" | "Method") {
			return format!("{kind} defined under `{parent}`.");
		}
	}
	if !detail.is_empty() {
		return format!("{kind} with detail `{detail}`.");
	}
	format!("{kind} item.")
}

fn print_markdown(
	title: &str,
	paths: &[PathBuf],
	items: &[InventoryItem],
) {
	println!("# {}", markdown_escape(title));
	println!();
	println!("Generated from `rust-analyzer` LSP document symbols over:");
	for path in paths {
		println!("- `{}`", markdown_escape(&path.display().to_string()));
	}
	println!();
	println!("| File | Line | Kind | Item path | Detail | Description | Description source |");
	println!("| --- | ---: | --- | --- | --- | --- | --- |");
	for item in items {
		println!(
			"| `{}` | {} | {} | `{}` | {} | {} | {} |",
			markdown_escape(&item.file.display().to_string()),
			item.line,
			markdown_escape(&item.kind),
			markdown_escape(&item.item_path),
			markdown_escape(&compact(&item.detail, 160)),
			markdown_escape(&compact(&item.description, 220)),
			markdown_escape(&item.description_source),
		);
	}
}

fn print_json(
	paths: &[PathBuf],
	items: &[InventoryItem],
) -> Result<(), String> {
	let value = json!({
		"paths": paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>(),
		"items": items.iter().map(|item| json!({
			"file": item.file.display().to_string(),
			"line": item.line,
			"kind": item.kind,
			"item_path": item.item_path,
			"detail": item.detail,
			"description": item.description,
			"description_source": item.description_source,
		})).collect::<Vec<_>>()
	});
	let output = serde_json::to_string_pretty(&value)
		.map_err(|error| format!("json encode failed: {error}"))?;
	println!("{output}");
	Ok(())
}

fn compact(
	value: &str,
	limit: usize,
) -> String {
	let value = compact_whitespace(value);
	if value.len() <= limit {
		return value;
	}
	format!("{}...", value[.. limit.saturating_sub(3)].trim_end())
}

fn compact_whitespace(value: &str) -> String {
	value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn markdown_escape(value: &str) -> String {
	value.replace('\n', " ").replace('|', "\\|").replace('`', "\\`")
}

fn path_to_file_uri(path: &Path) -> Result<String, String> {
	let path = path
		.canonicalize()
		.unwrap_or_else(|_| path.to_path_buf())
		.to_string_lossy()
		.replace('\\', "/");
	if !path.starts_with('/') {
		return Err(format!("file URI requires an absolute path: {path}"));
	}
	Ok(format!("file://{}", percent_encode_path(&path)))
}

fn percent_encode_path(path: &str) -> String {
	let mut result = String::new();
	for byte in path.bytes() {
		match byte {
			b'A' ..= b'Z' | b'a' ..= b'z' | b'0' ..= b'9' | b'/' | b'-' | b'_' | b'.' | b'~' =>
				result.push(byte as char),
			_ => result.push_str(&format!("%{byte:02X}")),
		}
	}
	result
}
