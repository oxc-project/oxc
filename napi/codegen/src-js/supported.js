import { rawTransferSupported as rawTransferSupportedBinding } from "./bindings.js";

let rawTransferIsSupported = null;

/**
 * Returns `true` if raw transfer back (and therefore `print`) is supported.
 *
 * Raw transfer back is only supported on 64-bit little-endian systems,
 * and NodeJS >= v22.0.0 or Deno >= v2.0.0 - same requirements as `oxc-parser`'s
 * `experimentalRawTransfer` (see `napi/parser/src-js/raw-transfer/supported.js`).
 *
 * @returns {boolean} - `true` if raw transfer back is supported on this platform
 */
export function rawTransferBackSupported() {
  if (rawTransferIsSupported === null) {
    rawTransferIsSupported = rawTransferRuntimeSupported() && rawTransferSupportedBinding();
  }
  return rawTransferIsSupported;
}

// Checks copied from:
// https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/src/runtimes.ts
// MIT license: https://github.com/unjs/std-env/blob/ab15595debec9e9115a9c1d31bc7597a8e71dbfd/LICENCE
function rawTransferRuntimeSupported() {
  let global;
  try {
    global = globalThis;
  } catch {
    return false;
  }

  const isBun = !!global.Bun || !!global.process?.versions?.bun;
  if (isBun) return false;

  const isDeno = !!global.Deno;
  if (isDeno) {
    const match = Deno.version?.deno?.match(/^(\d+)\./);
    return !!match && match[1] * 1 >= 2;
  }

  const isNode = global.process?.release?.name === "node";
  if (!isNode) return false;

  const match = process.version?.match(/^v(\d+)\./);
  return !!match && match[1] * 1 >= 22;
}
