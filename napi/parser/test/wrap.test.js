import { describe, expect, it } from "vitest";

import { wrap } from "../src-js/wrap.js";

describe("wrap", () => {
  it("releases every native-owned field eagerly", () => {
    const accessed = [];
    const nativeResult = {
      get program() {
        accessed.push("program");
        return '{"node":{"type":"Program","body":[]},"fixes":[]}';
      },
      get module() {
        accessed.push("module");
        return { hasModuleSyntax: false };
      },
      get comments() {
        accessed.push("comments");
        return [];
      },
      get errors() {
        accessed.push("errors");
        return [];
      },
    };

    const result = wrap(nativeResult);

    expect(accessed).toEqual(["program", "module", "comments", "errors"]);
    expect(result.errors).toEqual([]);
    expect(result.program).toEqual({ type: "Program", body: [] });
    expect(accessed).toEqual(["program", "module", "comments", "errors"]);
  });
});
