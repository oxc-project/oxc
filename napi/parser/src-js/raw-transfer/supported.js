import { rawTransferSupported as rawTransferSupportedBinding } from "../bindings.js";

let rawTransferIsSupported = null;

/**
 * Returns `true` if `experimentalRawTransfer` option is supported.
 *
 * Raw transfer is only supported on 64-bit little-endian systems,
 * and NodeJS >= v22.0.0, Deno >= v2.0.0, or Bun >= v1.2.0 (not on Windows).
 *
 * Where raw transfer buffers cannot be allocated on Rust side (e.g. in Electron), they're allocated on JS side,
 * which requires creating an `ArrayBuffer` larger than 4 GiB (see `createBuffer` in `common.js`).
 * Versions of NodeJS prior to v22.0.0 do not support that, and nor does Bun.
 * Bun before v1.2.0 fails to read from buffers allocated on Rust side.
 * Bun on Windows has not been tested, so is treated as unsupported.
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
  } catch {
    return false;
  }

  const isBun = !!global.Bun || !!global.process?.versions?.bun;
  if (isBun) {
    if (global.process?.platform === "win32") return false;
    const match = global.process?.versions?.bun?.match(/^(\d+)\.(\d+)\./);
    return !!match && (match[1] * 1 > 1 || (match[1] * 1 === 1 && match[2] * 1 >= 2));
  }

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
