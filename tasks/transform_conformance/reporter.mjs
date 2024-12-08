import { join as pathJoin } from 'path';
import { JsonReporter } from 'vitest/reporters';

const currentDir = pathJoin(import.meta.dirname, './'),
  rootDir = pathJoin(currentDir, '../../'),
  vitestPath = pathJoin(rootDir, 'node_modules/.pnpm/@vitest+runner@');

export default class CustomReporter extends JsonReporter {
  async writeReport(report) {
    const { testResults, numPassedTestSuites, numTotalTestSuites } = JSON.parse(report);

    const percentPassed = (numPassedTestSuites * 100 / numTotalTestSuites).toFixed(2);
    console.log(`\nPassed: ${numPassedTestSuites} of ${numTotalTestSuites} (${percentPassed}%)`);

    if (numPassedTestSuites === numTotalTestSuites) return;

    console.log('\nFailures:');

    for (const testResult of testResults) {
      if (testResult.status === 'passed') continue;

      const name = testResult.name.replace(currentDir, './');
      const message = testResult.message ||
        testResult.assertionResults.flatMap(result => result.failureMessages.map(formatMessage)).join('\n');
      console.log();
      console.log(name);
      console.log(message);
    }
  }
}

function formatMessage(message) {
  const lines = message.split('\n');
  const index = lines.findIndex(line => line.includes(vitestPath));
  if (index !== -1) lines.length = index;
  return lines.map(line => line.replace('file://', '').replace(rootDir, './')).join('\n');
}
