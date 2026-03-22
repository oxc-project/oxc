use std::{
    env, fmt, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter, SortImportsOptions, get_parse_options};
use oxc_parser::Parser;
use oxc_tasks_common::TestFiles;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

const DEFAULT_REPEATS: usize = 5;
const DEFAULT_SAMPLE_MS: u64 = 20;
const SYNTHETIC_FILE_COUNTS: [usize; 9] = [1, 10, 25, 100, 200, 500, 1000, 10_000, 25_000];
const SYNTHETIC_FILE_SIZE_BYTES: usize = 2048;
const REAL_JSON_FAMILY_ROOT: &str = "fixtures/json_family";
const OXFMT_LABEL: &str = "oxfmt";
const BIOME_LABEL: &str = "biome";
const PRETTIER_LABEL: &str = "prettier";

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse(env::args().skip(1).collect())?;
    match cli.mode {
        Mode::Supervisor => run_supervisor(cli),
        Mode::OxfmtWorker { input_dir, output_dir } => {
            run_oxfmt_worker(&input_dir, &output_dir)?;
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct Cli {
    repeats: usize,
    sample_ms: u64,
    filter: Option<String>,
    output_json: Option<PathBuf>,
    node_path: Option<PathBuf>,
    synthetic_json_family: bool,
    json_family_real: bool,
    score_summary: bool,
    baseline_json: Option<PathBuf>,
    synthetic_formats: Option<Vec<SyntheticFormat>>,
    synthetic_counts: Option<Vec<usize>>,
    mode: Mode,
}

#[derive(Debug, Clone)]
enum Mode {
    Supervisor,
    OxfmtWorker { input_dir: PathBuf, output_dir: PathBuf },
}

impl Cli {
    fn parse(args: Vec<String>) -> Result<Self, CliError> {
        let mut repeats = DEFAULT_REPEATS;
        let mut sample_ms = DEFAULT_SAMPLE_MS;
        let mut filter = None;
        let mut output_json = None;
        let mut node_path = None;
        let mut synthetic_json_family = false;
        let mut json_family_real = false;
        let mut score_summary = false;
        let mut baseline_json = None;
        let mut synthetic_formats = None;
        let mut synthetic_counts = None;
        let mut mode = Mode::Supervisor;

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--repeats" => {
                    index += 1;
                    repeats = parse_usize_arg(args.get(index), "--repeats")?;
                }
                "--sample-ms" => {
                    index += 1;
                    sample_ms = parse_u64_arg(args.get(index), "--sample-ms")?;
                }
                "--filter" => {
                    index += 1;
                    filter = Some(parse_string_arg(args.get(index), "--filter")?);
                }
                "--output-json" => {
                    index += 1;
                    output_json =
                        Some(PathBuf::from(parse_string_arg(args.get(index), "--output-json")?));
                }
                "--node" => {
                    index += 1;
                    node_path = Some(PathBuf::from(parse_string_arg(args.get(index), "--node")?));
                }
                "--synthetic-json-family" => {
                    synthetic_json_family = true;
                }
                "--json-family-real" => {
                    json_family_real = true;
                }
                "--score-summary" => {
                    score_summary = true;
                }
                "--baseline-json" => {
                    index += 1;
                    baseline_json =
                        Some(PathBuf::from(parse_string_arg(args.get(index), "--baseline-json")?));
                }
                "--synthetic-formats" => {
                    index += 1;
                    synthetic_formats = Some(parse_synthetic_formats(args.get(index))?);
                }
                "--synthetic-counts" => {
                    index += 1;
                    synthetic_counts = Some(parse_synthetic_counts(args.get(index))?);
                }
                "--worker" => {
                    index += 1;
                    let worker = parse_string_arg(args.get(index), "--worker")?;
                    if worker != OXFMT_LABEL {
                        return Err(CliError(format!("unsupported worker mode: {worker}")));
                    }
                    index += 1;
                    let input_dir =
                        PathBuf::from(parse_string_arg(args.get(index), "--worker input dir")?);
                    index += 1;
                    let output_dir =
                        PathBuf::from(parse_string_arg(args.get(index), "--worker output dir")?);
                    mode = Mode::OxfmtWorker { input_dir, output_dir };
                }
                "--help" | "-h" => return Err(CliError(Self::usage())),
                arg => {
                    return Err(CliError(format!("unknown argument: {arg}\n\n{}", Self::usage())));
                }
            }
            index += 1;
        }

        if repeats == 0 {
            return Err(CliError("`--repeats` must be greater than zero".into()));
        }

        Ok(Self {
            repeats,
            sample_ms,
            filter,
            output_json,
            node_path,
            synthetic_json_family,
            json_family_real,
            score_summary,
            baseline_json,
            synthetic_formats,
            synthetic_counts,
            mode,
        })
    }

    fn usage() -> String {
        format!(
            "Usage:\n  formatter_resources [--repeats N] [--sample-ms N] [--filter SUBSTR] [--output-json PATH] [--node PATH] [--synthetic-json-family] [--json-family-real] [--score-summary] [--baseline-json PATH] [--synthetic-formats csv] [--synthetic-counts csv]\n  formatter_resources --worker {OXFMT_LABEL} <input_dir> <output_dir>"
        )
    }
}

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

fn parse_string_arg(arg: Option<&String>, flag: &str) -> Result<String, CliError> {
    arg.cloned().ok_or_else(|| CliError(format!("missing value for {flag}")))
}

fn parse_usize_arg(arg: Option<&String>, flag: &str) -> Result<usize, CliError> {
    parse_string_arg(arg, flag)?
        .parse()
        .map_err(|_| CliError(format!("invalid numeric value for {flag}")))
}

fn parse_u64_arg(arg: Option<&String>, flag: &str) -> Result<u64, CliError> {
    parse_string_arg(arg, flag)?
        .parse()
        .map_err(|_| CliError(format!("invalid numeric value for {flag}")))
}

fn parse_synthetic_formats(arg: Option<&String>) -> Result<Vec<SyntheticFormat>, CliError> {
    let value = parse_string_arg(arg, "--synthetic-formats")?;
    let mut formats = Vec::new();
    for item in value.split(',') {
        formats.push(match item.trim() {
            "json" => SyntheticFormat::Json,
            "jsonc" => SyntheticFormat::Jsonc,
            "json5" => SyntheticFormat::Json5,
            other => {
                return Err(CliError(format!(
                    "unsupported synthetic format `{other}`; expected json,jsonc,json5"
                )));
            }
        });
    }
    if formats.is_empty() {
        return Err(CliError("`--synthetic-formats` cannot be empty".into()));
    }
    Ok(formats)
}

