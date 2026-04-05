import { describe, expect, it } from 'vitest';
import {
  createCombinedReport,
  getDefaultCombinedOutputPath,
  renderCombinedMarkdownReport,
  summarizeToolReport,
} from '../../../scripts/report-svelte-beta.mjs';

describe('svelte beta combined report', () => {
  it('summarizes passed and failed tool reports separately', () => {
    const lintSummary = summarizeToolReport('oxlint', '/tmp/lint-report.json', {
      installPackageJsonMatches: true,
      managedRunState: { laneStatus: 'passed', steps: [] },
      packages: [{ status: 'ok' }],
      links: [{ status: 'ok' }],
      testRootResolutions: [{ error: null, insideInstallNodeModules: true }],
      fixtures: [{ status: 'ok' }],
      npmLs: { exitCode: 0 },
    });

    const formatSummary = summarizeToolReport('oxfmt', '/tmp/format-report.json', {
      installPackageJsonMatches: false,
      managedRunState: {
        laneStatus: 'failed',
        steps: [{ status: 'failed', stepName: 'cli', errorMessage: 'formatter failed' }],
      },
      packages: [{ status: 'missing' }],
      links: [{ status: 'ok' }],
      resolutions: [{ error: 'resolution drift', insideInstallNodeModules: false }],
      npmLs: { exitCode: 1 },
    });

    expect(lintSummary.overallStatus).toBe('passed');
    expect(formatSummary.overallStatus).toBe('failed');
  });

  it('renders a combined markdown summary', () => {
    const report = createCombinedReport({
      profileName: 'latest-svelte',
      lintReportPath: '/tmp/lint-report.json',
      lintReport: {
        installPackageJsonMatches: true,
        managedRunState: { laneStatus: 'passed', steps: [] },
        packages: [{ status: 'ok' }],
        links: [{ status: 'ok' }],
        testRootResolutions: [{ error: null, insideInstallNodeModules: true }],
        fixtures: [{ status: 'ok' }],
        npmLs: { exitCode: 0 },
      },
      formatReportPath: '/tmp/format-report.json',
      formatReport: {
        installPackageJsonMatches: true,
        managedRunState: {
          laneStatus: 'failed',
          steps: [{ status: 'failed', stepName: 'lsp', errorMessage: 'code action mismatch' }],
        },
        packages: [{ status: 'ok' }],
        links: [{ status: 'ok' }],
        resolutions: [{ error: null, insideInstallNodeModules: true }],
        npmLs: { exitCode: 0 },
      },
      selectedToolNames: ['oxlint', 'oxfmt'],
    });

    expect(report.overallStatus).toBe('failed');
    expect(getDefaultCombinedOutputPath('latest-svelte', 'markdown')).toContain('.svelte-beta/report.latest-svelte.md');

    const markdown = renderCombinedMarkdownReport(report);
    expect(markdown).toContain('# Svelte beta report (latest-svelte)');
    expect(markdown).toContain('## Oxlint');
    expect(markdown).toContain('## Oxfmt');
    expect(markdown).toContain('code action mismatch');
  });
});
