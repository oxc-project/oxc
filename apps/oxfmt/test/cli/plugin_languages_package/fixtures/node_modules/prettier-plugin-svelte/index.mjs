const DEFAULT_SORT_ORDER = "markup-scripts-styles";

function extractBlock(source, tagName) {
  const regex = new RegExp(`<${tagName}(?:\\s[^>]*)?>[\\s\\S]*?<\\/${tagName}>`, "i");
  const match = regex.exec(source);
  if (!match) return null;

  const start = match.index ?? 0;
  return {
    raw: match[0],
    start,
    end: start + match[0].length,
  };
}

function normalizeBlock(block, tagName) {
  if (!block) return null;

  const openTagMatch = block.raw.match(new RegExp(`^<${tagName}(?:\\s[^>]*)?>`, "i"));
  const openTag = openTagMatch?.[0] ?? `<${tagName}>`;
  const inner = block.raw
    .replace(new RegExp(`^<${tagName}(?:\\s[^>]*)?>`, "i"), "")
    .replace(new RegExp(`<\\/${tagName}>$`, "i"), "")
    .trim();

  return `${openTag}\n${inner}\n</${tagName}>`;
}

function formatSvelte(source, sortOrder) {
  const script = extractBlock(source, "script");
  const style = extractBlock(source, "style");

  let markup = source;
  for (const block of [script, style].filter(Boolean).sort((a, b) => b.start - a.start)) {
    markup = `${markup.slice(0, block.start)}${markup.slice(block.end)}`;
  }
  markup = markup.trim();

  const sections = {
    markup,
    scripts: normalizeBlock(script, "script"),
    styles: normalizeBlock(style, "style"),
  };

  return sortOrder
    .split("-")
    .map((part) => sections[part])
    .filter(Boolean)
    .join("\n\n") + "\n";
}

export const options = {
  svelteSortOrder: {
    category: "Svelte",
    type: "choice",
    default: DEFAULT_SORT_ORDER,
    description: "Test-only option for ordering top-level Svelte blocks.",
    choices: [
      { value: DEFAULT_SORT_ORDER, description: "Keep markup before script/style." },
      { value: "scripts-markup-styles", description: "Move script before markup/style." },
    ],
  },
};

export const languages = [
  {
    name: "Svelte",
    parsers: ["svelte"],
    extensions: [".svelte"],
  },
];

export const parsers = {
  svelte: {
    parse(text) {
      return { type: "SvelteDocument", text };
    },
    astFormat: "svelte-ast",
    locStart: () => 0,
    locEnd: (node) => node.text.length,
  },
};

export const printers = {
  "svelte-ast": {
    print(path, options) {
      const source = path.getValue().text;
      return formatSvelte(source, options.svelteSortOrder ?? DEFAULT_SORT_ORDER);
    },
  },
};

export default {
  languages,
  options,
  parsers,
  printers,
};