fn parse_synthetic_counts(arg: Option<&String>) -> Result<Vec<usize>, CliError> {
    let value = parse_string_arg(arg, "--synthetic-counts")?;
    let mut counts = Vec::new();
    for item in value.split(',') {
        counts.push(
            item.trim()
                .parse::<usize>()
                .map_err(|_| CliError(format!("invalid synthetic count `{item}`")))?,
        );
    }
    if counts.is_empty() {
        return Err(CliError("`--synthetic-counts` cannot be empty".into()));
    }
    Ok(counts)
}

fn run_supervisor(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut corpora = Vec::new();
    if cli.synthetic_json_family {
        corpora.extend(materialize_synthetic_json_family(
            cli.synthetic_formats.as_deref(),
            cli.synthetic_counts.as_deref(),
        )?);
    }
    if cli.json_family_real {
        corpora.extend(materialize_real_json_family()?);
    }
    if corpora.is_empty() {
        corpora.push(materialize_formatter_corpus(cli.filter.as_deref())?);
    }
    let require_oxfmt_cli =
        corpora.iter().any(|corpus| matches!(corpus.oxfmt_mode, OxfmtMode::Cli));
    let tool_paths = ToolPaths::discover(cli.node_path, require_oxfmt_cli)?;

    let mut scenarios = Vec::with_capacity(corpora.len());
    for corpus in corpora {
        let mut results = Vec::new();

        for tool in [
            Tool::Oxfmt,
            Tool::Biome {
                node_path: tool_paths.node_path.clone(),
                runner_path: tool_paths.runner_path.clone(),
            },
            Tool::Prettier {
                node_path: tool_paths.node_path.clone(),
                runner_path: tool_paths.runner_path.clone(),
            },
        ] {
            let mut runs = Vec::with_capacity(cli.repeats);
            let note = unsupported_reason(&tool, &corpus).map(str::to_string);
            if note.is_none() {
                for _ in 0..cli.repeats {
                    runs.push(run_once(&tool, &corpus, &tool_paths, cli.sample_ms)?);
                }
            }
            results.push(ToolSummary {
                formatter: tool.label().to_string(),
                median: (!runs.is_empty()).then(|| MedianMetrics::from_runs(&runs)),
                runs,
                command: tool.command_display(),
                note,
            });
        }

        scenarios.push(BenchmarkSummary {
            scenario: corpus.scenario,
            repeats: cli.repeats,
            sample_interval_ms: cli.sample_ms,
            files: corpus.files.iter().map(|file| file.name.clone()).collect(),
            file_count: corpus.files.len(),
            results,
        });
    }

    let report = BenchmarkReport { scenarios };
    print!("{}", render_markdown_report(&report));
    if cli.score_summary {
        let baseline = cli.baseline_json.as_deref().map(load_report).transpose()?;
        print!("{}", render_score_summary(&report, baseline.as_ref()));
    }

    if let Some(path) = cli.output_json {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(&report)?)?;
    }

    Ok(())
}

fn load_report(path: &Path) -> Result<BenchmarkReport, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[derive(Debug)]
struct ToolPaths {
    node_path: PathBuf,
    runner_path: PathBuf,
    oxfmt_cli_path: Option<PathBuf>,
}

