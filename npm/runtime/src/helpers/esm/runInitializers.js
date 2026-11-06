// Adapted from esbuild's runtime (https://github.com/evanw/esbuild/blob/main/internal/runtime/runtime.go).
// Copyright (c) 2020 Evan Wallace. MIT License.
/**
 * Run initializers stored in slot `flags >> 1` of the decorator context array.
 * If `flags & 1`, these are extra initializers which are called for their side effects.
 * Otherwise each initializer transforms `value`, which is returned.
 */
function _runInitializers(array, flags, self, value) {
  for (var i = 0, fns = array[flags >> 1], n = fns && fns.length; i < n; i++) {
    flags & 1 ? fns[i].call(self) : value = fns[i].call(self, value);
  }
  return value;
}
export { _runInitializers as default };