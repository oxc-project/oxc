// oxlint-disable no-console

import { join as pathJoin } from "path";
import type { Reporter, TestModule } from "vitest/node";

const currentDir = pathJoin(import.meta.dirname, "./"),
  rootDir = pathJoin(currentDir, "../../");

export default class CustomReporter implements Reporter {
  onTestRunEnd(testModules: ReadonlyArray<TestModule>): void {
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

function formatMessage(message: string): string {
  return message
    .split("\n")
    .filter(
      (line) =>
        !/[\\/]node_modules[\\/]vitest[\\/]/.test(line)
        && line.trim() !== "at new Promise (<anonymous>)",
    )
    .map((line) =>
      line
        .replace("file://", "")
        .replace(rootDir, "./")
        .replace(
          /(?:[A-Za-z]:)?\.?[\\/](?:[^\s()"'`]*[\\/])?node_modules[\\/]([^\s()"'`]*)/g,
          (_, path: string) => `./node_modules/${path.replaceAll("\\", "/")}`,
        ),
    )
    .join("\n");
}