impl ToolPaths {
    fn discover(
        node_override: Option<PathBuf>,
        require_oxfmt_cli: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("formatter_tools");
        let runner_path = package_dir.join("runner.mjs");
        if !runner_path.is_file() {
            return Err(
                format!("missing formatter tools runner at {}", runner_path.display()).into()
            );
        }

        let node_path = node_override.unwrap_or_else(|| PathBuf::from("node"));
        let oxfmt_cli_path = if require_oxfmt_cli { Some(build_oxfmt_cli_binary()?) } else { None };
        Ok(Self { node_path, runner_path, oxfmt_cli_path })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkReport {
    scenarios: Vec<BenchmarkSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkSummary {
    scenario: String,
    repeats: usize,
    sample_interval_ms: u64,
    files: Vec<String>,
    file_count: usize,
    results: Vec<ToolSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolSummary {
    formatter: String,
    median: Option<MedianMetrics>,
    runs: Vec<RunMetrics>,
    command: String,
    note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunMetrics {
    wall_time_ms: f64,
    ops_per_second: f64,
    cpu_user_ms: f64,
    cpu_system_ms: f64,
    cpu_percent: f64,
    peak_rss_bytes: u64,
    disk_usage_bytes: u64,
    peak_fd_count: u64,
    io_read_bytes: Option<u64>,
    io_write_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MedianMetrics {
    wall_time_ms: f64,
    ops_per_second: f64,
    cpu_user_ms: f64,
    cpu_system_ms: f64,
    cpu_percent: f64,
    peak_rss_bytes: u64,
    disk_usage_bytes: u64,
    peak_fd_count: u64,
    io_read_bytes: Option<u64>,
    io_write_bytes: Option<u64>,
}

impl MedianMetrics {
    fn from_runs(runs: &[RunMetrics]) -> Self {
        Self {
            wall_time_ms: median_f64(runs.iter().map(|run| run.wall_time_ms).collect()),
            ops_per_second: median_f64(runs.iter().map(|run| run.ops_per_second).collect()),
            cpu_user_ms: median_f64(runs.iter().map(|run| run.cpu_user_ms).collect()),
            cpu_system_ms: median_f64(runs.iter().map(|run| run.cpu_system_ms).collect()),
            cpu_percent: median_f64(runs.iter().map(|run| run.cpu_percent).collect()),
            peak_rss_bytes: median_u64(runs.iter().map(|run| run.peak_rss_bytes).collect()),
            disk_usage_bytes: median_u64(runs.iter().map(|run| run.disk_usage_bytes).collect()),
            peak_fd_count: median_u64(runs.iter().map(|run| run.peak_fd_count).collect()),
            io_read_bytes: median_option_u64(runs.iter().map(|run| run.io_read_bytes).collect()),
            io_write_bytes: median_option_u64(runs.iter().map(|run| run.io_write_bytes).collect()),
        }
    }
}

#[derive(Debug)]
enum Tool {
    Oxfmt,
    Biome { node_path: PathBuf, runner_path: PathBuf },
    Prettier { node_path: PathBuf, runner_path: PathBuf },
}

impl Tool {
    fn label(&self) -> &'static str {
        match self {
            Self::Oxfmt => OXFMT_LABEL,
            Self::Biome { .. } => BIOME_LABEL,
            Self::Prettier { .. } => PRETTIER_LABEL,
        }
    }

    fn command_display(&self) -> String {
        match self {
            Self::Oxfmt => "formatter_resources --worker oxfmt <input_dir> <output_dir>".into(),
            Self::Biome { node_path, runner_path } => format!(
                "{} {} biome <input_dir> <output_dir>",
                node_path.display(),
                runner_path.display()
            ),
            Self::Prettier { node_path, runner_path } => format!(
                "{} {} prettier <input_dir> <output_dir>",
                node_path.display(),
                runner_path.display()
            ),
        }
    }

    fn spawn(
        &self,
        corpus: &MaterializedCorpus,
        tool_paths: &ToolPaths,
        output_dir: &Path,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        match self {
            Self::Oxfmt => {
                let mut command = match corpus.oxfmt_mode {
                    OxfmtMode::CoreWorker => {
                        let exe = env::current_exe()?;
                        let mut command = Command::new(exe);
                        command
                            .arg("--worker")
                            .arg(OXFMT_LABEL)
                            .arg(&corpus.input_dir)
                            .arg(output_dir);
                        command
                    }
                    OxfmtMode::Cli => {
                        let Some(oxfmt_cli_path) = &tool_paths.oxfmt_cli_path else {
                            return Err("missing oxfmt CLI binary path".into());
                        };
                        copy_dir_recursive(&corpus.input_dir, output_dir)?;
                        let mut command = Command::new(oxfmt_cli_path);
                        command.arg("--write").arg(output_dir);
                        command
                    }
                };
                command.stderr(Stdio::piped()).stdout(Stdio::null());
                Ok(command.spawn()?)
            }
            Self::Biome { node_path, runner_path } => spawn_node_runner(
                node_path,
                runner_path,
                BIOME_LABEL,
                &corpus.input_dir,
                output_dir,
            ),
            Self::Prettier { node_path, runner_path } => spawn_node_runner(
                node_path,
                runner_path,
                PRETTIER_LABEL,
                &corpus.input_dir,
                output_dir,
            ),
        }
    }
}

fn spawn_node_runner(
    node_path: &Path,
    runner_path: &Path,
    label: &str,
    input_dir: &Path,
    output_dir: &Path,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mut command = Command::new(node_path);
    command.arg(runner_path).arg(label).arg(input_dir).arg(output_dir);
    command.stderr(Stdio::piped()).stdout(Stdio::null());
    Ok(command.spawn()?)
}

#[derive(Debug)]
struct MaterializedCorpus {
    _temp_dir: TempDir,
    input_dir: PathBuf,
    scenario: String,
    oxfmt_mode: OxfmtMode,
    files: Vec<CorpusFile>,
}

#[derive(Debug)]
struct CorpusFile {
    name: String,
}

#[derive(Debug, Clone, Copy)]
enum OxfmtMode {
    CoreWorker,
    Cli,
}

fn materialize_formatter_corpus(
    filter: Option<&str>,
) -> Result<MaterializedCorpus, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir)?;

    let mut files = Vec::new();
    for file in TestFiles::formatter().files() {
        if let Some(filter) = filter
            && !file.file_name.contains(filter)
        {
            continue;
        }
        let path = input_dir.join(&file.file_name);
        fs::write(&path, &file.source_text)?;
        files.push(CorpusFile { name: file.file_name.clone() });
    }

    if files.is_empty() {
        return Err("formatter benchmark corpus is empty after filtering".into());
    }

    Ok(MaterializedCorpus {
        _temp_dir: temp_dir,
        input_dir,
        scenario: "formatter-corpus".into(),
        oxfmt_mode: OxfmtMode::CoreWorker,
        files,
    })
}

fn materialize_synthetic_json_family(
    formats: Option<&[SyntheticFormat]>,
    counts: Option<&[usize]>,
) -> Result<Vec<MaterializedCorpus>, Box<dyn std::error::Error>> {
    let mut corpora = Vec::new();
    let default_formats = [SyntheticFormat::Json, SyntheticFormat::Jsonc, SyntheticFormat::Json5];
    let default_counts = SYNTHETIC_FILE_COUNTS;
    for &format in formats.unwrap_or(&default_formats) {
        for &file_count in counts.unwrap_or(&default_counts) {
            corpora.push(materialize_synthetic_corpus(format, file_count)?);
        }
    }
    Ok(corpora)
}

fn materialize_synthetic_corpus(
    format: SyntheticFormat,
    file_count: usize,
) -> Result<MaterializedCorpus, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir)?;

    let mut files = Vec::with_capacity(file_count);
    for index in 0..file_count {
        let file_name = format.file_name(index);
        let path = input_dir.join(&file_name);
        let source_text = build_synthetic_source(format, index, SYNTHETIC_FILE_SIZE_BYTES);
        fs::write(&path, source_text)?;
        files.push(CorpusFile { name: file_name });
    }

    Ok(MaterializedCorpus {
        _temp_dir: temp_dir,
        input_dir,
        scenario: format!("synthetic-{}-{file_count}", format.label()),
        oxfmt_mode: OxfmtMode::Cli,
        files,
    })
}

fn materialize_real_json_family() -> Result<Vec<MaterializedCorpus>, Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REAL_JSON_FAMILY_ROOT);
    let scenarios = [
        ("real-json", SyntheticFormat::Json),
        ("real-jsonc", SyntheticFormat::Jsonc),
        ("real-json5", SyntheticFormat::Json5),
    ];

    let mut corpora = Vec::with_capacity(scenarios.len());
    for (scenario, format) in scenarios {
        let source_dir = root.join(format.label());
        if !source_dir.is_dir() {
            return Err(
                format!("missing benchmark fixture directory: {}", source_dir.display()).into()
            );
        }

        let temp_dir = tempfile::tempdir()?;
        let input_dir = temp_dir.path().join("input");
        copy_dir_recursive(&source_dir, &input_dir)?;
        let files = collect_fixture_files(&input_dir)?;
        if files.is_empty() {
            return Err(
                format!("benchmark fixture directory is empty: {}", source_dir.display()).into()
            );
        }

        corpora.push(MaterializedCorpus {
            _temp_dir: temp_dir,
            input_dir,
            scenario: scenario.into(),
            oxfmt_mode: OxfmtMode::Cli,
            files,
        });
    }

    Ok(corpora)
}

fn collect_fixture_files(root: &Path) -> Result<Vec<CorpusFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_fixture_files_impl(root, root, &mut files)?;
    files.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    Ok(files)
}

