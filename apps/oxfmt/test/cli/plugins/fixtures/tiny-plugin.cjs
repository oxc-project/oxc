// CommonJS counterpart of `tiny-plugin.js`.
//
// Plugins ship in both module systems and each reaches Prettier by a different
// route, so the redirect needs covering twice. This one uses `require`.
const { doc } = require("prettier");

exports.languages = [{ name: "TinyCjs", parsers: ["tiny-cjs"], extensions: [".tinycjs"] }];

exports.parsers = {
  "tiny-cjs": {
    parse: (text) => ({ type: "root", words: text.split(/\s+/).filter(Boolean) }),
    astFormat: "tiny-cjs-ast",
    locStart: () => 0,
    locEnd: () => 0,
  },
};

exports.printers = {
  "tiny-cjs-ast": {
    print: (path) => [
      doc.builders.join(doc.builders.hardline, path.node.words),
      doc.builders.hardline,
    ],
  },
};
