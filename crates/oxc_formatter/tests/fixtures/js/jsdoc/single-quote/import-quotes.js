// singleQuote: true — single-quoted source is preserved
/** @import {Foo} from 'bar' */
const a = 1;

// singleQuote: true — double-quoted source is converted to single
/** @import {Foo} from "bar" */
const b = 2;

// Default import and merged imports also use single quotes
/**
 * @import Default from "mod"
 * @import {Z, A} from './types.js'
 * @import {M} from "./types.js"
 */
const c = 3;
