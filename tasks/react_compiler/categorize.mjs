/**
 * Explains why two outputs differ, by finding which known-cosmetic
 * normalizations have to be applied before they compare equal.
 *
 * Each normalization stands for one class of difference. A file is described by
 * the smallest set of them that reconciles the two outputs, so a mismatch caused
 * by both dropped comments and renamed bindings is reported as both rather than
 * as whichever happened to be checked first. A file that no combination
 * reconciles is structural: the two pipelines emitted genuinely different code.
 */

/** React Compiler's generated temporaries, which the two may number apart. */
const GENERATED_TEMPORARY = /\b(_temp|t)\d+\b/g;
// `$` is not a word character, so these delimit identifiers with lookarounds
// rather than `\b`, which would not fire on names like `$_0`.
/** The `_1`, `_2`, … suffix React Compiler adds when it renames a binding. */
const RENAME_SUFFIX = /(?<![\w$])([A-Za-z_$][\w$]*)_\d+(?![\w$])/g;
/** An object property whose key and value are the same name, as in `a: a`. */
const REDUNDANT_PROPERTY_KEY = /(?<![\w$])([A-Za-z_$][\w$]*)\s*:\s*\1(?![\w$])/g;
/** Every `_c(n)` call, one per function React Compiler chose to memoize. */
const MEMO_CACHE_CALL = /\b_c\((\d+)\)/g;
/** Keywords after which a `/` opens a regular expression, never a division. */
const REGEXP_PRECEDING_KEYWORD =
  /\b(return|typeof|instanceof|in|of|new|delete|void|do|else|yield|await|case)$/;

/** A generated top-level import, split into its clause and its module. */
const IMPORT_STATEMENT = /^import\s+(.+?)\s+from\s+("[^"]*");$/;

const NORMALIZATIONS = [
  ["comments", (code) => dropBlankLines(stripComments(code))],
  ["empty-statements", (code) => dropLines(code, (line) => line.trim() === ";")],
  ["unused-imports", elideUnusedImports],
  ["import-layout", canonicalizeImports],
  // Collapses runs of whitespace and drops it next to punctuation, so removing
  // a comment from inside a call no longer counts as a difference in its own
  // right just because the remaining arguments fit on one line.
  [
    "whitespace",
    (code) =>
      code
        .replace(/\s+/g, " ")
        .replace(/ *([^\w$ ]) */g, "$1")
        .trim(),
  ],
  ["temporaries", (code) => code.replace(GENERATED_TEMPORARY, "$1")],
  ["renames", (code) => code.replace(RENAME_SUFFIX, "$1")],
  // Renaming a binding forces a shorthand property to be written out in full,
  // so collapse `a: a` back to `a` once the suffixes are gone.
  ["shorthand", (code) => code.replace(REDUNDANT_PROPERTY_KEY, "$1")],
  // Parentheses the two disagree about are always redundant ones: anything
  // load-bearing would have changed the surrounding text as well. This runs
  // last because dropping them fuses tokens, and the identifier rules above
  // match on token boundaries.
  ["parentheses", (code) => code.replace(/[()]/g, "")],
];

/**
 * Describes a mismatch as a category plus the evidence behind it.
 *
 * `causes` is the minimal set of normalizations that reconciles the outputs;
 * `memoizedFunctions` counts how many functions each side compiled, which
 * separates a formatting difference from one side bailing out of compilation.
 */
export function categorizeDifference(babelOutput, oxcOutput) {
  const causes = explainDifference(babelOutput, oxcOutput);
  const babelCaches = memoCaches(babelOutput);
  const oxcCaches = memoCaches(oxcOutput);
  const memoizedFunctions = { babel: babelCaches.length, oxc: oxcCaches.length };
  const memoizedSlots = { babel: sum(babelCaches), oxc: sum(oxcCaches) };

  let category;
  if (causes.length > 0) {
    category = causes.join("+");
  } else if (memoizedFunctions.babel !== memoizedFunctions.oxc) {
    // One side declined to compile a function the other compiled.
    category = "memoization-scope";
  } else if (memoizedSlots.babel !== memoizedSlots.oxc) {
    // The same functions were compiled, but into caches of different sizes.
    category = "memoization-slots";
  } else if (isReordered(babelOutput, oxcOutput)) {
    category = "statement-order";
  } else {
    category = "structural";
  }

  return { category, causes, memoizedFunctions, memoizedSlots };
}

