import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import {
  createBetaSoakReport,
  getDefaultSoakOutputPath,
  writeBetaSoakReportOutputs,
} from './report-svelte-beta-soak.mjs';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));
const PNPM_COMMAND = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
const NODE_COMMAND = process.platform === 'win32' ? 'node.exe' : 'node';
const TOOL_NAMES = ['oxlint', 'oxfmt'];
const TOOL_BUILD_FILTERS = {
  oxlint: './apps/oxlint',
  oxfmt: './apps/oxfmt',
};
const TOOL_ENTRY_POINTS = {
  oxlint: resolve(ROOT_PATH, 'apps/oxlint/dist/cli.js'),
  oxfmt: resolve(ROOT_PATH, 'apps/oxfmt/dist/cli.js'),
};

function parseToolNames(value) {
  if (value === undefined || value === null || value === '' || value === 'both') {
    return [...TOOL_NAMES];
  }

  const parts = value
    .split(',')
    .map((part) => part.trim())
    .filter(Boolean);

  if (parts.length === 0) {
    return [...TOOL_NAMES];
  }

  for (const part of parts) {
    if (!TOOL_NAMES.includes(part)) {
      throw new Error(`Unknown Svelte beta soak tool: ${part}. Expected oxlint, oxfmt, or both.`);
    }
  }

  return [...new Set(parts)];
}

function parseRepoNames(value) {
  if (!value) {
    return null;
  }

  const parts = value
    .split(',')
    .map((part) => part.trim())
    .filter(Boolean);

  return parts.length === 0 ? null : new Set(parts);
}

function parseCliOptions(argv) {
  let appendSummary = false;
  let build = false;
  let jsonOutputPath = null;
  let keepGoing = true;
  let manifestPath = resolve(ROOT_PATH, 'svelte-beta-repos.json');
  let markdownOutputPath = null;
  let selectedRepoNames = null;
  let selectedToolNames = [...TOOL_NAMES];

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === '--append-summary') {
      appendSummary = true;
      continue;
    }

    if (arg === '--build') {
      build = true;
      continue;
    }

    if (arg === '--fail-fast') {
      keepGoing = false;
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

    if (arg === '--manifest') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --manifest.');
      }
      manifestPath = resolve(ROOT_PATH, value);
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

    if (arg === '--repos') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a repo list after --repos.');
      }
      selectedRepoNames = parseRepoNames(value);
      index += 1;
      continue;
    }

    if (arg === '--tools') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a tool list after --tools.');
      }
      selectedToolNames = parseToolNames(value);
      index += 1;
      continue;
    }

    if (arg === '--help') {
      process.stdout.write(
        'Usage: node ./scripts/run-svelte-beta-soak.mjs [--manifest <path>] [--repos <name[,name...]>] [--tools <both|oxlint|oxfmt|oxlint,oxfmt>] [--build] [--fail-fast] [--markdown-output <path>] [--json-output <path>] [--append-summary]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    build,
    jsonOutputPath: jsonOutputPath ?? getDefaultSoakOutputPath('json'),
    keepGoing,
    manifestPath,
    markdownOutputPath: markdownOutputPath ?? getDefaultSoakOutputPath('markdown'),
    selectedRepoNames,
    selectedToolNames,
  };
}

function getDefaultToolArgs(toolName) {
  return toolName === 'oxfmt' ? ['--check', '.'] : ['.'];
}

function sanitizeName(value) {
  return value.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '').toLowerCase() || 'repo';
}

async function readManifest(path) {
  const raw = await readFile(path, 'utf8');
  const manifest = JSON.parse(raw);

  if (!Array.isArray(manifest.repos)) {
    throw new Error(`Expected ${path} to contain a top-level \"repos\" array.`);
  }

  return manifest;
}

function normalizeRepoEntry(entry, manifestPath, selectedToolNames) {
  if (!entry || typeof entry !== 'object') {
    throw new Error(`Invalid repo entry in ${manifestPath}. Expected an object.`);
  }

  if (typeof entry.name !== 'string' || entry.name.trim() === '') {
    throw new Error(`Invalid repo entry in ${manifestPath}. Expected each repo to have a non-empty \"name\".`);
  }

  if (typeof entry.root !== 'string' || entry.root.trim() === '') {
    throw new Error(`Invalid repo entry for ${entry.name}. Expected a non-empty \"root\".`);
  }

  const repoRootPath = resolve(dirname(manifestPath), entry.root);
  const selectedToolSet = new Set(Array.isArray(entry.tools) && entry.tools.length > 0 ? entry.tools : selectedToolNames);
  const toolRuns = [];

  for (const toolName of selectedToolNames) {
    const config = toolName === 'oxlint' ? entry.lint : entry.format;
    const enabled = selectedToolSet.has(toolName) && config?.enabled !== false;
    const args = Array.isArray(config?.args) && config.args.length > 0 ? [...config.args] : getDefaultToolArgs(toolName);
    const cwd = resolve(repoRootPath, config?.cwd ?? '.');

    toolRuns.push({
      args,
      commandDisplay: `${NODE_COMMAND} ${TOOL_ENTRY_POINTS[toolName]} ${args.join(' ')}`.trim(),
      cwd,
      enabled,
      toolName,
    });
  }

  return {
    name: entry.name,
    rootPath: repoRootPath,
    toolRuns,
  };
}

