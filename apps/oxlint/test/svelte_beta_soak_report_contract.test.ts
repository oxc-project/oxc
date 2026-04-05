import { describe, expect, it } from 'vitest';
import {
  createBetaSoakReport,
  getDefaultSoakOutputPath,
  renderBetaSoakMarkdownReport,
} from '../../../scripts/report-svelte-beta-soak.mjs';

describe('svelte beta soak report', () => {
  it('renders repo-level pass/fail summaries', () => {
    const report = createBetaSoakReport({
      manifestPath: '/tmp/svelte-beta-repos.json',
      repoRuns: [
        {
          name: 'sveltejs/svelte',
          rootPath: '/tmp/svelte',
          status: 'passed',
          toolRuns: [
            {
              toolName: 'oxlint',
              enabled: true,
              status: 'passed',
              exitCode: 0,
              durationMs: 1200,
              commandDisplay: 'node apps/oxlint/dist/cli.js . --type-aware',
              cwd: '/tmp/svelte',
            },
            {
              toolName: 'oxfmt',
              enabled: true,
              status: 'passed',
              exitCode: 0,
              durationMs: 900,
              commandDisplay: 'node apps/oxfmt/dist/cli.js --check .',
              cwd: '/tmp/svelte',
            },
          ],
        },
        {
          name: 'sveltejs/kit',
          rootPath: '/tmp/kit',
          status: 'failed',
          toolRuns: [
            {
              toolName: 'oxlint',
              enabled: true,
              status: 'failed',
              exitCode: 1,
              durationMs: 800,
              commandDisplay: 'node apps/oxlint/dist/cli.js packages --type-aware',
              cwd: '/tmp/kit',
              stdoutPath: '/tmp/logs/kit/oxlint.stdout.log',
              stderrPath: '/tmp/logs/kit/oxlint.stderr.log',
              errorMessage: 'lint failed',
            },
          ],
        },
      ],
      selectedToolNames: ['oxlint', 'oxfmt'],
    });

    expect(report.overallStatus).toBe('failed');
    expect(report.failedRepoCount).toBe(1);
    expect(getDefaultSoakOutputPath('markdown')).toContain('.svelte-beta-soak/report.md');

    const markdown = renderBetaSoakMarkdownReport(report);
    expect(markdown).toContain('# Svelte beta soak report');
    expect(markdown).toContain('sveltejs/svelte');
    expect(markdown).toContain('sveltejs/kit');
    expect(markdown).toContain('## Failures');
    expect(markdown).toContain('lint failed');
  });
});