fn collect_fixture_files_impl(
    root: &Path,
    directory: &Path,
    files: &mut Vec<CorpusFile>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_fixture_files_impl(root, &path, files)?;
        } else if file_type.is_file() {
            let name = path
                .strip_prefix(root)
                .map_err(|_| "failed to strip benchmark root prefix")?
                .to_string_lossy()
                .replace('\\', "/");
            files.push(CorpusFile { name });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum SyntheticFormat {
    Json,
    Jsonc,
    Json5,
}

impl SyntheticFormat {
    fn label(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Jsonc => "jsonc",
            Self::Json5 => "json5",
        }
    }

    fn extension(self) -> &'static str {
        self.label()
    }

    fn file_name(self, index: usize) -> String {
        format!("fixture-{index:05}.{}", self.extension())
    }
}

fn build_synthetic_source(format: SyntheticFormat, seed: usize, target_size: usize) -> String {
    let mut tags = String::new();
    let mut matrix = String::new();
    let mut history = String::new();
    for offset in 0..8 {
        let value = seed + offset;
        tags.push_str(&format!("{},", string_literal(format, &format!("tag-{value:05}"))));
        matrix.push_str(&format!("{},", value * 3 + 1));
        history.push_str(&format!(
            "{key}: {value},",
            key = object_key(format, &format!("rev_{offset}")),
            value = string_literal(format, &format!("2026-03-{day:02}", day = (value % 27) + 1)),
        ));
    }
    let strict_tags = tags.trim_end_matches(',');
    let strict_matrix = matrix.trim_end_matches(',');
    let strict_history = history.trim_end_matches(',');

    let payload = repeated_payload(seed, target_size / 2);
    let mut body = match format {
        SyntheticFormat::Json => format!(
            concat!(
                "{{\n",
                "  \"name\": {name},\n",
                "  \"version\": {version},\n",
                "  \"enabled\": true,\n",
                "  \"priority\": {priority},\n",
                "  \"paths\": [\"src\", \"dist\", \"fixtures\", \"benchmarks\"],\n",
                "  \"tags\": [{tags}],\n",
                "  \"matrix\": [{matrix}],\n",
                "  \"metadata\": {{\n",
                "    \"seed\": {seed},\n",
                "    \"checksum\": {checksum},\n",
                "    \"maintainer\": \"benchmark-runner\",\n",
                "    \"history\": {{{history}}}\n",
                "  }},\n",
                "  \"payload\": \"{payload}\"\n",
                "}}\n"
            ),
            name = string_literal(format, &format!("fixture-{seed:05}")),
            version = string_literal(format, &format!("1.{}.{}", seed % 10, seed % 17)),
            priority = seed % 9,
            tags = strict_tags,
            matrix = strict_matrix,
            seed = seed,
            checksum = seed * 97 + 13,
            history = strict_history,
            payload = payload,
        ),
        SyntheticFormat::Jsonc => format!(
            concat!(
                "{{\n",
                "  // synthetic jsonc fixture\n",
                "  \"name\": {name},\n",
                "  \"version\": {version},\n",
                "  \"enabled\": true,\n",
                "  \"priority\": {priority},\n",
                "  \"tags\": [{tags}],\n",
                "  \"matrix\": [{matrix}],\n",
                "  /* metadata block */\n",
                "  \"metadata\": {{\n",
                "    \"seed\": {seed},\n",
                "    \"checksum\": {checksum},\n",
                "    \"history\": {{{history}}},\n",
                "  }},\n",
                "  \"payload\": \"{payload}\",\n",
                "}}\n"
            ),
            name = string_literal(format, &format!("fixture-{seed:05}")),
            version = string_literal(format, &format!("1.{}.{}", seed % 10, seed % 17)),
            priority = seed % 9,
            tags = tags,
            matrix = matrix,
            seed = seed,
            checksum = seed * 97 + 13,
            history = history,
            payload = payload,
        ),
        SyntheticFormat::Json5 => format!(
            concat!(
                "{{\n",
                "  // synthetic json5 fixture\n",
                "  name: {name},\n",
                "  version: {version},\n",
                "  enabled: true,\n",
                "  priority: {priority},\n",
                "  hex: 0x{hex:04x},\n",
                "  tags: [{tags}],\n",
                "  matrix: [{matrix}],\n",
                "  metadata: {{\n",
                "    seed: {seed},\n",
                "    checksum: {checksum},\n",
                "    history: {{{history}}},\n",
                "  }},\n",
                "  payload: {payload},\n",
                "}}\n"
            ),
            name = string_literal(format, &format!("fixture-{seed:05}")),
            version = string_literal(format, &format!("1.{}.{}", seed % 10, seed % 17)),
            priority = seed % 9,
            hex = seed * 17 + 31,
            tags = tags,
            matrix = matrix,
            seed = seed,
            checksum = seed * 97 + 13,
            history = history,
            payload = string_literal(format, &payload),
        ),
    };

    while body.len() < target_size {
        let padding = repeated_payload(seed.wrapping_add(body.len()), 48);
        match format {
            SyntheticFormat::Json => {
                let extra = format!("\n  ,\"extra_{:04}\": \"{}\"", body.len(), padding);
                if let Some(idx) = body.rfind("\n}\n") {
                    body.insert_str(idx, &extra);
                } else {
                    break;
                }
            }
            SyntheticFormat::Jsonc => {
                let extra = format!(
                    "\n  // extra {}\n  \"extra_{:04}\": \"{}\",",
                    body.len(),
                    body.len(),
                    padding
                );
                if let Some(idx) = body.rfind("\n}\n") {
                    body.insert_str(idx, &extra);
                } else {
                    break;
                }
            }
            SyntheticFormat::Json5 => {
                let extra =
                    format!("\n  extra_{:04}: {},", body.len(), string_literal(format, &padding));
                if let Some(idx) = body.rfind("\n}\n") {
                    body.insert_str(idx, &extra);
                } else {
                    break;
                }
            }
        }
    }

    body
}

fn repeated_payload(seed: usize, target_len: usize) -> String {
    let mut payload = String::new();
    let mut cursor = seed;
    while payload.len() < target_len {
        payload.push_str(&format!("block-{cursor:05}-payload-"));
        cursor = cursor.wrapping_mul(31).wrapping_add(7) % 99_991;
    }
    payload.truncate(target_len);
    payload
}

fn object_key(format: SyntheticFormat, value: &str) -> String {
    match format {
        SyntheticFormat::Json | SyntheticFormat::Jsonc => format!("\"{value}\""),
        SyntheticFormat::Json5 => value.into(),
    }
}

fn string_literal(format: SyntheticFormat, value: &str) -> String {
    match format {
        SyntheticFormat::Json | SyntheticFormat::Jsonc => format!("\"{value}\""),
        SyntheticFormat::Json5 => format!("'{value}'"),
    }
}

