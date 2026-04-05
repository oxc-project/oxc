import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { collectSupportChecks, createSupportReport, getDefaultSupportOutputPath } from './report-svelte-support.mjs';
import { getDefaultCombinedOutputPath } from './report-svelte-beta.mjs';
import { getDefaultSoakOutputPath } from './report-svelte-beta-soak.mjs';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));

export function getDefaultReadinessOutputPath(format) {
  const extension = format === 'json' ? 'json' : 'md';
  return resolve(ROOT_PATH, `.svelte-beta/readiness.${extension}`);
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

function summarizeBetaProfile(label, reportPath, report, required) {
  if (report === null) {
    return {
      label,
      reportPath,
      required,
      present: false,
      status: 'missing-report',
      toolStatuses: {},
      blocking: required,
    };
  }

  const toolStatuses = Object.fromEntries(
    Object.entries(report.tools ?? {}).map(([toolName, summary]) => [
      toolName,
      {
        overallStatus: summary?.overallStatus ?? 'unknown',
        laneStatus: summary?.laneStatus ?? 'unknown',
        failedSteps: summary?.failedSteps?.length ?? 0,
      },
    ]),
  );

  const status = report.overallStatus ?? 'unknown';
  return {
    label,
    reportPath,
    required,
    present: true,
    status,
    toolStatuses,
    blocking: required && status !== 'passed',
  };
}

function summarizeSoakReport(reportPath, report) {
  if (report === null) {
    return {
      reportPath,
      present: false,
      status: 'missing-report',
      repoCount: 0,
      failedRepoCount: 0,
      failedRepos: [],
    };
  }

  const failedRepos = (report.repoSummaries ?? [])
    .filter((repo) => repo?.status && repo.status !== 'passed')
    .map((repo) => repo.name);

  return {
    reportPath,
    present: true,
    status: report.overallStatus ?? (failedRepos.length > 0 ? 'failed' : 'unknown'),
    repoCount: report.repoCount ?? report.repoRuns?.length ?? 0,
    failedRepoCount: report.failedRepoCount ?? failedRepos.length,
    failedRepos,
  };
}

function evaluateAnnouncementStatus({ latest, pinned, soak, support }) {
  if (support.status !== 'passed' || pinned.status !== 'passed') {
    return 'hold';
  }

  if (soak.present && soak.status === 'failed') {
    return 'hold';
  }

  if (!soak.present) {
    return 'needs-real-repo-soak';
  }

  if (latest.status !== 'passed') {
    return 'ready-with-canary-caveat';
  }

  return 'ready-for-beta-announcement';
}

function createChecklist({ latest, pinned, soak, support }) {
  return [
    {
      label: 'Support matrix / rollout docs are complete',
      status: support.status,
    },
    {
      label: 'Pinned combined beta lane is green',
      status: pinned.status,
    },
    {
      label: 'Latest-Svelte canary is green',
      status: latest.status,
    },
    {
      label: 'Real-repo beta soak has been run',
      status: soak.present ? soak.status : 'missing-report',
    },
  ];
}

function createRecommendedNextStep(announcementStatus) {
  switch (announcementStatus) {
    case 'ready-for-beta-announcement':
      return 'Publish the beta announcement / release-note wording and keep the pinned + latest-Svelte lanes as the ongoing guardrail.';
    case 'ready-with-canary-caveat':
      return 'You can announce beta support, but keep the canary caveat explicit and link the latest-Svelte follow-up bug if one exists.';
    case 'needs-real-repo-soak':
      return 'Run the real-repo beta soak on a few representative Svelte repos before posting the announcement.';
    default:
      return 'Hold the announcement, fix the blocking pinned/support/soak failures, then re-run the readiness report.';
  }
}

export function createReleaseReadinessReport({
  latestReportPath,
  latestBetaReport,
  pinnedReportPath,
  pinnedBetaReport,
  soakReportPath,
  soakBetaReport,
  supportOutputPath,
  supportReport,
}) {
  const support = {
    present: supportReport !== null,
    reportPath: supportOutputPath,
    status: supportReport?.overallStatus ?? 'missing-report',
  };
  const pinned = summarizeBetaProfile('pinned', pinnedReportPath, pinnedBetaReport, true);
  const latest = summarizeBetaProfile('latest-svelte', latestReportPath, latestBetaReport, false);
  const soak = summarizeSoakReport(soakReportPath, soakBetaReport);
  const announcementStatus = evaluateAnnouncementStatus({ latest, pinned, soak, support });
  const checklist = createChecklist({ latest, pinned, soak, support });

  return {
    announcementStatus,
    checklist,
    createdAt: new Date().toISOString(),
    latest,
    pinned,
    recommendedNextStep: createRecommendedNextStep(announcementStatus),
    soak,
    support,
  };
}

function renderStatusEmoji(status) {
  switch (status) {
    case 'passed':
      return '✅';
    case 'ready-for-beta-announcement':
      return '✅';
    case 'ready-with-canary-caveat':
      return '⚠️';
    case 'needs-real-repo-soak':
      return '🕒';
    case 'hold':
    case 'failed':
      return '❌';
    case 'missing-report':
      return '🕒';
    default:
      return '•';
  }
}

function renderBetaProfileSection(summary) {
  const lines = [];
  lines.push(`## ${summary.label}`);
  lines.push('');
  lines.push(`- Report found: ${summary.present ? 'yes' : 'no'}`);
  lines.push(`- Status: ${summary.status}`);
  lines.push(`- Required blocker: ${summary.required ? 'yes' : 'no'}`);
  lines.push(`- Source report: \
\`${summary.reportPath}\``);

  const toolEntries = Object.entries(summary.toolStatuses);
  if (toolEntries.length > 0) {
    lines.push('');
    lines.push('| Tool | Overall | Lane | Failed steps |');
    lines.push('| --- | --- | --- | --- |');
    for (const [toolName, toolStatus] of toolEntries) {
      lines.push(
        `| ${toolName} | ${toolStatus.overallStatus} | ${toolStatus.laneStatus} | ${toolStatus.failedSteps} |`,
      );
    }
  }

  lines.push('');
  return lines.join('\n');
}

export function renderReleaseReadinessMarkdown(report) {
  const lines = [];
  lines.push('# Svelte release readiness');
  lines.push('');
  lines.push(`- Generated at: ${report.createdAt}`);
  lines.push(`- Announcement status: **${report.announcementStatus}** ${renderStatusEmoji(report.announcementStatus)}`);
  lines.push(`- Recommended next step: ${report.recommendedNextStep}`);
  lines.push('');
  lines.push('## Checklist');
  lines.push('');
  lines.push('| Item | Status |');
  lines.push('| --- | --- |');
  for (const entry of report.checklist) {
    lines.push(`| ${entry.label} | ${renderStatusEmoji(entry.status)} ${entry.status} |`);
  }
  lines.push('');
  lines.push('## Support matrix');
  lines.push('');
  lines.push(`- Status: ${report.support.status}`);
  lines.push(`- Source report: \
\`${report.support.reportPath}\``);
  lines.push('');
  lines.push(renderBetaProfileSection(report.pinned));
  lines.push(renderBetaProfileSection(report.latest));
  lines.push('## Real-repo beta soak');
  lines.push('');
  lines.push(`- Report found: ${report.soak.present ? 'yes' : 'no'}`);
  lines.push(`- Status: ${report.soak.status}`);
  lines.push(`- Repo count: ${report.soak.repoCount}`);
  lines.push(`- Failed repos: ${report.soak.failedRepoCount}`);
  lines.push(`- Source report: \
\`${report.soak.reportPath}\``);

  if (report.soak.failedRepos.length > 0) {
    lines.push('');
    lines.push('### Failing soak repos');
    for (const repoName of report.soak.failedRepos) {
      lines.push(`- ${repoName}`);
    }
  }

  lines.push('');
  return `${lines.join('\n').trimEnd()}\n`;
}

export async function writeReleaseReadinessOutputs(options, report) {
  await mkdir(dirname(options.jsonOutputPath), { recursive: true });
  await writeFile(options.jsonOutputPath, `${JSON.stringify(report, null, 2)}\n`);

  const markdown = renderReleaseReadinessMarkdown(report);
  await mkdir(dirname(options.markdownOutputPath), { recursive: true });
  await writeFile(options.markdownOutputPath, markdown);

  if (options.appendSummary && process.env.GITHUB_STEP_SUMMARY) {
    await appendFile(process.env.GITHUB_STEP_SUMMARY, `\n${markdown}`);
  }

  return markdown;
}

export function parseCliOptions(argv) {
  let appendSummary = false;
  let jsonOutputPath = null;
  let latestReportPath = getDefaultCombinedOutputPath('latest-svelte', 'json');
  let markdownOutputPath = null;
  let pinnedReportPath = getDefaultCombinedOutputPath('pinned', 'json');
  let soakReportPath = getDefaultSoakOutputPath('json');
  let strict = false;
  let supportOutputPath = getDefaultSupportOutputPath('json');

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

    if (arg === '--latest-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --latest-report.');
      }
      latestReportPath = value;
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

    if (arg === '--pinned-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --pinned-report.');
      }
      pinnedReportPath = value;
      index += 1;
      continue;
    }

    if (arg === '--soak-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --soak-report.');
      }
      soakReportPath = value;
      index += 1;
      continue;
    }

    if (arg === '--strict') {
      strict = true;
      continue;
    }

    if (arg === '--support-report') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Expected a file path after --support-report.');
      }
      supportOutputPath = value;
      index += 1;
      continue;
    }

    if (arg === '--help') {
      process.stdout.write(
        'Usage: node ./scripts/report-svelte-release-readiness.mjs [--pinned-report <path>] [--latest-report <path>] [--soak-report <path>] [--support-report <path>] [--markdown-output <path>] [--json-output <path>] [--append-summary] [--strict]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    jsonOutputPath: jsonOutputPath ?? getDefaultReadinessOutputPath('json'),
    latestReportPath,
    markdownOutputPath: markdownOutputPath ?? getDefaultReadinessOutputPath('markdown'),
    pinnedReportPath,
    soakReportPath,
    strict,
    supportOutputPath,
  };
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const [pinnedBetaReport, latestBetaReport, soakBetaReport] = await Promise.all([
    safeReadJson(options.pinnedReportPath),
    safeReadJson(options.latestReportPath),
    safeReadJson(options.soakReportPath),
  ]);

  let supportReport = await safeReadJson(options.supportOutputPath);
  if (supportReport === null) {
    supportReport = createSupportReport({ checks: await collectSupportChecks() });
  }

  const report = createReleaseReadinessReport({
    latestReportPath: options.latestReportPath,
    latestBetaReport,
    pinnedReportPath: options.pinnedReportPath,
    pinnedBetaReport,
    soakReportPath: options.soakReportPath,
    soakBetaReport,
    supportOutputPath: options.supportOutputPath,
    supportReport,
  });

  await writeReleaseReadinessOutputs(options, report);

  if (options.strict && report.announcementStatus !== 'ready-for-beta-announcement') {
    process.exitCode = 1;
  }

  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
