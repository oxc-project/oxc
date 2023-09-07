export const c = 'foo'

export * from './named-exports'

// #328: this exports only 'foo', not the default.
export * from './bar'
