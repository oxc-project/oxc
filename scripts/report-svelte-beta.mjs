import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));
const PROFILE_NAMES = ['pinned', 'latest-svelte'];
const TOOL_NAMES = ['oxlint', 'oxfmt'];

export function getProfileSuffix(profileName) {
  return profileName === 'pinned' ? '' : `.${profileName}`;
}

export function getDefaultToolReportPath(toolName, profileName) {
  const suffix = getProfileSuffix(profileName);
  const basePath = toolName === 'oxlint' ? 'apps/oxlint/test' : 'apps/oxfmt/test';
  return resolve(ROOT_PATH, `${basePath}/.real-svelte-packages${suffix}-report.json`);
}

export function getDefaultCombinedOutputPath(profileName, format) {
  const suffix = getProfileSuffix(profileName);
  const extension = format === 'json' ? 'json' : 'md';
  return resolve(ROOT_PATH, `.svelte-beta/report${suffix}.${extension}`);
}

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

export function parseCliOptions(argv) {
  let appendSummary = false;
  let jsonOutputPath = null;
  let lintReportPath = null;
  let markdownOutputPath = null;
  let profileName = parseProfileName(undefined);
  let strict = false;
  let formatReportPath = null;
  let selectedToolNames = [...TOOL_NAMES];

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === '--append-summary') {
      appendSummary = true;
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

    if (arg === '--lint-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --lint-report.');
      }
      lintReportPath = value;
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

    if (arg === '--format-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --format-report.');
      }
      formatReportPath = value;
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

    if (arg === '--strict') {
      strict = true;
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
        'Usage: node ./scripts/report-svelte-beta.mjs [--profile <pinned|latest-svelte>] [--tools <both|oxlint|oxfmt|oxlint,oxfmt>] [--lint-report <path>] [--format-report <path>] [--markdown-output <path>] [--json-output <path>] [--append-summary] [--strict]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    jsonOutputPath: jsonOutputPath ?? getDefaultCombinedOutputPath(profileName, 'json'),
    lintReportPath: lintReportPath ?? getDefaultToolReportPath('oxlint', profileName),
    markdownOutputPath: markdownOutputPath ?? getDefaultCombinedOutputPath(profileName, 'markdown'),
    profileName,
    strict,
    formatReportPath: formatReportPath ?? getDefaultToolReportPath('oxfmt', profileName),
    selectedToolNames,
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

function getFailedSteps(report) {
  return (report?.managedRunState?.steps ?? []).filter((step) => step?.status === 'failed');
}

function countPackageIssues(report) {
  return (report?.packages ?? []).filter((entry) => entry?.status && entry.status !== 'ok').length;
}

function countLinkIssues(report) {
  return (report?.links ?? []).filter((entry) => entry?.status && entry.status !== 'ok').length;
}

function getResolutionEntries(report) {
  return report?.resolutions ?? report?.testRootResolutions ?? [];
}

function countResolutionIssues(report) {
  return getResolutionEntries(report).filter(
    (entry) => entry?.error || entry?.insideInstallNodeModules === false,
  ).length;
}

function countFixtureIssues(report) {
  return (report?.fixtures ?? []).filter((entry) => entry?.status && entry.status !== 'ok').length;
}

function getNpmLsExitCode(report) {
  const exitCode = report?.npmLs?.exitCode;
  return typeof exitCode === 'number' ? exitCode : null;
}

export function summarizeToolReport(toolName, reportPath, report) {
  if (report === null) {
    return {
      toolName,
      reportPath,
      present: false,
      overallStatus: 'missing-report',
      laneStatus: 'missing-report',
      failedSteps: [],
      packageIssueCount: 0,
      linkIssueCount: 0,
      resolutionIssueCount: 0,
      fixtureIssueCount: 0,
      npmLsExitCode: null,
      installPackageJsonMatches: null,
    };
  }

  const failedSteps = getFailedSteps(report);
  const packageIssueCount = countPackageIssues(report);
  const linkIssueCount = countLinkIssues(report);
  const resolutionIssueCount = countResolutionIssues(report);
  const fixtureIssueCount = countFixtureIssues(report);
  const npmLsExitCode = getNpmLsExitCode(report);
  const laneStatus = report?.managedRunState?.laneStatus ?? (failedSteps.length > 0 ? 'failed' : 'unknown');
  const hasFailure =
    report?.installPackageJsonMatches === false ||
    laneStatus === 'failed' ||
    failedSteps.length > 0 ||
    packageIssueCount > 0 ||
    linkIssueCount > 0 ||
    resolutionIssueCount > 0 ||
    fixtureIssueCount > 0 ||
    (typeof npmLsExitCode === 'number' && npmLsExitCode !== 0);

  return {
    toolName,
    reportPath,
    present: true,
    overallStatus: hasFailure ? 'failed' : 'passed',
    laneStatus,
    failedSteps,
    packageIssueCount,
    linkIssueCount,
    resolutionIssueCount,
    fixtureIssueCount,
    npmLsExitCode,
    installPackageJsonMatches: report?.installPackageJsonMatches ?? null,
  };
}

export function createCombinedReport({
  profileName,
  lintReportPath,
  lintReport,
  formatReportPath,
  formatReport,
  selectedToolNames,
}) {
  const tools = {};

  if (selectedToolNames.includes('oxlint')) {
    tools.oxlint = summarizeToolReport('oxlint', lintReportPath, lintReport);
  }

  if (selectedToolNames.includes('oxfmt')) {
    tools.oxfmt = summarizeToolReport('oxfmt', formatReportPath, formatReport);
  }

  const summaries = Object.values(tools);
  const overallStatus = summaries.length > 0 && summaries.every((entry) => entry.overallStatus === 'passed')
    ? 'passed'
    : 'failed';

  return {
    generatedAt: new Date().toISOString(),
    profileName,
    overallStatus,
    selectedToolNames,
    tools,
  };
}

function renderToolSection(summary) {
  const title = summary.toolName === 'oxlint' ? 'Oxlint' : 'Oxfmt';
  const lines = [
    `## ${title}`,
    '',
    `- Report found: ${summary.present ? 'yes' : 'no'}`,
    `- Overall status: ${summary.overallStatus}`,
    `- Lane status: ${summary.laneStatus}`,
    `- Helper manifest matches: ${summary.installPackageJsonMatches === null ? 'unknown' : summary.installPackageJsonMatches ? 'yes' : 'no'}`,
    `- Package issues: ${summary.packageIssueCount}`,
    `- Link issues: ${summary.linkIssueCount}`,
    `- Resolution issues: ${summary.resolutionIssueCount}`,
    `- Fixture issues: ${summary.fixtureIssueCount}`,
    `- Failed managed steps: ${summary.failedSteps.length}`,
    `- npm ls exit code: ${summary.npmLsExitCode === null ? 'unknown' : summary.npmLsExitCode}`,
    `- Source report: \`${summary.reportPath}\``,
  ];

  if (summary.failedSteps.length > 0) {
    lines.push('', '### Failed steps');
    for (const step of summary.failedSteps) {
      lines.push(
        `- ${step.stepName ?? step.scriptName ?? 'unknown step'}: ${step.errorMessage ?? `exit code ${step.exitCode ?? 'unknown'}`}`,
      );
    }
  }

  return `${lines.join('\n').trimEnd()}\n`;
}

export function renderCombinedMarkdownReport(report) {
  const lines = [
    `# Svelte beta report (${report.profileName})`,
    '',
    `- Generated at: ${report.generatedAt}`,
    `- Overall status: ${report.overallStatus}`,
    `- Included tools: ${report.selectedToolNames.join(', ')}`,
    '',
  ];

  for (const toolName of report.selectedToolNames) {
    const summary = report.tools[toolName];
    if (summary) {
      lines.push(renderToolSection(summary));
    }
  }

  return `${lines.join('\n').trimEnd()}\n`;
}

export async function writeCombinedReportOutputs(options, report) {
  const markdown = renderCombinedMarkdownReport(report);
  const json = `${JSON.stringify(report, null, 2)}\n`;

  await mkdir(dirname(options.markdownOutputPath), { recursive: true });
  await mkdir(dirname(options.jsonOutputPath), { recursive: true });
  await writeFile(options.markdownOutputPath, markdown);
  await writeFile(options.jsonOutputPath, json);

  if (options.appendSummary) {
    const summaryPath = process.env.GITHUB_STEP_SUMMARY;
    if (summaryPath) {
      await appendFile(summaryPath, `\n${markdown}`);
    }
  }

  return { markdown, json };
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const lintReport = options.selectedToolNames.includes('oxlint') ? await safeReadJson(options.lintReportPath) : null;
  const formatReport = options.selectedToolNames.includes('oxfmt') ? await safeReadJson(options.formatReportPath) : null;
  const report = createCombinedReport({
    profileName: options.profileName,
    lintReportPath: options.lintReportPath,
    lintReport,
    formatReportPath: options.formatReportPath,
    formatReport,
    selectedToolNames: options.selectedToolNames,
  });

  await writeCombinedReportOutputs(options, report);
  process.stdout.write(`Wrote ${options.markdownOutputPath} and ${options.jsonOutputPath}.\n`);

  if (options.strict && report.overallStatus !== 'passed') {
    process.exitCode = 1;
  }

  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
