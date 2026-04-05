import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));

export function getDefaultSoakOutputPath(format) {
  const extension = format === 'json' ? 'json' : 'md';
  return resolve(ROOT_PATH, `.svelte-beta-soak/report.${extension}`);
}

export function parseCliOptions(argv) {
  let appendSummary = false;
  let inputPath = getDefaultSoakOutputPath('json');
  let jsonOutputPath = null;
  let markdownOutputPath = null;
  let strict = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === '--append-summary') {
      appendSummary = true;
      continue;
    }

    if (arg === '--input') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --input.');
      }
      inputPath = value;
      index += 1;
      continue;
    }

    if (arg === '--json-output') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --json-output.');
      }
      jsonOutputPath = value;
      index += 1;
      continue;
    }

    if (arg === '--markdown-output') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --markdown-output.');
      }
      markdownOutputPath = value;
      index += 1;
      continue;
    }

    if (arg === '--strict') {
      strict = true;
      continue;
    }

    if (arg === '--help') {
      process.stdout.write(
        'Usage: node ./scripts/report-svelte-beta-soak.mjs [--input <path>] [--markdown-output <path>] [--json-output <path>] [--append-summary] [--strict]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    inputPath,
    jsonOutputPath: jsonOutputPath ?? getDefaultSoakOutputPath('json'),
    markdownOutputPath: markdownOutputPath ?? getDefaultSoakOutputPath('markdown'),
    strict,
  };
}

async function safeReadJson(path) {
  try {
    return JSON.parse(await readFile(path, 'utf8'));
  } catch (error) {
    if (error && typeof error === 'object' && error.code === 'ENOENT') {
      return null;
    }
    throw error;
  }
}

function getRunStatus(run) {
  if (!run || run.enabled === false) {
    return 'not-run';
  }

  if (run.status) {
    return run.status;
  }

  if (typeof run.exitCode === 'number') {
    return run.exitCode === 0 ? 'passed' : 'failed';
  }

  return 'unknown';
}

function hasRepoFailure(repo) {
  if (repo.status && repo.status !== 'passed') {
    return true;
  }

  return (repo.toolRuns ?? []).some((run) => {
    const status = getRunStatus(run);
    return status !== 'passed' && status !== 'not-run';
  });
}

function summarizeRepo(repo) {
  const summaries = {};

  for (const run of repo.toolRuns ?? []) {
    summaries[run.toolName] = {
      status: getRunStatus(run),
      exitCode: typeof run.exitCode === 'number' ? run.exitCode : null,
      durationMs: typeof run.durationMs === 'number' ? run.durationMs : null,
    };
  }

  return {
    name: repo.name,
    rootPath: repo.rootPath,
    status: hasRepoFailure(repo) ? 'failed' : 'passed',
    toolSummaries: summaries,
  };
}

export function createBetaSoakReport({ manifestPath, repoRuns, selectedToolNames }) {
  const repoSummaries = repoRuns.map((repo) => summarizeRepo(repo));
  const failedRepoCount = repoSummaries.filter((repo) => repo.status === 'failed').length;
  const passedRepoCount = repoSummaries.filter((repo) => repo.status === 'passed').length;

  return {
    createdAt: new Date().toISOString(),
    manifestPath,
    overallStatus: failedRepoCount > 0 ? 'failed' : 'passed',
    repoCount: repoRuns.length,
    failedRepoCount,
    passedRepoCount,
    repoRuns,
    repoSummaries,
    selectedToolNames,
  };
}

function formatDuration(durationMs) {
  if (typeof durationMs !== 'number') {
    return 'n/a';
  }
  return `${(durationMs / 1000).toFixed(2)}s`;
}

function renderToolStatusCell(repoSummary, toolName) {
  const summary = repoSummary.toolSummaries[toolName];
  if (!summary) {
    return 'not-run';
  }

  if (summary.status === 'passed') {
    return `passed (${formatDuration(summary.durationMs)})`;
  }

  if (summary.status === 'failed') {
    return `failed (${summary.exitCode ?? 'n/a'})`;
  }

  return summary.status;
}

function renderFailureDetails(run) {
  const lines = [];
  lines.push(`- Command: \`${run.commandDisplay ?? 'n/a'}\``);
  lines.push(`- CWD: \`${run.cwd ?? 'n/a'}\``);
  lines.push(`- Exit code: ${typeof run.exitCode === 'number' ? run.exitCode : 'n/a'}`);
  lines.push(`- Duration: ${formatDuration(run.durationMs)}`);

  if (run.stdoutPath) {
    lines.push(`- Stdout log: \`${relative(ROOT_PATH, run.stdoutPath)}\``);
  }

  if (run.stderrPath) {
    lines.push(`- Stderr log: \`${relative(ROOT_PATH, run.stderrPath)}\``);
  }

  if (run.errorMessage) {
    lines.push(`- Error: ${run.errorMessage}`);
  }

  return lines.join('\n');
}

export function renderBetaSoakMarkdownReport(report) {
  const lines = [];
  lines.push('# Svelte beta soak report');
  lines.push('');
  lines.push(`- Overall status: **${report.overallStatus}**`);
  lines.push(`- Manifest: \`${relative(ROOT_PATH, report.manifestPath)}\``);
  lines.push(`- Selected tools: ${report.selectedToolNames.join(', ')}`);
  lines.push(`- Repos: ${report.repoCount} total, ${report.passedRepoCount} passed, ${report.failedRepoCount} failed`);
  lines.push('');
  lines.push('| Repo | Status | Oxlint | Oxfmt |');
  lines.push('| --- | --- | --- | --- |');

  for (const repoSummary of report.repoSummaries) {
    lines.push(
      `| ${repoSummary.name} | ${repoSummary.status} | ${renderToolStatusCell(repoSummary, 'oxlint')} | ${renderToolStatusCell(repoSummary, 'oxfmt')} |`,
    );
  }

  const failingRuns = report.repoRuns.flatMap((repo) =>
    (repo.toolRuns ?? [])
      .filter((run) => getRunStatus(run) === 'failed')
      .map((run) => ({ repoName: repo.name, run })),
  );

  if (failingRuns.length > 0) {
    lines.push('');
    lines.push('## Failures');
    lines.push('');

    for (const entry of failingRuns) {
      lines.push(`### ${entry.repoName} / ${entry.run.toolName}`);
      lines.push('');
      lines.push(renderFailureDetails(entry.run));
      lines.push('');
    }
  }

  return `${lines.join('\n').trimEnd()}\n`;
}

export async function writeBetaSoakReportOutputs(options, report) {
  await mkdir(dirname(options.jsonOutputPath), { recursive: true });
  await writeFile(options.jsonOutputPath, `${JSON.stringify(report, null, 2)}\n`);

  const markdown = renderBetaSoakMarkdownReport(report);
  await mkdir(dirname(options.markdownOutputPath), { recursive: true });
  await writeFile(options.markdownOutputPath, markdown);

  if (options.appendSummary && process.env.GITHUB_STEP_SUMMARY) {
    await appendFile(process.env.GITHUB_STEP_SUMMARY, `\n${markdown}`);
  }

  return markdown;
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const report = await safeReadJson(options.inputPath);

  if (report === null) {
    if (options.strict) {
      throw new Error(`Svelte beta soak report input not found: ${options.inputPath}`);
    }
    return null;
  }

  await writeBetaSoakReportOutputs(options, report);
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