async function ensureFileExists(path, description) {
  try {
    await access(path);
  } catch {
    throw new Error(`${description} not found: ${relative(ROOT_PATH, path)}`);
  }
}

function runBuild(toolName) {
  const args = ['--filter', TOOL_BUILD_FILTERS[toolName], 'run', 'build-test'];
  process.stdout.write(`\n> ${PNPM_COMMAND} ${args.join(' ')}\n`);
  const result = spawnSync(PNPM_COMMAND, args, {
    cwd: ROOT_PATH,
    env: process.env,
    stdio: 'inherit',
  });

  if (result.status !== 0) {
    throw new Error(`Failed to build ${toolName} for Svelte beta soak.`);
  }
}

async function writeToolLog(path, contents) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, contents ?? '');
}

function runToolForRepo(repoName, toolRun, logRootPath) {
  const startedAt = Date.now();
  const repoSlug = sanitizeName(repoName);
  const toolSlug = sanitizeName(toolRun.toolName);
  const stdoutPath = resolve(logRootPath, repoSlug, `${toolSlug}.stdout.log`);
  const stderrPath = resolve(logRootPath, repoSlug, `${toolSlug}.stderr.log`);

  process.stdout.write(`\n> ${toolRun.commandDisplay}\n`);
  process.stdout.write(`  cwd: ${toolRun.cwd}\n`);

  const result = spawnSync(NODE_COMMAND, [TOOL_ENTRY_POINTS[toolRun.toolName], ...toolRun.args], {
    cwd: toolRun.cwd,
    env: process.env,
    encoding: 'utf8',
  });

  const durationMs = Date.now() - startedAt;

  return {
    ...toolRun,
    durationMs,
    errorMessage: result.error ? String(result.error.message ?? result.error) : null,
    exitCode: typeof result.status === 'number' ? result.status : 1,
    status: typeof result.status === 'number' && result.status === 0 ? 'passed' : 'failed',
    stderr: result.stderr ?? '',
    stderrPath,
    stdout: result.stdout ?? '',
    stdoutPath,
  };
}

async function persistToolLogs(run) {
  await writeToolLog(run.stdoutPath, run.stdout ?? '');
  await writeToolLog(run.stderrPath, run.stderr ?? '');
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const manifest = await readManifest(options.manifestPath);

  if (options.build) {
    for (const toolName of options.selectedToolNames) {
      runBuild(toolName);
    }
  }

  for (const toolName of options.selectedToolNames) {
    await ensureFileExists(TOOL_ENTRY_POINTS[toolName], `${toolName} CLI entry point`);
  }

  const normalizedRepos = manifest.repos
    .map((entry) => normalizeRepoEntry(entry, options.manifestPath, options.selectedToolNames))
    .filter((entry) => (options.selectedRepoNames ? options.selectedRepoNames.has(entry.name) : true));

  const logRootPath = resolve(ROOT_PATH, '.svelte-beta-soak/logs');
  const repoRuns = [];

  for (const repo of normalizedRepos) {
    const repoRun = {
      name: repo.name,
      rootPath: repo.rootPath,
      status: 'passed',
      toolRuns: [],
    };

    try {
      await access(repo.rootPath);
    } catch {
      repoRun.status = 'failed';
      repoRun.toolRuns.push({
        commandDisplay: 'n/a',
        cwd: repo.rootPath,
        durationMs: 0,
        enabled: true,
        errorMessage: `Repository root not found: ${repo.rootPath}`,
        exitCode: 1,
        status: 'failed',
        stderrPath: null,
        stdoutPath: null,
        toolName: 'manifest',
      });
      repoRuns.push(repoRun);
      if (!options.keepGoing) {
        break;
      }
      continue;
    }

    for (const toolRun of repo.toolRuns) {
      if (!toolRun.enabled) {
        repoRun.toolRuns.push({ ...toolRun, durationMs: 0, exitCode: 0, status: 'not-run' });
        continue;
      }

      const executedRun = runToolForRepo(repo.name, toolRun, logRootPath);
      await persistToolLogs(executedRun);
      repoRun.toolRuns.push({ ...executedRun, stderr: undefined, stdout: undefined });

      if (executedRun.status !== 'passed') {
        repoRun.status = 'failed';
        if (!options.keepGoing) {
          break;
        }
      }
    }

    repoRuns.push(repoRun);

    if (repoRun.status === 'failed' && !options.keepGoing) {
      break;
    }
  }

  const report = createBetaSoakReport({
    manifestPath: options.manifestPath,
    repoRuns,
    selectedToolNames: options.selectedToolNames,
  });

  await writeBetaSoakReportOutputs(
    {
      appendSummary: options.appendSummary,
      jsonOutputPath: options.jsonOutputPath,
      markdownOutputPath: options.markdownOutputPath,
    },
    report,
  );

  if (report.overallStatus !== 'passed') {
    process.exitCode = 1;
  }

  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
