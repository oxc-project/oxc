import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("static ETS", () => {
  const source = [
    "package example.formatter;",
    "final class Box{value:int=1;method(value:int):int{return value}}",
    "let character:char=c'a';",
  ].join("\n");

  it("requires the explicit language option and formats idempotently", async () => {
    const inferred = await format("test.ets", source);
    expect(inferred.errors.length).toBeGreaterThan(0);
    expect(inferred.code).toBe(source);

    const explicit = await format("test.ets", source, { lang: "ets-static" });
    expect(explicit.errors).toEqual([]);
    expect(explicit.code).toContain("package example.formatter;");
    expect(explicit.code).toContain("final class Box {");
    expect(explicit.code).toContain("value: int = 1;");
    expect(explicit.code).toContain("let character: char = c'a';");

    const second = await format("test.ets", explicit.code, { lang: "ets-static" });
    expect(second.errors).toEqual([]);
    expect(second.code).toBe(explicit.code);
  });
});
