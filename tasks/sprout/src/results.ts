/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import type { EvaluatorResult } from "./evaluator.ts";

export type SproutResult = { kind: "success"; value: string } | { kind: "invalid"; value: string };

function stringify(result: EvaluatorResult): string {
  return `(kind: ${result.kind}) ${result.value}${result.logs.length > 0 ? `\nlogs: [${result.logs.toString()}]` : ""}`;
}

export function compareResults(
  original: EvaluatorResult | null,
  compiled: EvaluatorResult,
): SproutResult {
  if (compiled.kind === "UnexpectedError") {
    return { kind: "invalid", value: "Unexpected error in compiled runner\n" + compiled.value };
  }
  if (original?.kind === "UnexpectedError") {
    return { kind: "invalid", value: "Unexpected error in original runner\n" + original.value };
  }
  if (
    original !== null
    && (original.kind !== compiled.kind
      || original.value !== compiled.value
      || original.logs.length !== compiled.logs.length
      || original.logs.some((value, index) => value !== compiled.logs[index]))
  ) {
    return {
      kind: "invalid",
      value: `Found differences in evaluator results\nOriginal (expected):\n${stringify(original)}\nCompiled:\n${stringify(compiled)}`,
    };
  }
  return { kind: "success", value: stringify(compiled) };
}
