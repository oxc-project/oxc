import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Line ending handling", () => {
  // Some entries parse a newline-normalized copy, but error labels must point into the original source
  it.each([
    ["a.css", "a {}\nb {"],
    ["a.css", "\uFEFFa {}\nb {"],
    ["a.yaml", "a: 1\nb: ["],
  ])("reports error labels in %s against the CRLF source", async (filename, source) => {
    const lf = await format(filename, source);
    const crlf = await format(filename, source.replaceAll("\n", "\r\n"));
    // Each input has 1 line break before the error
    expect(crlf.errors[0].labels).toStrictEqual(
      // oxlint-disable no-map-spread
      lf.errors[0].labels.map((l) => ({ ...l, start: l.start + 1, end: l.end + 1 })),
    );
  });
});