fn run_oxfmt_worker(input_dir: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;
    let format_options =
        FormatOptions { sort_imports: Some(SortImportsOptions::default()), ..Default::default() };

    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let input_path = entry.path();
        let output_path = output_dir.join(entry.file_name());
        let source_text = fs::read_to_string(&input_path)?;
        let source_type = oxc_span::SourceType::from_path(&input_path)
            .map_err(|_| format!("unsupported file type: {}", input_path.display()))?;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source_text, source_type)
            .with_options(get_parse_options())
            .parse();
        if !parsed.errors.is_empty() {
            return Err(format!(
                "parse failed for {} with {} errors",
                input_path.display(),
                parsed.errors.len()
            )
            .into());
        }
        let output = Formatter::new(&allocator, format_options.clone()).build(&parsed.program);
        fs::write(output_path, output)?;
    }

    Ok(())
}

fn run_once(
    tool: &Tool,
    corpus: &MaterializedCorpus,
    tool_paths: &ToolPaths,
    sample_ms: u64,
) -> Result<RunMetrics, Box<dyn std::error::Error>> {
    let run_dir = tempfile::tempdir()?;
    let output_dir = run_dir.path().join(tool.label());
    fs::create_dir_all(&output_dir)?;

    let mut child = tool.spawn(corpus, tool_paths, &output_dir)?;
    let child_stderr = child.stderr.take();
    let pid = child.id() as i32;
    let collector = MetricsCollector::new(pid);
    let cpu_before = ChildRusage::snapshot()?;
    let started_at = Instant::now();

    let mut peak_rss_bytes = 0;
    let mut peak_fd_count = 0;
    let mut io_read_bytes = None;
    let mut io_write_bytes = None;
    let exit_status = loop {
        if let Some(sample) = collector.sample() {
            peak_rss_bytes = peak_rss_bytes.max(sample.rss_bytes);
            peak_fd_count = peak_fd_count.max(sample.fd_count);
            io_read_bytes = sample.read_bytes.or(io_read_bytes);
            io_write_bytes = sample.write_bytes.or(io_write_bytes);
        }

        if let Some(status) = child.try_wait()? {
            break status;
        }

        thread::sleep(Duration::from_millis(sample_ms));
    };

    let elapsed = started_at.elapsed();
    let cpu_after = ChildRusage::snapshot()?;
    let cpu_delta = cpu_after.saturating_sub(cpu_before);
    if !exit_status.success() {
        let stderr = read_child_stderr(child_stderr)?;
        return Err(format!("{} runner failed with {exit_status}: {stderr}", tool.label()).into());
    }

    let disk_usage_bytes = total_file_size(&output_dir)?;
    let cpu_total_ms = cpu_delta.total_micros as f64 / 1000.0;
    let wall_time_ms = elapsed.as_secs_f64() * 1000.0;
    let ops_per_second =
        if wall_time_ms == 0.0 { 0.0 } else { (corpus.files.len() as f64 * 1000.0) / wall_time_ms };
    let cpu_percent = if wall_time_ms == 0.0 { 0.0 } else { (cpu_total_ms / wall_time_ms) * 100.0 };

    Ok(RunMetrics {
        wall_time_ms,
        ops_per_second,
        cpu_user_ms: cpu_delta.user_micros as f64 / 1000.0,
        cpu_system_ms: cpu_delta.system_micros as f64 / 1000.0,
        cpu_percent,
        peak_rss_bytes,
        disk_usage_bytes,
        peak_fd_count,
        io_read_bytes,
        io_write_bytes,
    })
}

fn unsupported_reason(tool: &Tool, corpus: &MaterializedCorpus) -> Option<&'static str> {
    if matches!(tool, Tool::Biome { .. })
        && (corpus.scenario.starts_with("synthetic-json5-") || corpus.scenario == "real-json5")
    {
        return Some("unsupported");
    }
    None
}

fn read_child_stderr(stderr: Option<impl Read>) -> Result<String, Box<dyn std::error::Error>> {
    let Some(mut stderr) = stderr else {
        return Ok(String::new());
    };
    let mut output = String::new();
    stderr.read_to_string(&mut output)?;
    Ok(output.trim().to_string())
}

fn total_file_size(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += total_file_size(&entry.path())?;
        }
    }
    Ok(total)
}

fn copy_dir_recursive(
    source_dir: &Path,
    destination_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination_dir)?;
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = destination_dir.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn build_oxfmt_cli_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("failed to resolve workspace root")?
        .to_path_buf();
    let target_dir = workspace_root.join("target").join("formatter_resources_tools");
    let binary_name = if cfg!(windows) { "oxfmt.exe" } else { "oxfmt" };
    let binary_path = target_dir.join("debug").join(binary_name);
    if binary_path.is_file() {
        return Ok(binary_path);
    }

    let status = Command::new("cargo")
        .current_dir(&workspace_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .arg("build")
        .arg("-p")
        .arg("oxfmt")
        .arg("--bin")
        .arg("oxfmt")
        .arg("--no-default-features")
        .status()?;
    if !status.success() {
        return Err("failed to build Rust-only oxfmt CLI".into());
    }

    Ok(binary_path)
}

#[derive(Debug, Clone, Copy, Default)]
struct ProcessSample {
    rss_bytes: u64,
    fd_count: u64,
    read_bytes: Option<u64>,
    write_bytes: Option<u64>,
}

struct MetricsCollector {
    pid: i32,
}

impl MetricsCollector {
    fn new(pid: i32) -> Self {
        Self { pid }
    }

