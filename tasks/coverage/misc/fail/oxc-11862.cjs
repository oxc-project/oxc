// Note that using is allowed in CJS files https://github.com/nodejs/node/issues/58663.
// But we treat cjs as `script`.
using a = { [Symbol.dispose]() {} }
