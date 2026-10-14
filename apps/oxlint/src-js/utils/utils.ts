/**
 * Get error message from an error.
 *
 * `err` is expected to be an `Error` object, but can be anything.
 *
 * * If it's an `Error`, the error message and stack trace is returned.
 * * If it's another object with a string `message` property, `message` is returned.
 * * Otherwise, a generic "Unknown error" message is returned.
 *
 * This function will never throw, and always returns a non-empty string, even if:
 *
 * * `err` is `null` or `undefined`.
 * * `err` is an object with a getter for `message` property which throws.
 * * `err` has a getter for `stack` or `message` property which returns a different value each time it's accessed.
 *
 * @param err - Error
 * @returns Error message
 */
export function getErrorMessage(err: unknown): string {
  try {
    if (err instanceof Error) {
      // Note: `stack` includes the error message
      const { stack } = err;
      if (typeof stack === "string" && stack !== "") return stack;
    }

    const { message } = err as { message?: unknown };
    if (typeof message === "string" && message !== "") return message;
  } catch {}

  return "Unknown error";
}

/**
 * Replace escaped lone surrogates in a JSON string with U+FFFD.
 *
 * `JSON.stringify` writes a lone surrogate as a `\uXXXX` escape (well-formed `JSON.stringify`, ES2019).
 * The Rust side parses the JSON into `String`s, which are UTF-8 and cannot hold a lone surrogate,
 * so `serde_json` rejects the escape and the whole payload fails to parse.
 *
 * A well-formed surrogate pair is written literally, so any `\uD800`-`\uDFFF` escape in the output is lone.
 * An escaped backslash (`\\`) is consumed first, so a literal `\ud800` in the source text is left alone.
 *
 * @param json - Output of `JSON.stringify`
 * @returns `json` with every lone surrogate escape replaced by `\ufffd`
 */
export function replaceLoneSurrogateEscapes(json: string): string {
  return json.replace(/\\\\|\\u[dD][89a-fA-F][0-9a-fA-F]{2}/g, (match) =>
    match === "\\\\" ? match : "\\ufffd",
  );
}
