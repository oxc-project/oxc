// oxlint-disable no-console -- CLI prints its report to stdout.

// Compare the nodes Oxc assigns comments to with the nodes Babel attaches them to.
//
// Usage: node scripts/compare-babel-comments.js [--json <file>] [paths...]
//
// `paths` are files or directories, searched recursively. Defaults to `scripts/fixtures/babel-comments`.
// Each comment's attachments are normalized to edges `<placement> <node type>[<start>,<end>]`,
// and the edge sets from Oxc and Babel are compared. Babel can attach a comment to several nodes.
// Build oxc-parser first with `pnpm run build-dev`.

import { parse as babelParse } from "@babel/parser";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { parseSync } from "../src-js/index.js";

const EXTENSIONS = new Set([".js", ".mjs", ".cjs", ".jsx", ".ts", ".mts", ".cts", ".tsx"]);
const DEFAULT_PATH = join(import.meta.dirname, "fixtures/babel-comments");
const SKIPPED_KEYS = new Set([
  "comments",
  "leadingComments",
  "trailingComments",
  "innerComments",
  "loc",
  "range",
  "extra",
  "tokens",
  "errors",
]);
const BABEL_PLACEMENTS = [
  ["leadingComments", "leading"],
  ["trailingComments", "trailing"],
  ["innerComments", "dangling"],
];
const OXC_PLACEMENTS = ["leading", "trailing", "dangling"];

function main() {
  const { jsonPath, paths } = parseArgs(process.argv.slice(2));
  const files = [];
  for (const path of paths.length === 0 ? [DEFAULT_PATH] : paths) {
    collectFiles(resolve(path), files);
  }
  files.sort(compareStrings);

  const report = {
    summary: {
      files: files.length,
      failedFiles: 0,
      comments: 0,
      matching: 0,
      different: 0,
      babelMultiple: 0,
      oxcUnexposed: 0,
      babelUnattached: 0,
    },
    files: [],
  };

  for (const path of files) {
    const fileReport = compareFile(path, relative(process.cwd(), path));
    report.files.push(fileReport);
    if (fileReport.error !== null) {
      report.summary.failedFiles++;
      continue;
    }
    for (const comment of fileReport.comments) {
      report.summary.comments++;
      if (comment.matches) {
        report.summary.matching++;
      } else {
        report.summary.different++;
      }
      if (comment.babel.length > 1) report.summary.babelMultiple++;
      if (comment.oxc.length === 0) report.summary.oxcUnexposed++;
      if (comment.babel.length === 0) report.summary.babelUnattached++;
    }
  }

  printReport(report);
  if (jsonPath !== null) writeFileSync(jsonPath, JSON.stringify(report, null, 2) + "\n");
}

function parseArgs(args) {
  let jsonPath = null;
  const paths = [];
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--json") {
      jsonPath = args[++index];
      if (jsonPath === undefined) usage("`--json` requires a file path");
    } else if (arg === "--help" || arg === "-h") {
      usage(null);
    } else if (arg.startsWith("-")) {
      usage(`Unknown option \`${arg}\``);
    } else {
      paths.push(arg);
    }
  }
  return { jsonPath, paths };
}

function usage(message) {
  const text = "Usage: node scripts/compare-babel-comments.js [--json <file>] [paths...]";
  if (message === null) {
    console.log(text);
    process.exit(0);
  }
  console.error(`${message}\n${text}`);
  process.exit(1);
}

function collectFiles(path, files) {
  if (statSync(path).isDirectory()) {
    for (const entry of readdirSync(path)) {
      if (entry !== "node_modules") collectFiles(join(path, entry), files);
    }
  } else if (EXTENSIONS.has(extname(path))) {
    files.push(path);
  }
}

function compareFile(path, displayPath) {
  const sourceText = readFileSync(path, "utf8");
  const fileReport = { path: displayPath, error: null, comments: [] };

  let oxc, babel;
  try {
    oxc = parseWithOxc(path, sourceText);
  } catch (err) {
    fileReport.error = `Oxc: ${err.message}`;
    return fileReport;
  }
  try {
    babel = parseWithBabel(path, sourceText);
  } catch (err) {
    fileReport.error = `Babel: ${err.message}`;
    return fileReport;
  }

  const comments = new Map();
  for (const comment of [...oxc.comments, ...babel.comments]) {
    const key = commentKey(comment);
    if (!comments.has(key)) {
      comments.set(key, {
        start: comment.start,
        end: comment.end,
        text: sourceText.slice(comment.start, comment.end),
        oxc: [],
        babel: [],
      });
    }
  }
  for (const [key, edges] of oxc.edges) comments.get(key)?.oxc.push(...edges);
  for (const [key, edges] of babel.edges) comments.get(key)?.babel.push(...edges);

  for (const comment of [...comments.values()].sort((a, b) => a.start - b.start)) {
    comment.oxc.sort(compareStrings);
    comment.babel.sort(compareStrings);
    comment.matches =
      comment.oxc.length === comment.babel.length &&
      comment.oxc.every((edge, index) => edge === comment.babel[index]);
    fileReport.comments.push(comment);
  }
  return fileReport;
}

