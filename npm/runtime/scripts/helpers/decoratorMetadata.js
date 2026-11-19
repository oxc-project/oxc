// Adapted from esbuild's runtime (https://github.com/evanw/esbuild/blob/main/internal/runtime/runtime.go).
// Copyright (c) 2020 Evan Wallace. MIT License.
function _decoratorMetadata(array, target) {
  var symbolMetadata = Symbol.metadata || Symbol["for"]("Symbol.metadata");
  var value = array[3];
  Object.defineProperty(target, symbolMetadata, {
    enumerable: true,
    configurable: true,
    writable: true,
    value: value,
  });
  return target;
}
