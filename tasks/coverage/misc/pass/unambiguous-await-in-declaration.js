// In unambiguous mode with ESM syntax, await in declarations should parse as await expression
const x = await (async function () { })(); export {}
