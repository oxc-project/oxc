import { doc } from "prettier";
import { jsTextToDoc } from "../../index";
import type { Parser, Doc } from "prettier";

const { hardline, join } = doc.builders;
const LINE_BREAK_RE = /\r?\n/;

export const textToDoc: Parser<Doc>["parse"] = async (embeddedSourceText, textToDocOptions) => {
  // NOTE: For (j|t)s-in-xxx, default `parser` is either `babel` or `typescript`
  // In case of ts-in-md, `filepath` is overridden to distinguish TSX or TS
  // We need to infer `SourceType::from_path(fileName)` for `oxc_formatter`.
  const { parser, parentParser, filepath, _oxfmtPluginOptionsJson } = textToDocOptions;
  const fileName =
    parser === "typescript"
      ? filepath.endsWith(".tsx")
        ? "dummy.tsx" // tsx-in-md
        : "dummy.ts" // ts-in-md / ts-in-xxx
      : "dummy.jsx"; // Otherwise, always enable JSX for js-in-xxx, it's safe

  const { doc: formattedText, errors } = await jsTextToDoc(
    fileName,
    embeddedSourceText,
    _oxfmtPluginOptionsJson as string,
    [parentParser].join(":"),
  );

  if (0 < errors.length) throw new Error(errors[0].message);

  // NOTE: This is required for the parent ((j|t)s-in-xxx) printer
  // to handle line breaks correctly,
  // not only for `options.vueIndentScriptAndStyle` but also for basic printing.
  // TODO: Will be handled in Rust, convert our IR to Prettier's Doc directly.
  return join(hardline, formattedText.split(LINE_BREAK_RE));
};
