import { JsonReporter } from 'vitest/reporters';

export default class CustomReporter extends JsonReporter {
  async writeReport(report) {
    const json = JSON.parse(report);
    console.log();
    for (const testResult of json.testResults) {
      if (testResult.status !== 'failed') {
        continue;
      }
      const message = testResult.message;
      if (!message) {
        continue;
      }
      const name = testResult.name.replace(import.meta.dirname, '.');
      console.log(name);
      console.log(message);
      console.log();
    }
  }
}
