import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import type { CodeAction, Command } from "vscode-languageserver-protocol/node";
import { TextDocument } from "vscode-languageserver-textdocument";
import { createLspConnection, FULL_DOCUMENT_RANGE, PULL_DIAGNOSTICS_CAPABILITY } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");
// This file does not exist on disk: the server only works on the text held in memory.
const FILE_URI = pathToFileURL(join(FIXTURES_DIR, "test.ts")).href;
const OPENED_TEXT = "if (foo == NaN) {}\n";
const EDITED_TEXT = "if (bar == NaN) {}\n";
const FIXED_TEXT = "if (isNaN(bar)) {}\n";

// The client opens a document, edits it, then asks for the code actions of the edited text.
async function codeActionAfterEdit(only: string): Promise<(Command | CodeAction)[] | null> {
  await using connection = createLspConnection();
  await connection.initialize(
    [{ uri: pathToFileURL(FIXTURES_DIR).href, name: "workspace-0" }],
    // Push mode would lint on `didChange` and fill the cache.
    PULL_DIAGNOSTICS_CAPABILITY,
  );
  await connection.didOpen(FILE_URI, "typescript", OPENED_TEXT);
  await connection.didChange(FILE_URI, EDITED_TEXT);
  return await connection.codeAction(FILE_URI, FULL_DOCUMENT_RANGE, {
    diagnostics: [],
    only: [only],
  });
}

function applyEdits(codeActions: (Command | CodeAction)[] | null): string[] {
  return (codeActions ?? []).map((action) => {
    const textEdits = "edit" in action ? (action.edit?.changes?.[FILE_URI] ?? []) : [];
    return TextDocument.applyEdits(
      TextDocument.create(FILE_URI, "typescript", 2, EDITED_TEXT),
      textEdits,
    );
  });
}

describe("LSP code actions after an edit", () => {
  it.each(["source.fixAll.oxc", "source.fixAll"])(
    "should apply `%s` to the edited text without a diagnostic pull",
    async (kind) => {
      expect(applyEdits(await codeActionAfterEdit(kind))).toStrictEqual([FIXED_TEXT]);
    },
  );

  it("should not lint on demand for a quickfix request after an edit", async () => {
    expect(await codeActionAfterEdit("quickfix")).toBeNull();
  });
});
