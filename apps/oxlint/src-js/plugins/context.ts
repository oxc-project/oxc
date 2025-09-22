// Diagnostic in form passed by user to `Context#report()`
interface Diagnostic {
  message: string;
  node: {
    start: number;
    end: number;
    [key: string]: unknown;
  };
}

// Diagnostic in form sent to Rust
interface DiagnosticReport {
  message: string;
  loc: { start: number; end: number };
  ruleIndex: number;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

/**
 * Update a `Context` with file-specific data.
 *
 * We have to define this function within class body, as it's not possible to access private property
 * `#internal` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 *
 * @param context - `Context` object
 * @param ruleIndex - Index of this rule within `ruleIds` passed from Rust
 * @param filePath - Absolute path of file being linted
 */
export let setupContextForFile: (
  context: Context,
  ruleIndex: number,
  filePath: string,
) => void;

// Internal data within `Context` that don't want to expose to plugins.
// Stored as `#internal` property of `Context`.
interface InternalContext {
  // Full rule name, including plugin name e.g. `my-plugin/my-rule`.
  id: string;
  // Index into `ruleIds` sent from Rust
  ruleIndex: number;
  // Absolute path of file being linted
  filePath: string;
  // Options
  options: unknown[];
}

/**
 * Context class.
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` function.
 */
export class Context {
  // Internal data.
  // Initialized in constructor, updated by `setupContextForFile` before running visitor on file.
  #internal: InternalContext;

  /**
   * @class
   * @param fullRuleName - Rule name, in form `<plugin>/<rule>`
   */
  constructor(fullRuleName: string) {
    this.#internal = {
      id: fullRuleName,
      filePath: '',
      ruleIndex: 0,
      options: [],
    };
  }

  // Getter for full rule name, in form `<plugin>/<rule>`
  get id() {
    return this.#internal.id;
  }

  // Getter for absolute path of file being linted.
  get filename() {
    return this.#internal.filePath;
  }

  // Getter for absolute path of file being linted.
  // TODO: Unclear how this differs from `filename`.
  get physicalFilename() {
    return this.#internal.filePath;
  }

  // Getter for options for file being linted.
  get options() {
    return this.#internal.options;
  }

  /**
   * Report error.
   * @param diagnostic - Diagnostic object
   */
  report(diagnostic: Diagnostic): void {
    diagnostics.push({
      message: diagnostic.message,
      loc: { start: diagnostic.node.start, end: diagnostic.node.end },
      ruleIndex: this.#internal.ruleIndex,
    });
  }

  static {
    setupContextForFile = (context, ruleIndex, filePath) => {
      // TODO: Support `options`
      const internal = context.#internal;
      internal.ruleIndex = ruleIndex;
      internal.filePath = filePath;
    };
  }
}
