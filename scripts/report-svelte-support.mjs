import { access, appendFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT_PATH = resolve(fileURLToPath(new URL('..', import.meta.url)));
const OUTPUT_ROOT_PATH = resolve(ROOT_PATH, '.svelte-beta');

export const REQUIRED_ROOT_SCRIPTS = [
  'test:svelte-beta',
  'test:svelte-beta:latest-svelte',
  'report:svelte-beta',
  'report:svelte-beta:latest-svelte',
  'test:svelte-beta:soak',
  'report:svelte-beta:soak',
  'report:svelte-support',
  'check:svelte-support',
  'report:svelte-readiness',
  'check:svelte-readiness',
];

export const REQUIRED_WORKFLOWS = [
  {
    category: 'combined-beta',
    description: 'Manual combined beta validation workflow',
    path: '.github/workflows/svelte-beta-validation.yml',
  },
  {
    category: 'lint-ci',
    description: 'Pinned Oxlint Svelte CI in the main CI workflow',
    path: '.github/workflows/ci.yml',
  },
  {
    category: 'lint-canary',
    description: 'Latest-Svelte Oxlint upstream canary workflow',
    path: '.github/workflows/oxlint-svelte-upstream-canary.yml',
  },
  {
    category: 'format-canary',
    description: 'Latest-Svelte Oxfmt upstream canary workflow',
    path: '.github/workflows/oxfmt-svelte-upstream-canary.yml',
  },
];

export const REQUIRED_DOCS = [
  {
    category: 'status-guide',
    description: 'Maintainer-facing Svelte support guide',
    path: 'SVELTE_SUPPORT.md',
  },
  {
    category: 'soak-guide',
    description: 'Real-repo beta soak guide',
    path: 'SVELTE_BETA_SOAK.md',
  },
  {
    category: 'announcement-template',
    description: 'Announcement / release-note wording template',
    path: 'SVELTE_ANNOUNCEMENT_TEMPLATE.md',
  },
  {
    category: 'release-checklist',
    description: 'Release-readiness checklist and status guide',
    path: 'SVELTE_RELEASE_CHECKLIST.md',
  },
  {
    category: 'phase-tracker',
    description: 'Svelte implementation phase tracker',
    path: 'svelte-support-phases.md',
  },
  {
    category: 'soak-manifest-example',
    description: 'Example beta soak manifest',
    path: 'svelte-beta-repos.example.json',
  },
];

export const SUPPORT_MATRIX = {
  profiles: [
    {
      name: 'pinned',
      description: 'Pinned ecosystem versions used in normal CI lanes.',
    },
    {
      name: 'latest-svelte',
      description: 'Floating upstream-drift canary for current Svelte ecosystem releases.',
    },
  ],
  tools: [
    {
      caveats: [
        'Uses whole-file custom-parser interop for `.svelte`.',
        'Type-aware embedded-language corner cases remain the main residual risk area.',
      ],
      capabilities: [
        'CLI/runtime linting for real-package `.svelte` configs',
        'Built vs raw CLI smoke parity',
        'Fixes and `--fix-suggestions` parity checks',
        'LSP diagnostics, code actions, config reload, and workspace isolation',
        'Linux + Windows pinned-package CI',
        'Latest-Svelte upstream-drift canary',
      ],
      ciSurfaces: ['runtime', 'fixtures', 'smoke', 'lsp'],
      implementation: 'Hybrid JS-plugin/custom-parser interop',
      name: 'oxlint',
      supportLevel: 'beta',
    },
    {
      caveats: [
        'Uses external formatter-plugin interop with `prettier-plugin-svelte`.',
        'Migration is strong for common Svelte configs but intentionally lossy cases still exist.',
      ],
      capabilities: [
        'JS API formatting for real-package `.svelte` flows',
        'CLI formatting and check/list-different coverage',
        'LSP formatting coverage',
        'Built vs raw formatter smoke parity',
        'Linux + Windows pinned-package CI',
        'Latest-Svelte upstream-drift canary',
      ],
      ciSurfaces: ['api', 'cli', 'lsp', 'smoke'],
      implementation: 'External plugin interop via `prettier-plugin-svelte`',
      name: 'oxfmt',
      supportLevel: 'beta',
    },
  ],
};

export function getDefaultSupportOutputPath(format) {
  const extension = format === 'json' ? 'json' : 'md';
  return resolve(OUTPUT_ROOT_PATH, `support-matrix.${extension}`);
}

async function pathExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function readRootScripts() {
  const packageJson = JSON.parse(await readFile(resolve(ROOT_PATH, 'package.json'), 'utf8'));
  return packageJson.scripts ?? {};
}

export async function collectSupportChecks(rootPath = ROOT_PATH) {
  const rootScripts = await readRootScripts();
  const scriptChecks = REQUIRED_ROOT_SCRIPTS.map((scriptName) => ({
    scriptName,
    exists: typeof rootScripts[scriptName] === 'string',
    command: rootScripts[scriptName] ?? null,
  }));

  const workflowChecks = await Promise.all(
    REQUIRED_WORKFLOWS.map(async (workflow) => ({
      ...workflow,
      exists: await pathExists(resolve(rootPath, workflow.path)),
    })),
  );

  const docChecks = await Promise.all(
    REQUIRED_DOCS.map(async (doc) => ({
      ...doc,
      exists: await pathExists(resolve(rootPath, doc.path)),
    })),
  );

  return {
    docChecks,
    scriptChecks,
    workflowChecks,
  };
}

function summarizeChecks(entries) {
  const missing = entries.filter((entry) => entry.exists === false);
  return {
    entries,
    missing,
    missingCount: missing.length,
    status: missing.length === 0 ? 'passed' : 'failed',
  };
}

export function createSupportReport({ checks }) {
  const scriptSummary = summarizeChecks(checks.scriptChecks);
  const workflowSummary = summarizeChecks(checks.workflowChecks);
  const docSummary = summarizeChecks(checks.docChecks);
  const hasFailure = [scriptSummary, workflowSummary, docSummary].some((summary) => summary.status === 'failed');

  return {
    createdAt: new Date().toISOString(),
    overallStatus: hasFailure ? 'failed' : 'passed',
    profiles: SUPPORT_MATRIX.profiles,
    tools: SUPPORT_MATRIX.tools,
    scripts: scriptSummary,
    workflows: workflowSummary,
    docs: docSummary,
  };
}

function renderCheckRows(summary, keyName) {
  return summary.entries.map((entry) => {
    const label = keyName === 'scriptName' ? `\`${entry[keyName]}\`` : `\`${entry[keyName]}\``;
    const extra = keyName === 'scriptName'
      ? ` | ${entry.command ?? 'missing'}`
      : ` | ${entry.description}`;
    return `| ${label} | ${entry.exists ? 'present' : 'missing'}${extra} |`;
  });
}

export function renderSupportMarkdownReport(report) {
  const lines = [];
  lines.push('# Svelte support matrix');
  lines.push('');
  lines.push(`- Overall status: **${report.overallStatus}**`);
  lines.push(`- Generated at: ${report.createdAt}`);
  lines.push('');
  lines.push('## Support summary');
  lines.push('');

  for (const tool of report.tools) {
    lines.push(`### ${tool.name}`);
    lines.push('');
    lines.push(`- Support level: **${tool.supportLevel}**`);
    lines.push(`- Implementation: ${tool.implementation}`);
    lines.push(`- CI surfaces: ${tool.ciSurfaces.join(', ')}`);
    lines.push('- Capabilities:');
    for (const capability of tool.capabilities) {
      lines.push(`  - ${capability}`);
    }
    lines.push('- Caveats:');
    for (const caveat of tool.caveats) {
      lines.push(`  - ${caveat}`);
    }
    lines.push('');
  }

  lines.push('## Profiles');
  lines.push('');
  for (const profile of report.profiles) {
    lines.push(`- **${profile.name}** — ${profile.description}`);
  }
  lines.push('');

  lines.push('## Root scripts');
  lines.push('');
  lines.push('| Script | Status | Command |');
  lines.push('| --- | --- | --- |');
  lines.push(...renderCheckRows(report.scripts, 'scriptName'));
  lines.push('');

  lines.push('## Workflows');
  lines.push('');
  lines.push('| Workflow | Status | Description |');
  lines.push('| --- | --- | --- |');
  lines.push(...renderCheckRows(report.workflows, 'path'));
  lines.push('');

  lines.push('## Docs');
  lines.push('');
  lines.push('| Doc | Status | Description |');
  lines.push('| --- | --- | --- |');
  lines.push(...renderCheckRows(report.docs, 'path'));
  lines.push('');

  const failures = [
    ...report.scripts.missing.map((entry) => `Missing root script: ${entry.scriptName}`),
    ...report.workflows.missing.map((entry) => `Missing workflow: ${entry.path}`),
    ...report.docs.missing.map((entry) => `Missing doc: ${entry.path}`),
  ];

  if (failures.length > 0) {
    lines.push('## Missing items');
    lines.push('');
    for (const failure of failures) {
      lines.push(`- ${failure}`);
    }
    lines.push('');
  }

  return `${lines.join('\n').trimEnd()}\n`;
}

export async function writeSupportReportOutputs(options, report) {
  await mkdir(dirname(options.jsonOutputPath), { recursive: true });
  await writeFile(options.jsonOutputPath, `${JSON.stringify(report, null, 2)}\n`);

  const markdown = renderSupportMarkdownReport(report);
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
  let markdownOutputPath = null;
  let strict = false;

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
        'Usage: node ./scripts/report-svelte-support.mjs [--markdown-output <path>] [--json-output <path>] [--append-summary] [--strict]\n',
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    jsonOutputPath: jsonOutputPath ?? getDefaultSupportOutputPath('json'),
    markdownOutputPath: markdownOutputPath ?? getDefaultSupportOutputPath('markdown'),
    strict,
  };
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseCliOptions(argv);
  const checks = await collectSupportChecks();
  const report = createSupportReport({ checks });
  await writeSupportReportOutputs(options, report);

  if (options.strict && report.overallStatus !== 'passed') {
    process.exitCode = 1;
  }

  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
