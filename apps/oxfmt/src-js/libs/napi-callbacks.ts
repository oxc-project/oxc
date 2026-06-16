/**
 * Adapters that turn the throwing `apis.ts` functions into never-rejecting NAPI callbacks.
 *
 * Rust awaits these callbacks as Promises.
 * A rejected Promise becomes a `napi::Error` whose drop can reach `napi_reference_unref`
 * during ThreadsafeFunction teardown and trigger a V8 fatal crash sometimes.
 *
 * To avoid that, every callback handed to Rust must always resolve,
 * carrying recoverable failures as data instead of rejecting.
 */

export type FormatFileResult = { ok: true; code: string } | { ok: false; error: string };

/**
 * Wrap a `formatFile`-shaped `Promise<string>` so it never rejects, preserving the error message.
 */
export async function toFormatFileResult(run: Promise<string>): Promise<FormatFileResult> {
  return run
    .then((code) => ({ ok: true as const, code }))
    .catch((err) => ({ ok: false as const, error: errorToMessage(err) }));
}

/**
 * Wrap a best-effort formatter `Promise<T>` so it never rejects, discarding the error as `null`.
 * Used for embedded code / tailwind sorting, where Rust falls back to the original code on failure.
 */
export async function toNullable<T>(run: Promise<T>): Promise<T | null> {
  return run.catch(() => null);
}

// Reproduces the `name: message` form (e.g. `SyntaxError: ...`) that Rust previously
// received when these errors crossed the NAPI boundary as rejected Promises.
function errorToMessage(err: unknown): string {
  if (err instanceof Error) return String(err);
  // `tinypool` with `runtime: "child_process"` serializes Error as a plain object
  // via IPC (e.g. `{ name, message, stack, ... }`), so rebuild the form from it.
  if (err !== null && typeof err === "object") {
    const { name, message } = err as { name?: unknown; message?: unknown };
    if (typeof message === "string") {
      return typeof name === "string" && name.length > 0 ? `${name}: ${message}` : message;
    }
  }
  return String(err);
}