/** The slot count of every `_c(n)` cache the output allocates. */
function memoCaches(code) {
  return [...code.matchAll(MEMO_CACHE_CALL)].map(([, slots]) => Number.parseInt(slots, 10));
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

/**
 * Returns the minimal set of normalization names explaining the difference, or
 * an empty array when none of them do.
 */
function explainDifference(babelOutput, oxcOutput) {
  if (!matchesUnder(NORMALIZATIONS, babelOutput, oxcOutput)) {
    return [];
  }

  // Every normalization together reconciles the two, so drop the ones that are
  // not pulling their weight and keep those whose removal breaks the match.
  let required = NORMALIZATIONS;
  for (const normalization of NORMALIZATIONS) {
    const without = required.filter((candidate) => candidate !== normalization);
    if (matchesUnder(without, babelOutput, oxcOutput)) {
      required = without;
    }
  }
  return required.map(([name]) => name);
}

/** Reports whether the two outputs hold the same lines in a different order. */
function isReordered(babelOutput, oxcOutput) {
  const key = (code) => dropBlankLines(stripComments(code)).split("\n").sort().join("\n");
  return key(babelOutput) === key(oxcOutput);
}

function matchesUnder(normalizations, babelOutput, oxcOutput) {
  const apply = (code) =>
    normalizations.reduce((current, [, normalize]) => normalize(current), code);
  return apply(babelOutput) === apply(oxcOutput);
}

function dropBlankLines(code) {
  return dropLines(
    code
      .split("\n")
      .map((line) => line.trim())
      .join("\n"),
    (line) => line === "",
  );
}

function dropLines(code, shouldDrop) {
  return code
    .split("\n")
    .filter((line) => !shouldDrop(line))
    .join("\n");
}

/**
 * Drops imported bindings that nothing else references, so a side that elides
 * an import React Compiler made redundant lines up with a side that keeps it.
 */
function elideUnusedImports(code) {
  const lines = code.split("\n");
  const body = lines.filter((line) => !IMPORT_STATEMENT.test(line)).join("\n");
  const isUsed = (localName) => new RegExp(`\\b${escapeRegExp(localName)}\\b`).test(body);

  return lines
    .map((line) => {
      const statement = line.match(IMPORT_STATEMENT);
      if (statement === null) {
        return line;
      }
      const [, clause, module] = statement;
      const kept = splitImportClause(clause).filter(({ localName }) => isUsed(localName));
      if (kept.length === 0) {
        return null;
      }
      const named = kept.filter(({ isNamed }) => isNamed).map(({ text }) => text);
      const bare = kept.filter(({ isNamed }) => !isNamed).map(({ text }) => text);
      const parts = named.length === 0 ? bare : [...bare, `{ ${named.join(", ")} }`];
      return `import ${parts.join(", ")} from ${module};`;
    })
    .filter((line) => line !== null)
    .join("\n");
}

/**
 * Rewrites every top-level import to one sorted statement per module at the top
 * of the file, so where an import was placed, how its modules were split, and
 * what order its specifiers came in stop counting as differences.
 */
function canonicalizeImports(code) {
  const others = [];
  const bindingsByModule = new Map();

  for (const line of code.split("\n")) {
    const statement = line.match(IMPORT_STATEMENT);
    if (statement === null) {
      others.push(line);
      continue;
    }
    const [, clause, module] = statement;
    const bindings = bindingsByModule.get(module) ?? [];
    // `default` keeps a default import from sorting in among the named ones.
    bindings.push(
      ...splitImportClause(clause).map(({ text, isNamed }) => (isNamed ? text : `default ${text}`)),
    );
    bindingsByModule.set(module, bindings);
  }

  const imports = [...bindingsByModule]
    .sort(([left], [right]) => (left < right ? -1 : 1))
    .map(
      ([module, bindings]) => `import ${[...new Set(bindings)].sort().join(", ")} from ${module};`,
    );
  return [...imports, ...others].join("\n");
}

/** Splits `React, { a, b as c }` into its individual bindings. */
function splitImportClause(clause) {
  const named = clause.match(/\{([^}]*)\}/);
  const bindings = [];

  for (const text of clause.replace(/\{[^}]*\}/, "").split(",")) {
    if (text.trim() !== "") {
      bindings.push({ text: text.trim(), localName: localNameOf(text), isNamed: false });
    }
  }
  for (const text of named?.[1].split(",") ?? []) {
    if (text.trim() !== "") {
      bindings.push({ text: text.trim(), localName: localNameOf(text), isNamed: true });
    }
  }

  return bindings;
}

