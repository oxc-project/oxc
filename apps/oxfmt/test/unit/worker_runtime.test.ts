import { describe, expect, it } from "vitest";
import { hasWorkerTeardownBug } from "../../src-js/cli/worker-runtime";

describe("hasWorkerTeardownBug()", () => {
  it.each(["24.2.0", "24.18.0", "24.21.5", "25.0.0", "25.9.0", "26.0.0", "26.9.3"])(
    "is true for affected Node.js %s",
    (version) => {
      expect(hasWorkerTeardownBug(version)).toBe(true);
    },
  );

  it.each(["20.19.0", "22.12.0", "22.23.5", "24.1.0", "24.22.0", "24.22.3", "26.10.0", "27.0.0"])(
    "is false for Node.js %s",
    (version) => {
      expect(hasWorkerTeardownBug(version)).toBe(false);
    },
  );
});
