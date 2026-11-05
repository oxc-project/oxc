// A line-start `@name` starts a block tag on the next pass (`@scope/pkg` does not, it has a `/`),
// so wrapping must keep such a word off line starts (issue #28808).

export interface Mount {
  /** The cues the project resolved as it evaluated and first rendered (`cues` in @scope/pkg). */
  cues: string[];
}

/**
 * @param cues The cues the project resolved as it evaluated and rendered (`cues` in @scope/pkg).
 */
export function mount(cues: string[]) {}

/** Pass the value through the options bag as described in the docs for the big @foo plugin here. */
export const foo = 1;

// The tag name ends at `{`, so a `/` inside the type keeps the tag
/**
 * Foo bar.
 * @param{import('./a').B} x The value.
 */
export function bar(x) {}
