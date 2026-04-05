import { dirname, join, resolve as pathResolve } from "node:path";
import { describe, expect, it } from "vitest";
import replaceGlobalsPlugin from "../tsdown_plugins/replace_globals.ts";

class FakeMagicString {
  #prefix = "";
  readonly #edits: Array<{ start: number; end: number; text: string }> = [];

  constructor(private readonly source: string) {}

  overwrite(start: number, end: number, text: string): void {
    this.#edits.push({ start, end, text });
  }

  prepend(text: string): void {
    this.#prefix = `${text}${this.#prefix}`;
  }

  toString(): string {
    let output = this.source;

    for (const edit of [...this.#edits].sort((a, b) => b.start - a.start)) {
      output = `${output.slice(0, edit.start)}${edit.text}${output.slice(edit.end)}`;
    }

    return `${this.#prefix}${output}`;
  }
}

describe("replace-globals tsdown plugin", () => {
  it("does not prepend duplicate globals imports when the source already imports them", async () => {
    const path = join(import.meta.dirname, "..", "src-js", "js_config.ts");
    const source = [
      'import { JSONStringify } from "./utils/globals.ts";',
      "const payload = JSON.stringify({ answer: 42 });",
      "",
    ].join("\n");

    const magicString = new FakeMagicString(source);
    const result = await replaceGlobalsPlugin.transform!.handler.call(
      {
        async resolve(specifier: string, importer: string) {
          return { id: pathResolve(dirname(importer), specifier) };
        },
      },
      source,
      path,
      { magicString },
    );

    const output = String(result?.code ?? magicString);

    expect(output.match(/import \{ JSONStringify \} from "\.\/utils\/globals\.ts";/g)).toHaveLength(1);
    expect(output).toContain("const payload = JSONStringify({ answer: 42 });");
  });
});