function localNameOf(specifier) {
  return specifier
    .trim()
    .split(/\s+as\s+/)
    .at(-1)
    .trim();
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Removes comments from generated JavaScript. Scans literals rather than
 * pattern-matching them, so comment markers inside strings, templates, and
 * regular expressions are left alone.
 */
function stripComments(code) {
  let output = "";
  let index = 0;
  // The brace depth each open template literal sits at, so a `}` can be told
  // apart from the end of a `${…}` substitution.
  const templateDepths = [];
  let braceDepth = 0;
  const emit = (text) => {
    output += text;
  };

  while (index < code.length) {
    const character = code[index];

    if (templateDepths.length > 0 && braceDepth === templateDepths.at(-1)) {
      // Inside template text, where nothing but `\`, a backtick, and `${` is
      // special — comment markers there are literal characters.
      if (character === "\\") {
        emit(code.slice(index, index + 2));
        index += 2;
      } else if (character === "`") {
        templateDepths.pop();
        emit(character);
        index++;
      } else if (character === "$" && code[index + 1] === "{") {
        braceDepth++;
        emit("${");
        index += 2;
      } else {
        emit(character);
        index++;
      }
      continue;
    }

    if (character === "/" && code[index + 1] === "/") {
      index = skipComment(code, index, "\n", false);
    } else if (character === "/" && code[index + 1] === "*") {
      index = skipComment(code, index + 2, "*/", true);
    } else if (character === '"' || character === "'") {
      index = copyQuoted(code, index, emit);
    } else if (character === "/" && startsRegExp(output)) {
      index = copyRegExp(code, index, emit);
    } else {
      if (character === "`") {
        templateDepths.push(braceDepth);
      } else if (character === "{") {
        braceDepth++;
      } else if (character === "}") {
        braceDepth--;
      }
      emit(character);
      index++;
    }
  }

  return output;
}

/** Advances past a comment, keeping the newline that ended a line comment. */
function skipComment(code, index, terminator, consumeTerminator) {
  const end = code.indexOf(terminator, index);
  if (end === -1) {
    return code.length;
  }
  return consumeTerminator ? end + terminator.length : end;
}

function copyQuoted(code, index, emit) {
  const quote = code[index];
  let cursor = index + 1;
  while (cursor < code.length && code[cursor] !== quote) {
    cursor += code[cursor] === "\\" ? 2 : 1;
  }
  emit(code.slice(index, cursor + 1));
  return cursor + 1;
}

function copyRegExp(code, index, emit) {
  let cursor = index + 1;
  let inCharacterClass = false;

  while (cursor < code.length) {
    const character = code[cursor];
    if (character === "\\") {
      cursor += 2;
      continue;
    }
    if (character === "\n") {
      // Not a regular expression after all, so treat the slash as an operator.
      emit("/");
      return index + 1;
    }
    if (character === "[") {
      inCharacterClass = true;
    } else if (character === "]") {
      inCharacterClass = false;
    } else if (character === "/" && !inCharacterClass) {
      break;
    }
    cursor++;
  }

  emit(code.slice(index, cursor + 1));
  return cursor + 1;
}

/**
 * Decides whether a `/` opens a regular expression rather than a division, from
 * what was emitted before it.
 */
function startsRegExp(output) {
  const before = output.trimEnd();
  if (before === "") {
    return true;
  }
  return !/[\w$)\]]$/.test(before) || REGEXP_PRECEDING_KEYWORD.test(before);
}
