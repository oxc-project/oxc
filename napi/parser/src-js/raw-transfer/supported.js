import { rawTransferSupported as rawTransferSupportedBinding } from '../bindings.js';

let rawTransferIsSupported = null;

/**
 * Returns `true` if `experimentalRawTransfer` is option is supported.
 *
 * Raw transfer is only supported on 64-bit little-endian systems,
 * and NodeJS >= v22.0.0 or Deno >= v2.0.0.
 *
 * Versions of NodeJS prior to v22.0.0 do not support creating an `ArrayBuffer` larger than 4 GiB.
 * Bun (as at v1.2.4) also does not support creating an `ArrayBuffer` larger than 4 GiB.
 * Support on Deno v1 is unknown and it's EOL, so treating Deno before v2.0.0 as unsupported.
 *
 * No easy way to determining pointer width (64 bit or 32 bit) in JS,
 * so call a function on Rust side to find out.
 *
 * @returns {boolean} - `true` if raw transfer is supported on this platform
 */
export function rawTransferSupported() {
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
  } catch (_err) { // oxlint-disable-line no-unused-vars
    return false;
  }

  const isBun = !!global.Bun || !!global.process?.versions?.bun;
  if (isBun) return false;

  const isDeno = !!global.Deno;
  if (isDeno) {
    const match = Deno.version?.deno?.match(/^(\d+)\./);
    return !!match && match[1] * 1 >= 2;
  }

  const isNode = global.process?.release?.name === 'node';
  if (!isNode) return false;

  const match = process.version?.match(/^v(\d+)\./);
  return !!match && match[1] * 1 >= 22;
}