    fn sample(&self) -> Option<ProcessSample> {
        platform::sample_process(self.pid)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ChildRusage {
    user_micros: u64,
    system_micros: u64,
    total_micros: u64,
}

impl ChildRusage {
    fn snapshot() -> Result<Self, Box<dyn std::error::Error>> {
        let mut usage = libc::rusage {
            ru_utime: libc::timeval { tv_sec: 0, tv_usec: 0 },
            ru_stime: libc::timeval { tv_sec: 0, tv_usec: 0 },
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        };
        let result = unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let user_micros = timeval_to_micros(usage.ru_utime);
        let system_micros = timeval_to_micros(usage.ru_stime);
        Ok(Self { user_micros, system_micros, total_micros: user_micros + system_micros })
    }

    fn saturating_sub(self, rhs: Self) -> Self {
        let user_micros = self.user_micros.saturating_sub(rhs.user_micros);
        let system_micros = self.system_micros.saturating_sub(rhs.system_micros);
        Self { user_micros, system_micros, total_micros: user_micros + system_micros }
    }
}

fn timeval_to_micros(value: libc::timeval) -> u64 {
    let secs = u64::try_from(value.tv_sec).unwrap_or_default();
    let micros = u64::try_from(value.tv_usec).unwrap_or_default();
    secs.saturating_mul(1_000_000).saturating_add(micros)
}

#[cfg(target_os = "linux")]
mod platform {
    use super::ProcessSample;
    use std::{fs, path::PathBuf};

    pub fn sample_process(pid: i32) -> Option<ProcessSample> {
        let root = PathBuf::from("/proc").join(pid.to_string());
        let status = fs::read_to_string(root.join("status")).ok()?;
        let rss_bytes = status
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:"))
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|value| value * 1024)
            .unwrap_or_default();
        let fd_count = fs::read_dir(root.join("fd")).ok()?.count() as u64;
        let io = fs::read_to_string(root.join("io")).ok();
        let read_bytes =
            io.as_deref().and_then(|content| parse_proc_io_field(content, "read_bytes:"));
        let write_bytes =
            io.as_deref().and_then(|content| parse_proc_io_field(content, "write_bytes:"));

        Some(ProcessSample { rss_bytes, fd_count, read_bytes, write_bytes })
    }

    fn parse_proc_io_field(content: &str, name: &str) -> Option<u64> {
        content
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| value.trim().parse::<u64>().ok())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::ProcessSample;
    use std::mem;

    const PROC_PIDLISTFDS: i32 = 1;
    const RUSAGE_INFO_V4: i32 = 4;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ProcFdInfo {
        proc_fd: i32,
        proc_fdtype: u32,
    }

    #[repr(C)]
    struct RusageInfoV4 {
        ri_uuid: [u8; 16],
        ri_user_time: u64,
        ri_system_time: u64,
        ri_pkg_idle_wkups: u64,
        ri_interrupt_wkups: u64,
        ri_pageins: u64,
        ri_wired_size: u64,
        ri_resident_size: u64,
        ri_phys_footprint: u64,
        ri_proc_start_abstime: u64,
        ri_proc_exit_abstime: u64,
        ri_child_user_time: u64,
        ri_child_system_time: u64,
        ri_child_pkg_idle_wkups: u64,
        ri_child_interrupt_wkups: u64,
        ri_child_pageins: u64,
        ri_child_elapsed_abstime: u64,
        ri_diskio_bytesread: u64,
        ri_diskio_byteswritten: u64,
        ri_cpu_time_qos_default: u64,
        ri_cpu_time_qos_maintenance: u64,
        ri_cpu_time_qos_background: u64,
        ri_cpu_time_qos_utility: u64,
        ri_cpu_time_qos_legacy: u64,
        ri_cpu_time_qos_user_initiated: u64,
        ri_cpu_time_qos_user_interactive: u64,
        ri_billed_system_time: u64,
        ri_serviced_system_time: u64,
        ri_logical_writes: u64,
        ri_lifetime_max_phys_footprint: u64,
        ri_instructions: u64,
        ri_cycles: u64,
        ri_billed_energy: u64,
        ri_serviced_energy: u64,
        ri_interval_max_phys_footprint: u64,
    }

    unsafe extern "C" {
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut libc::c_void,
            buffersize: i32,
        ) -> i32;
        fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut libc::c_void) -> i32;
    }

    pub fn sample_process(pid: i32) -> Option<ProcessSample> {
        let mut usage = RusageInfoV4 {
            ri_uuid: [0; 16],
            ri_user_time: 0,
            ri_system_time: 0,
            ri_pkg_idle_wkups: 0,
            ri_interrupt_wkups: 0,
            ri_pageins: 0,
            ri_wired_size: 0,
            ri_resident_size: 0,
            ri_phys_footprint: 0,
            ri_proc_start_abstime: 0,
            ri_proc_exit_abstime: 0,
            ri_child_user_time: 0,
            ri_child_system_time: 0,
            ri_child_pkg_idle_wkups: 0,
            ri_child_interrupt_wkups: 0,
            ri_child_pageins: 0,
            ri_child_elapsed_abstime: 0,
            ri_diskio_bytesread: 0,
            ri_diskio_byteswritten: 0,
            ri_cpu_time_qos_default: 0,
            ri_cpu_time_qos_maintenance: 0,
            ri_cpu_time_qos_background: 0,
            ri_cpu_time_qos_utility: 0,
            ri_cpu_time_qos_legacy: 0,
            ri_cpu_time_qos_user_initiated: 0,
            ri_cpu_time_qos_user_interactive: 0,
            ri_billed_system_time: 0,
            ri_serviced_system_time: 0,
            ri_logical_writes: 0,
            ri_lifetime_max_phys_footprint: 0,
            ri_instructions: 0,
            ri_cycles: 0,
            ri_billed_energy: 0,
            ri_serviced_energy: 0,
            ri_interval_max_phys_footprint: 0,
        };
        let usage_result = unsafe {
            proc_pid_rusage(pid, RUSAGE_INFO_V4, (&mut usage as *mut RusageInfoV4).cast())
        };
        if usage_result != 0 {
            return None;
        }

        let mut fds = vec![ProcFdInfo { proc_fd: 0, proc_fdtype: 0 }; 64];
        let buffer_len = i32::try_from(fds.len() * mem::size_of::<ProcFdInfo>()).ok()?;
        let fd_bytes =
            unsafe { proc_pidinfo(pid, PROC_PIDLISTFDS, 0, fds.as_mut_ptr().cast(), buffer_len) };
        if fd_bytes < 0 {
            return None;
        }
        let fd_count =
            u64::try_from(fd_bytes).ok()? / u64::try_from(mem::size_of::<ProcFdInfo>()).ok()?;
        let rss_bytes = usage
            .ri_lifetime_max_phys_footprint
            .max(usage.ri_interval_max_phys_footprint)
            .max(usage.ri_phys_footprint.max(usage.ri_resident_size));

        Some(ProcessSample {
            rss_bytes,
            fd_count,
            read_bytes: Some(usage.ri_diskio_bytesread),
            write_bytes: Some(usage.ri_diskio_byteswritten),
        })
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    use super::ProcessSample;

    pub fn sample_process(_pid: i32) -> Option<ProcessSample> {
        None
    }
}

