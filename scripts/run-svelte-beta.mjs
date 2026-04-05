import { access, appendFile, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  createCombinedReport,
  getDefaultCombinedOutputPath,
  getDefaultToolReportPath,
  writeCombinedReportOutputs,
} from './report-svelte-beta.mjs';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));
const PNPM_COMMAND = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
const TOOL_FILTERS = {
  oxlint: './apps/oxlint',
  oxfmt: './apps/oxfmt',
};
const PROFILE_NAMES = ['pinned', 'latest-svelte'];
const TOOL_NAMES = ['oxlint', 'oxfmt'];

function parseProfileName(value) {
  const profileName = value ?? 'pinned';
  if (!PROFILE_NAMES.includes(profileName)) {
    throw new Error(`Unknown Svelte beta profile: ${profileName}. Expected one of: ${PROFILE_NAMES.join(', ')}.`);
  }
  return profileName;
}

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
      throw new Error(`Unknown Svelte beta tool: ${part}. Expected oxlint, oxfmt, or both.`);
    }
  }

  return [...new Set(parts)];
}

function parseReportMode(value) {
  const reportMode = value ?? 'always';
  if (!['always', 'failure', 'never'].includes(reportMode)) {
    throw new Error(`Unknown report mode: ${reportMode}. Expected always, failure, or never.`);
  }
  return reportMode;
}

function parseCliOptions(argv) {
  let appendSummary = false;
  let build = false;
  let jsonOutputPath = null;
  let keepOnFailure = false;
  let markdownOutputPath = null;
  let profileName = parseProfileName(undefined);
  let reportMode = parseReportMode(undefined);
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

    if (arg === '--json-output') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --json-output.');
      }
      jsonOutputPath = value;
      index += 1;
      continue;
    }

    if (arg === '--keep-on-failure') {
      keepOnFailure = true;
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

    if (arg === '--profile') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a profile name after --profile.');
      }
      profileName = parseProfileName(value);
      index += 1;
      continue;
    }

    if (arg === '--report-mode') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a report mode after --report-mode.');
      }
      reportMode = parseReportMode(value);
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
        'Usage: node ./scripts/run-svelte-beta.mjs [--profile <pinned|latest-svelte>] [--tools <both|oxlint|oxfmt|oxlint,oxfmt>] [--build] [--report-mode <always|failure|never>] [--markdown-output <path>] [--json-output <path>] [--append-summary] [--keep-on-failure]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    build,
    jsonOutputPath: jsonOutputPath ?? getDefaultCombinedOutputPath(profileName, 'json'),
    keepOnFailure,
    markdownOutputPath: markdownOutputPath ?? getDefaultCombinedOutputPath(profileName, 'markdown'),
    profileName,
    reportMode,
    selectedToolNames,
  };
}

function spawnManagedLane(toolName, options) {
  const args = ['--filter', TOOL_FILTERS[toolName], 'run', 'test:svelte-real-packages:managed', '--'];
  args.push('--profile', options.profileName, '--report-mode', options.reportMode);

  if (options.build) {
    args.push('--build');
  }

  if (options.keepOnFailure) {
    args.push('--keep-on-failure');
  }

  process.stdout.write(`\n> ${PNPM_COMMAND} ${args.join(' ')}\n`);
  return spawnSync(PNPM_COMMAND, args, {
    cwd: ROOT_PATH,
    env: process.env,
    stdio: 'inherit',
  }).status;
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

async function appendMarkdownReportToSummary(markdownReportPath) {
  const summaryPath = process.env.GITHUB_STEP_SUMMARY;
  if (!summaryPath) {
    return;
  }

  await access(markdownReportPath);
  const contents = await readFile(markdownReportPath, 'utf8');
  await appendFile(summaryPath, `\n${contents}`);
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const exitCodes = [];

  for (const toolName of options.selectedToolNames) {
    const exitCode = spawnManagedLane(toolName, options);
    exitCodes.push({ exitCode, toolName });
  }

  const lintReportPath = getDefaultToolReportPath('oxlint', options.profileName);
  const formatReportPath = getDefaultToolReportPath('oxfmt', options.profileName);
  const lintReport = options.selectedToolNames.includes('oxlint') ? await safeReadJson(lintReportPath) : null;
  const formatReport = options.selectedToolNames.includes('oxfmt') ? await safeReadJson(formatReportPath) : null;

  const report = createCombinedReport({
    profileName: options.profileName,
    lintReportPath,
    lintReport,
    formatReportPath,
    formatReport,
    selectedToolNames: options.selectedToolNames,
  });

  await writeCombinedReportOutputs(
    {
      appendSummary: false,
      jsonOutputPath: options.jsonOutputPath,
      markdownOutputPath: options.markdownOutputPath,
    },
    report,
  );

  if (options.appendSummary) {
    await appendMarkdownReportToSummary(options.markdownOutputPath);
  }

  if (exitCodes.some((entry) => entry.exitCode !== 0)) {
    process.exitCode = 1;
  }

  return { exitCodes, report };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