function parseWithOxc(path, sourceText) {
  const ret = parseSync(path, sourceText, {
    sourceType: "module",
    preserveParens: false,
    experimentalNodeComments: true,
  });
  if (ret.errors.length > 0) throw new Error(ret.errors[0].message);

  const edges = new Map();
  walk(ret.program, (node) => {
    const nodeComments = node.comments;
    if (nodeComments === null || nodeComments === undefined) return;
    for (const placement of OXC_PLACEMENTS) {
      for (const comment of nodeComments[placement] ?? []) {
        addEdge(edges, comment, placement, node);
      }
    }
  });

  const comments = ret.comments.filter((comment) => !sourceText.startsWith("#!", comment.start));
  return { comments, edges };
}

function parseWithBabel(path, sourceText) {
  const extension = extname(path);
  const isTs = [".ts", ".mts", ".cts", ".tsx"].includes(extension);
  const plugins = ["estree", "decorators", "sourcePhaseImports"];
  if (extension !== ".ts" && extension !== ".mts" && extension !== ".cts") plugins.push("jsx");
  if (isTs) plugins.push("typescript");

  const ast = babelParse(sourceText, { sourceType: "module", plugins });

  const edges = new Map();
  walk(ast, (node) => {
    for (const [key, placement] of BABEL_PLACEMENTS) {
      for (const comment of node[key] ?? []) {
        addEdge(edges, comment, placement, node);
      }
    }
  });
  return { comments: ast.comments, edges };
}

function walk(node, visit) {
  if (node === null || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const child of node) walk(child, visit);
    return;
  }
  if (typeof node.type === "string" && typeof node.start === "number") visit(node);
  for (const key in node) {
    if (!SKIPPED_KEYS.has(key)) walk(node[key], visit);
  }
}

function addEdge(edges, comment, placement, node) {
  const key = commentKey(comment);
  let commentEdges = edges.get(key);
  if (commentEdges === undefined) {
    commentEdges = [];
    edges.set(key, commentEdges);
  }
  commentEdges.push(`${placement} ${nodeKey(node)}`);
}

function commentKey(comment) {
  return `${comment.start}-${comment.end}`;
}

function compareStrings(a, b) {
  if (a < b) return -1;
  return a > b ? 1 : 0;
}

function nodeKey(node) {
  if (node.type === "Program" || node.type === "File") return node.type;
  return `${node.type}[${node.start},${node.end}]`;
}

function printReport(report) {
  for (const file of report.files) {
    if (file.error !== null) {
      console.log(`## ${file.path}\n  parse error: ${file.error}\n`);
      continue;
    }
    const different = file.comments.filter((comment) => !comment.matches);
    if (different.length === 0) continue;
    console.log(`## ${file.path}`);
    for (const comment of different) {
      console.log(`  ${formatText(comment.text)} [${comment.start},${comment.end}]`);
      console.log(`    oxc:   ${formatEdges(comment.oxc)}`);
      console.log(`    babel: ${formatEdges(comment.babel)}`);
    }
    console.log();
  }

  const { summary } = report;
  console.log("## Summary");
  console.log(`  files:            ${summary.files} (${summary.failedFiles} failed to parse)`);
  console.log(`  comments:         ${summary.comments}`);
  console.log(`  matching:         ${summary.matching}`);
  console.log(`  different:        ${summary.different}`);
  console.log(`  babel multiple:   ${summary.babelMultiple}`);
  console.log(`  oxc unexposed:    ${summary.oxcUnexposed}`);
  console.log(`  babel unattached: ${summary.babelUnattached}`);
}

function formatText(text) {
  const escaped = JSON.stringify(text).slice(1, -1);
  return escaped.length > 60 ? `${escaped.slice(0, 57)}...` : escaped;
}

function formatEdges(edges) {
  return edges.length === 0 ? "(none)" : edges.join(", ");
}

main();
