// Adapted from esbuild's runtime (https://github.com/evanw/esbuild/blob/main/internal/runtime/runtime.go).
// Copyright (c) 2020 Evan Wallace. MIT License.
/**
 * Create the decorator context array for a class.
 *
 * Slots: 0 class extra initializers, 1 static method extra initializers,
 * 2 instance method extra initializers, 3 metadata object, then
 * `[initializers, extraInitializers]` pairs for each decorated field and accessor.
 */
function _decoratorStart(base) {
  var symbolMetadata = Symbol.metadata || Symbol["for"]("Symbol.metadata");
  var parentMetadata = base == null ? null : base[symbolMetadata];
  return [, , , Object.create(parentMetadata == null ? null : parentMetadata)];
}
export { _decoratorStart as default };