fn render_markdown_report(report: &BenchmarkReport) -> String {
    let mut output = String::new();
    output.push_str(
        "| Dataset | Files | Formatter | OPS/sec | CPU usage | Memory usage | Disk usage | File descriptor | IO Access |\n",
    );
    output.push_str("| --- | ---: | --- | ---: | --- | --- | --- | --- | --- |\n");

    for summary in &report.scenarios {
        for result in &summary.results {
            let ops_per_second =
                result.median.as_ref().map_or_else(|| "n/a".into(), format_ops_per_second);
            let cpu_usage = result.median.as_ref().map_or_else(
                || result.note.clone().unwrap_or_else(|| "n/a".into()),
                format_cpu_usage,
            );
            let memory_usage = result
                .median
                .as_ref()
                .map_or_else(|| "n/a".into(), |metrics| format_bytes(metrics.peak_rss_bytes));
            let disk_usage = result
                .median
                .as_ref()
                .map_or_else(|| "n/a".into(), |metrics| format_bytes(metrics.disk_usage_bytes));
            let file_descriptors = result
                .median
                .as_ref()
                .map_or_else(|| "n/a".into(), |metrics| metrics.peak_fd_count.to_string());
            let io_access = result.median.as_ref().map_or_else(
                || "n/a".into(),
                |metrics| format_io_access(metrics.io_read_bytes, metrics.io_write_bytes),
            );
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                summary.scenario,
                summary.file_count,
                result.formatter,
                ops_per_second,
                cpu_usage,
                memory_usage,
                disk_usage,
                file_descriptors,
                io_access,
            ));
        }
    }

    output
}

fn render_score_summary(report: &BenchmarkReport, baseline: Option<&BenchmarkReport>) -> String {
    let current = compute_score(report);
    let mut output = String::new();
    output.push_str("\n## Score Summary\n\n");
    output.push_str("| Formatter | Scenarios | Geomean OPS | Max RSS |\n");
    output.push_str("| --- | ---: | ---: | --- |\n");
    for formatter in [OXFMT_LABEL, BIOME_LABEL, PRETTIER_LABEL] {
        if let Some(score) = current.iter().find(|score| score.formatter == formatter) {
            output.push_str(&format!(
                "| {} | {} | {:.3} | {} |\n",
                score.formatter,
                score.scenarios,
                score.geomean_ops,
                format_bytes(score.max_rss_bytes),
            ));
        }
    }

    if let Some(baseline) = baseline {
        let baseline_scores = compute_score(baseline);
        output.push_str(
            "\n| Formatter | OPS Ratio | RSS Ratio | Throughput Guardrail | RSS Guardrail |\n",
        );
        output.push_str("| --- | ---: | ---: | --- | --- |\n");
        for formatter in [OXFMT_LABEL, BIOME_LABEL, PRETTIER_LABEL] {
            let Some(current_score) = current.iter().find(|score| score.formatter == formatter)
            else {
                continue;
            };
            let Some(baseline_score) =
                baseline_scores.iter().find(|score| score.formatter == formatter)
            else {
                continue;
            };
            let ops_ratio = current_score.geomean_ops / baseline_score.geomean_ops;
            let rss_ratio =
                current_score.max_rss_bytes as f64 / baseline_score.max_rss_bytes as f64;
            let throughput_guardrail = current_score.min_ops_ratio_vs(baseline_score);
            let rss_guardrail = current_score.max_rss_ratio_vs(baseline_score);
            output.push_str(&format!(
                "| {} | {:.3}x | {:.3}x | {} | {} |\n",
                formatter,
                ops_ratio,
                rss_ratio,
                pass_fail(throughput_guardrail >= 0.95),
                pass_fail(rss_guardrail <= 1.10),
            ));
        }
    }

    output
}

#[derive(Debug)]
struct FormatterScore {
    formatter: String,
    geomean_ops: f64,
    max_rss_bytes: u64,
    scenarios: usize,
    scenario_metrics: Vec<ScenarioMetric>,
}

#[derive(Debug)]
struct ScenarioMetric {
    scenario: String,
    ops_per_second: f64,
    peak_rss_bytes: u64,
}

