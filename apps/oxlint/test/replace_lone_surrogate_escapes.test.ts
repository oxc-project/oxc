import { describe, expect, it } from "vitest";
import { replaceLoneSurrogateEscapes } from "../src-js/utils/utils.ts";

describe("replaceLoneSurrogateEscapes", () => {
  it("replaces a lone high or low surrogate escape with U+FFFD", () => {
    const json = replaceLoneSurrogateEscapes(JSON.stringify({ high: "\ud800", low: "a\udfffb" }));
    expect(json).toBe('{"high":"\\ufffd","low":"a\\ufffdb"}');
    expect(JSON.parse(json)).toEqual({ high: "�", low: "a�b" });
  });

  it("leaves a well-formed surrogate pair alone", () => {
    const json = replaceLoneSurrogateEscapes(JSON.stringify({ emoji: "😀" }));
    expect(JSON.parse(json)).toEqual({ emoji: "😀" });
  });

  it("leaves a literal backslash followed by a surrogate spelling alone", () => {
    const json = replaceLoneSurrogateEscapes(
      JSON.stringify({ text: "\\ud800", mixed: "\\\ud800" }),
    );
    expect(JSON.parse(json)).toEqual({ text: "\\ud800", mixed: "\\�" });
  });

  it("matches the escape case-insensitively", () => {
    expect(replaceLoneSurrogateEscapes('{"a":"\\uD800"}')).toBe('{"a":"\\ufffd"}');
  });
});
