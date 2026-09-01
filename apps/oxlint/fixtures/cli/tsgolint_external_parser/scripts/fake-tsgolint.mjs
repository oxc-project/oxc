// Stand-in for the tsgolint binary.
//
// Records the payload oxlint sends on stdin, then replies with one rule diagnostic per
// `.gts` file in that payload, anchored at the byte offset of `label.toUpperCase` inside
// the file's `<template>` block. This exercises the whole oxlint side of the handoff --
// path selection, payload construction, diagnostic decoding, and source rendering -- with
// no dependency on tsgolint's own content-mapper support.
//
// See README.md in the parent directory.
import { readFileSync, writeFileSync } from "node:fs";

const capturePath = process.env.FAKE_TSGOLINT_CAPTURE;

const stdin = readFileSync(0, "utf8");
if (capturePath) writeFileSync(capturePath, stdin);

const payload = JSON.parse(stdin);

function frame(kind, obj) {
  const body = Buffer.from(JSON.stringify(obj), "utf8");
  const header = Buffer.alloc(5);
  header.writeUInt32LE(body.length, 0);
  header.writeUInt8(kind, 4);
  return Buffer.concat([header, body]);
}

const out = [];
for (const config of payload.configs ?? []) {
  for (const filePath of config.file_paths ?? []) {
    if (!filePath.endsWith(".gts")) continue;

    const source =
      payload.source_overrides?.[filePath] ?? readFileSync(filePath, "utf8");
    // Offset of the erroneous expression in the ORIGINAL .gts, not in any mapped output.
    const pos = Buffer.byteLength(
      source.slice(0, source.indexOf("label.toUpperCase")),
      "utf8",
    );
    const end = pos + Buffer.byteLength("label.toUpperCase", "utf8");

    // Under `--type-check` oxlint asks for TS's own diagnostics, which arrive as
    // `kind: 1` (internal) rather than as a rule diagnostic.
    const kind = payload.report_semantic ? 1 : 0;
    out.push(
      frame(1, {
        kind,
        rule: kind === 0 ? "no-floating-promises" : undefined,
        range: { pos, end },
        message: {
          id: "fakeTypeError",
          description: "Property 'toUpperCase' does not exist on type 'number'.",
          help: null,
        },
        file_path: filePath,
      }),
    );
  }
}

process.stdout.write(Buffer.concat(out));
