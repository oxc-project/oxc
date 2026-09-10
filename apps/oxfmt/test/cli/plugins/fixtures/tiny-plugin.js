// A minimal Prettier plugin, kept local so the fixture needs no extra dependency.
//
// It imports `prettier` the way published plugins do, which is what exercises
// the specifier redirect in `libs/plugin-resolution.ts`.
import { doc } from "prettier";

export const languages = [
  { name: "Tiny", parsers: ["tiny"], extensions: [".tiny"], filenames: ["TINYFILE"] },
];

export const parsers = {
  tiny: {
    parse: (text) => ({ type: "root", words: text.split(/\s+/).filter(Boolean) }),
    astFormat: "tiny-ast",
    locStart: () => 0,
    locEnd: () => 0,
  },
};

export const printers = {
  "tiny-ast": {
    print: (path) => [doc.builders.join(doc.builders.line, path.node.words), doc.builders.hardline],
  },
};
