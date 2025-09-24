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
  start: number;
  end: number;
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

/**
 * Get internal data from `Context`.
 *
 * Throws an `Error` if `Context` has not been set up for a file (in body of `createOnce`).
 *
 * We have to define this function within class body, as it's not possible to access private property
 * `#internal` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 * We don't use a private class method, because private property/method accesses are somewhat expensive.
 *
 * @param context - `Context` object
 * @param actionDescription - Description of the action being attempted. Used in error message if context is not set up.
 * @returns `InternalContext` object
 * @throws {Error} If context has not been set up
 */
let getInternal: (context: Context, actionDescription: string) => InternalContext;

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
      ruleIndex: -1,
      options: [],
    };
  }

  // Getter for full rule name, in form `<plugin>/<rule>`
  get id() {
    return getInternal(this, 'access `context.id`').id;
  }

  // Getter for absolute path of file being linted.
  get filename() {
    return getInternal(this, 'access `context.filename`').filePath;
  }

  // Getter for absolute path of file being linted.
  // TODO: Unclear how this differs from `filename`.
  get physicalFilename() {
    return getInternal(this, 'access `context.physicalFilename`').filePath;
  }

  // Getter for options for file being linted.
  get options() {
    return getInternal(this, 'access `context.options`').options;
  }

  /**
   * Report error.
   * @param diagnostic - Diagnostic object
   */
  report(diagnostic: Diagnostic): void {
    const { ruleIndex } = getInternal(this, 'report errors');
    // TODO: Validate `diagnostic`
    const { node } = diagnostic;
    diagnostics.push({
      message: diagnostic.message,
      start: node.start,
      end: node.end,
      ruleIndex,
    });
  }

  static {
    setupContextForFile = (context, ruleIndex, filePath) => {
      // TODO: Support `options`
      const internal = context.#internal;
      internal.ruleIndex = ruleIndex;
      internal.filePath = filePath;
    };

    getInternal = (context, actionDescription) => {
      const internal = context.#internal;
      if (internal.ruleIndex === -1) throw new Error(`Cannot ${actionDescription} in \`createOnce\``);
      return internal;
    };
  }
}
