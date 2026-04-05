import { describe, expect, it } from 'vitest';
import {
  createReleaseReadinessReport,
  getDefaultReadinessOutputPath,
  renderReleaseReadinessMarkdown,
} from '../../../scripts/report-svelte-release-readiness.mjs';

describe('svelte release readiness report', () => {
  it('holds the announcement when the pinned lane fails', () => {
    const report = createReleaseReadinessReport({
      supportOutputPath: '/tmp/support.json',
      supportReport: { overallStatus: 'passed' },
      pinnedReportPath: '/tmp/pinned.json',
      pinnedBetaReport: {
        overallStatus: 'failed',
        tools: {
          oxlint: { overallStatus: 'failed', laneStatus: 'failed', failedSteps: [{ stepName: 'smoke' }] },
          oxfmt: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
        },
      },
      latestReportPath: '/tmp/latest.json',
      latestBetaReport: {
        overallStatus: 'passed',
        tools: {
          oxlint: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
          oxfmt: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
        },
      },
      soakReportPath: '/tmp/soak.json',
      soakBetaReport: {
        overallStatus: 'passed',
        repoCount: 2,
        failedRepoCount: 0,
        repoSummaries: [],
      },
    });

    expect(report.announcementStatus).toBe('hold');

    const markdown = renderReleaseReadinessMarkdown(report);
    expect(markdown).toContain('# Svelte release readiness');
    expect(markdown).toContain('Announcement status: **hold**');
    expect(markdown).toContain('Pinned combined beta lane is green');
    expect(markdown).toContain('## pinned');
    expect(markdown).toContain('## latest-svelte');
  });

  it('marks the rollout ready once support, pinned, latest, and soak are all green', () => {
    const report = createReleaseReadinessReport({
      supportOutputPath: '/tmp/support.json',
      supportReport: { overallStatus: 'passed' },
      pinnedReportPath: '/tmp/pinned.json',
      pinnedBetaReport: {
        overallStatus: 'passed',
        tools: {
          oxlint: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
          oxfmt: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
        },
      },
      latestReportPath: '/tmp/latest.json',
      latestBetaReport: {
        overallStatus: 'passed',
        tools: {
          oxlint: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
          oxfmt: { overallStatus: 'passed', laneStatus: 'passed', failedSteps: [] },
        },
      },
      soakReportPath: '/tmp/soak.json',
      soakBetaReport: {
        overallStatus: 'passed',
        repoCount: 3,
        failedRepoCount: 0,
        repoSummaries: [
          { name: 'repo-a', status: 'passed' },
          { name: 'repo-b', status: 'passed' },
          { name: 'repo-c', status: 'passed' },
        ],
      },
    });

    expect(report.announcementStatus).toBe('ready-for-beta-announcement');
    expect(getDefaultReadinessOutputPath('markdown')).toContain('.svelte-beta/readiness.md');

    const markdown = renderReleaseReadinessMarkdown(report);
    expect(markdown).toContain('Announcement status: **ready-for-beta-announcement**');
    expect(markdown).toContain('Publish the beta announcement');
    expect(markdown).toContain('Source report:');
  });
});
