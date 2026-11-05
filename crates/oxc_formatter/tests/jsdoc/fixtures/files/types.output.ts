/** @type {((element: HTMLElement) => boolean)[]} */
var filters = [];

/**
 * Returns a slice of the first array that doesn't contain the leading and
 * trailing elements the both arrays have in common.
 *
 * Examples:
 *
 *     trimCommon([1,2,3,4], [1,3,2,4]) => [2,3]
 *     trimCommon([1,2,3,4], [1,2,3,4]) => []
 *     trimCommon([1,2,0,0,3,4], [1,2,3,4]) => [0,0]
 *     trimCommon([1,2,3,4], [1,2,0,0,3,4]) => []
 *
 *
 *
 *     trimCommon([1,2,3,4],[1,3,2,4])
 *     trimCommon([1,2,3,4   ], [1,2,3,4])
 *     trimCommon([1,2,0,0,  3,4], [1,2,3,4])
 *     trimCommon([1,2,3,4],[1,2,0,0,3,4])
 *
 * @template T
 * @param {readonly T[]} a1
 * @param {readonly T[]} a2
 * @returns {T[]}
 */

/**
 * Returns a slice of the first array that doesn't contain the leading and
 * trailing elements the both arrays have in common.
 *
 * Examples:
 *
 *     trimCommon([1,2,3,4], [1,3,2,4]) => [2,3]
 *     trimCommon([1,2,3,4], [1,2,3,4]) => []
 *     trimCommon([1,2,0,0,3,4], [1,2,3,4]) => [0,0]
 *     trimCommon([1,2,3,4], [1,2,0,0,3,4]) => []
 *
 *     trimCommon([1,2,3,4],[1,3,2,4])
 *     trimCommon([1,2,3,4   ], [1,2,3,4])
 *     trimCommon([1,2,0,0,  3,4], [1,2,3,4])
 *     trimCommon([1,2,3,4],[1,2,0,0,3,4])
 *
 * @template T
 * @param {readonly T[]} a1
 * @param {readonly T[]} a2
 * @returns {T[]}
 */
