// Upstream bug 13 (`upstream-jsdoc-bugs.md`): the written text parses back to the same markdown,
// and never closes the comment.

/** See __a__/b for details. */
export function a() {}

/** Glob: src/**\/*.js */
export function b() {}

/** Escaped \*not emphasis\* here, `a*b*c` stays a*b*c, and &amp; stays. */
export function c() {}

/**
 * @param {string} x __init__/x
 * @param {string} y _italic_ and **bold**
 */
export function d(x, y) {}
