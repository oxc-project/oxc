import { describe, expect, it } from 'vitest';
import {
  REQUIRED_DOCS,
  REQUIRED_ROOT_SCRIPTS,
  REQUIRED_WORKFLOWS,
  SUPPORT_MATRIX,
  createSupportReport,
  getDefaultSupportOutputPath,
  renderSupportMarkdownReport,
} from '../../../scripts/report-svelte-support.mjs';

describe('svelte support matrix report', () => {
  it('keeps the expected support surface explicit', () => {
    expect(REQUIRED_ROOT_SCRIPTS).toEqual(
      expect.arrayContaining([
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
      ]),
    );

    expect(REQUIRED_WORKFLOWS.map((entry) => entry.path)).toEqual(
      expect.arrayContaining([
        '.github/workflows/svelte-beta-validation.yml',
        '.github/workflows/ci.yml',
        '.github/workflows/oxlint-svelte-upstream-canary.yml',
        '.github/workflows/oxfmt-svelte-upstream-canary.yml',
      ]),
    );

    expect(REQUIRED_DOCS.map((entry) => entry.path)).toEqual(
      expect.arrayContaining([
        'SVELTE_SUPPORT.md',
        'SVELTE_BETA_SOAK.md',
        'SVELTE_ANNOUNCEMENT_TEMPLATE.md',
        'SVELTE_RELEASE_CHECKLIST.md',
      ]),
    );

    expect(SUPPORT_MATRIX.tools.map((tool) => tool.name)).toEqual(['oxlint', 'oxfmt']);
  });

  it('renders missing scripts, workflows, and docs in the markdown output', () => {
    const report = createSupportReport({
      checks: {
        scriptChecks: [
          {
            scriptName: 'test:svelte-beta',
            exists: true,
            command: 'node ./scripts/run-svelte-beta.mjs',
          },
          {
            scriptName: 'report:svelte-support',
            exists: false,
            command: null,
          },
        ],
        workflowChecks: [
          {
            category: 'combined-beta',
            description: 'Manual combined beta validation workflow',
            exists: false,
            path: '.github/workflows/svelte-beta-validation.yml',
          },
        ],
        docChecks: [
          {
            category: 'status-guide',
            description: 'Maintainer-facing Svelte support guide',
            exists: false,
            path: 'SVELTE_SUPPORT.md',
          },
        ],
      },
    });

    expect(report.overallStatus).toBe('failed');
    expect(getDefaultSupportOutputPath('markdown')).toContain('.svelte-beta/support-matrix.md');

    const markdown = renderSupportMarkdownReport(report);
    expect(markdown).toContain('# Svelte support matrix');
    expect(markdown).toContain('Missing root script: report:svelte-support');
    expect(markdown).toContain('Missing workflow: .github/workflows/svelte-beta-validation.yml');
    expect(markdown).toContain('Missing doc: SVELTE_SUPPORT.md');
    expect(markdown).toContain('## Root scripts');
    expect(markdown).toContain('## Workflows');
    expect(markdown).toContain('## Docs');
  });
});
