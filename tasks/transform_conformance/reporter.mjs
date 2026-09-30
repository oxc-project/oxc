// oxlint-disable no-console

import { join as pathJoin } from "path";

const currentDir = pathJoin(import.meta.dirname, "./"),
  rootDir = pathJoin(currentDir, "../../"),
  vitestPath = pathJoin(rootDir, "node_modules/.pnpm/vitest@");

export default class CustomReporter {
  onTestRunEnd(testModules) {
    const numTotalTestSuites = testModules.length;
    const numPassedTestSuites = testModules.filter(
      (testModule) => testModule.state() === "passed",
    ).length;

    const percentPassed = ((numPassedTestSuites * 100) / numTotalTestSuites).toFixed(2);
    console.log(`\nPassed: ${numPassedTestSuites} of ${numTotalTestSuites} (${percentPassed}%)`);

    if (numPassedTestSuites === numTotalTestSuites) return;

    console.log("\nFailures:");

    for (const testModule of testModules) {
      if (testModule.state() !== "failed") continue;

      const name = testModule.moduleId.replace(currentDir, "./");
      const moduleErrors = testModule.errors();
      const message =
        moduleErrors.length > 0
          ? moduleErrors.map((error) => formatMessage(error.stack ?? error.message)).join("\n")
          : [...testModule.children.allTests("failed")]
              .flatMap((test) => test.result().errors ?? [])
              .map((error) => formatMessage(error.stack ?? error.message))
              .join("\n");
      console.log();
      console.log(name);
      console.log(message);
    }
  }
}

function formatMessage(message) {
  return message
    .split("\n")
    .filter((line) => !line.includes(vitestPath) && line.trim() !== "at new Promise (<anonymous>)")
    .map((line) => line.replace("file://", "").replace(rootDir, "./"))
    .join("\n");
}