impl FormatterScore {
    fn min_ops_ratio_vs(&self, baseline: &Self) -> f64 {
        self.scenario_metrics
            .iter()
            .filter_map(|current| {
                baseline
                    .scenario_metrics
                    .iter()
                    .find(|baseline| baseline.scenario == current.scenario)
                    .map(|baseline| current.ops_per_second / baseline.ops_per_second)
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn max_rss_ratio_vs(&self, baseline: &Self) -> f64 {
        self.scenario_metrics
            .iter()
            .filter_map(|current| {
                baseline
                    .scenario_metrics
                    .iter()
                    .find(|baseline| baseline.scenario == current.scenario)
                    .map(|baseline| current.peak_rss_bytes as f64 / baseline.peak_rss_bytes as f64)
            })
            .fold(0.0, f64::max)
    }
}

fn compute_score(report: &BenchmarkReport) -> Vec<FormatterScore> {
    let mut scores = Vec::new();
    for formatter in [OXFMT_LABEL, BIOME_LABEL, PRETTIER_LABEL] {
        let scenario_metrics: Vec<ScenarioMetric> = report
            .scenarios
            .iter()
            .filter_map(|summary| {
                summary.results.iter().find(|result| result.formatter == formatter).and_then(
                    |result| {
                        result.median.as_ref().map(|median| ScenarioMetric {
                            scenario: summary.scenario.clone(),
                            ops_per_second: median.ops_per_second,
                            peak_rss_bytes: median.peak_rss_bytes,
                        })
                    },
                )
            })
            .collect();
        if scenario_metrics.is_empty() {
            continue;
        }

        let geomean_ops =
            geometric_mean(scenario_metrics.iter().map(|metric| metric.ops_per_second));
        let max_rss_bytes =
            scenario_metrics.iter().map(|metric| metric.peak_rss_bytes).max().unwrap_or_default();
        scores.push(FormatterScore {
            formatter: formatter.into(),
            geomean_ops,
            max_rss_bytes,
            scenarios: scenario_metrics.len(),
            scenario_metrics,
        });
    }
    scores
}

fn geometric_mean(values: impl Iterator<Item = f64>) -> f64 {
    let values: Vec<f64> = values.collect();
    let sum_logs = values.iter().map(|value| value.ln()).sum::<f64>();
    (sum_logs / values.len() as f64).exp()
}

fn pass_fail(condition: bool) -> &'static str {
    if condition { "PASS" } else { "FAIL" }
}

#[cfg(test)]
fn render_markdown_table(summary: &BenchmarkSummary) -> String {
    render_markdown_report(&BenchmarkReport { scenarios: vec![summary.clone()] })
}

fn format_cpu_usage(metrics: &MedianMetrics) -> String {
    format!(
        "{:.1}% avg ({:.1}ms user + {:.1}ms sys)",
        metrics.cpu_percent, metrics.cpu_user_ms, metrics.cpu_system_ms
    )
}

fn format_ops_per_second(metrics: &MedianMetrics) -> String {
    format!("{:.1}", metrics.ops_per_second)
}

fn format_io_access(read_bytes: Option<u64>, write_bytes: Option<u64>) -> String {
    match (read_bytes, write_bytes) {
        (Some(read_bytes), Some(write_bytes)) => {
            format!("{} read / {} write", format_bytes(read_bytes), format_bytes(write_bytes))
        }
        _ => "n/a".into(),
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0_usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

fn median_f64(mut values: Vec<f64>) -> f64 {
    values.sort_by(|lhs, rhs| lhs.total_cmp(rhs));
    values[values.len() / 2]
}

fn median_u64(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn median_option_u64(values: Vec<Option<u64>>) -> Option<u64> {
    let mut present: Vec<u64> = values.into_iter().flatten().collect();
    if present.is_empty() {
        return None;
    }
    present.sort_unstable();
    Some(present[present.len() / 2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    #[test]
    fn parses_supervisor_args() {
        let cli = Cli::parse(vec![
            "--repeats".into(),
            "7".into(),
            "--sample-ms".into(),
            "11".into(),
            "--filter".into(),
            "tsx".into(),
            "--output-json".into(),
            "out.json".into(),
            "--synthetic-json-family".into(),
        ])
        .unwrap();

        assert!(matches!(cli.mode, Mode::Supervisor));
        assert_eq!(cli.repeats, 7);
        assert_eq!(cli.sample_ms, 11);
        assert_eq!(cli.filter.as_deref(), Some("tsx"));
        assert_eq!(cli.output_json.as_deref(), Some(Path::new("out.json")));
        assert!(cli.synthetic_json_family);
    }

    #[test]
    fn renders_markdown_table() {
        let summary = BenchmarkSummary {
            scenario: "formatter-corpus".into(),
            repeats: 3,
            sample_interval_ms: 20,
            files: vec!["example.ts".into()],
            file_count: 1,
            results: vec![ToolSummary {
                formatter: "oxfmt".into(),
                median: Some(MedianMetrics {
                    wall_time_ms: 10.0,
                    ops_per_second: 100.0,
                    cpu_user_ms: 7.0,
                    cpu_system_ms: 2.0,
                    cpu_percent: 90.0,
                    peak_rss_bytes: 1_048_576,
                    disk_usage_bytes: 512,
                    peak_fd_count: 4,
                    io_read_bytes: Some(1024),
                    io_write_bytes: Some(2048),
                }),
                runs: vec![],
                command: "cmd".into(),
                note: None,
            }],
        };

        let rendered = render_markdown_table(&summary);
        assert!(rendered.contains(
            "| formatter-corpus | 1 | oxfmt | 100.0 | 90.0% avg (7.0ms user + 2.0ms sys) |"
        ));
        assert!(rendered.contains("1.00 MB"));
        assert!(rendered.contains("1.00 KB read / 2.00 KB write"));
    }

    #[test]
    fn renders_score_summary() {
        let report = BenchmarkReport {
            scenarios: vec![
                BenchmarkSummary {
                    scenario: "real-json".into(),
                    repeats: 3,
                    sample_interval_ms: 20,
                    files: vec!["package.json".into()],
                    file_count: 1,
                    results: vec![ToolSummary {
                        formatter: OXFMT_LABEL.into(),
                        median: Some(MedianMetrics {
                            wall_time_ms: 10.0,
                            ops_per_second: 100.0,
                            cpu_user_ms: 1.0,
                            cpu_system_ms: 1.0,
                            cpu_percent: 20.0,
                            peak_rss_bytes: 1_048_576,
                            disk_usage_bytes: 512,
                            peak_fd_count: 4,
                            io_read_bytes: Some(1024),
                            io_write_bytes: Some(2048),
                        }),
                        runs: vec![],
                        command: "cmd".into(),
                        note: None,
                    }],
                },
                BenchmarkSummary {
                    scenario: "real-jsonc".into(),
                    repeats: 3,
                    sample_interval_ms: 20,
                    files: vec!["config.jsonc".into()],
                    file_count: 1,
                    results: vec![ToolSummary {
                        formatter: OXFMT_LABEL.into(),
                        median: Some(MedianMetrics {
                            wall_time_ms: 10.0,
                            ops_per_second: 121.0,
                            cpu_user_ms: 1.0,
                            cpu_system_ms: 1.0,
                            cpu_percent: 20.0,
                            peak_rss_bytes: 2_097_152,
                            disk_usage_bytes: 512,
                            peak_fd_count: 4,
                            io_read_bytes: Some(1024),
                            io_write_bytes: Some(2048),
                        }),
                        runs: vec![],
                        command: "cmd".into(),
                        note: None,
                    }],
                },
            ],
        };

        let rendered = render_score_summary(&report, None);
        assert!(rendered.contains("| oxfmt | 2 | 110.000 | 2.00 MB |"));
    }

    #[test]
    fn materializes_synthetic_json_corpus() {
        let corpus = materialize_synthetic_corpus(SyntheticFormat::Jsonc, 10).unwrap();
        assert_eq!(corpus.files.len(), 10);
        assert_eq!(corpus.scenario, "synthetic-jsonc-10");
        let first_path = corpus.input_dir.join("fixture-00000.jsonc");
        let source = fs::read_to_string(first_path).unwrap();
        assert!(source.contains("// synthetic jsonc fixture"));
        assert!(source.len() >= SYNTHETIC_FILE_SIZE_BYTES);
    }

    #[test]
    fn materializes_real_json_family_corpus() {
        let corpora = materialize_real_json_family().unwrap();
        assert_eq!(corpora.len(), 3);
        assert!(corpora.iter().any(|corpus| corpus.scenario == "real-json5"));
        assert!(corpora.iter().all(|corpus| !corpus.files.is_empty()));
    }

    #[test]
    fn samples_child_process_metrics() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("input.txt");
        let output_path = temp_dir.path().join("output.txt");
        fs::write(&input_path, "benchmark").unwrap();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exec 3<\"$1\"; cat \"$1\" > \"$2\"; sleep 0.05")
            .arg("sh")
            .arg(&input_path)
            .arg(&output_path)
            .spawn()
            .unwrap();

        let collector = MetricsCollector::new(child.id() as i32);
        let mut saw_fd = false;
        let mut saw_io_counters = false;
        #[cfg(target_os = "linux")]
        let mut saw_io = false;
        loop {
            if let Some(sample) = collector.sample() {
                saw_fd |= sample.fd_count > 0;
                saw_io_counters |= sample.read_bytes.is_some() || sample.write_bytes.is_some();
                #[cfg(target_os = "linux")]
                {
                    saw_io |= sample.read_bytes.unwrap_or_default() > 0
                        || sample.write_bytes.unwrap_or_default() > 0;
                }
            }
            if child.try_wait().unwrap().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(saw_fd);
        #[cfg(target_os = "linux")]
        assert!(saw_io);
        #[cfg(target_os = "macos")]
        assert!(saw_io_counters);
    }
}
