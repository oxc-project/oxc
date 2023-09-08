export const baz = "baz? really?"

export { default as bar } from './default-export'
export { default as foo } from './named-default-export'

// should allow conversion from CJS to ES6 as follows:
export { default as common } from './common'
