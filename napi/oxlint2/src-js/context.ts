import type { Node, Visitor } from './types.ts';

// Linter plugin
export interface Plugin {
  meta: {
    name: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

// Linter rule
export interface Rule {
  create: (context: Context) => Visitor;
}

// Diagnostic passed to `Context#report`
export interface Diagnostic {
  message: string;
  node: Node;
}

// Diagnostic as stored internally in `diagnostics`
export interface DiagnosticReport {
  message: string;
  loc: { start: number; end: number };
  ruleIndex: number;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

/**
 * Update a `Context` with file-specific data.
 *
 * We have to define this function within class body, as it's not possible to set private property
 * `#ruleIndex` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 *
 * @param context - `Context` object
 * @param ruleIndex - Index of this rule within `ruleIds` passed from Rust
 * @param filePath - Absolute path of file being linted
 */
export let setupContextForFile: (context: Context, ruleIndex: number, filePath: string) => void;

/**
 * Context class.
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` function.
 */
export class Context {
  // Full rule name, including plugin name e.g. `my-plugin/my-rule`.
  id: string;
  // Index into `ruleIds` sent from Rust. Set before calling `rule`'s `create` method.
  #ruleIndex: number;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  filename: string;
  // Absolute path of file being linted. Set before calling `rule`'s `create` method.
  physicalFilename: string;

  /**
   * @class
   * @param fullRuleName - Rule name, in form `<plugin>/<rule>`
   */
  constructor(fullRuleName: string) {
    this.id = fullRuleName;
  }

  /**
   * Report error.
   * @param diagnostic - Diagnostic object
   */
  report(diagnostic: Diagnostic): void {
    const { node } = diagnostic;
    diagnostics.push({
      message: diagnostic.message,
      loc: { start: node.start, end: node.end },
      ruleIndex: this.#ruleIndex,
    });
  }

  static {
    setupContextForFile = (context, ruleIndex, filePath) => {
      context.#ruleIndex = ruleIndex;
      context.filename = filePath;
      context.physicalFilename = filePath;
    };
  }
}
