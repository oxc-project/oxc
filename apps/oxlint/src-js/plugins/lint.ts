import {
  DATA_POINTER_POS_32,
  SOURCE_LEN_OFFSET,
  // TODO(camc314): we need to generate `.d.ts` file for this module.
  // @ts-expect-error
} from '../generated/constants.mjs';
import { diagnostics, setupContextForFile } from './context.js';
import { registeredRules } from './load.js';
import { assertIs } from './utils.js';
import { addVisitorToCompiled, compiledVisitor, finalizeCompiledVisitor, initCompiledVisitor } from './visitor.js';

// @ts-expect-error we need to generate `.d.ts` file for this module.
import { TOKEN } from '../../dist/raw-transfer/lazy-common.mjs';
// @ts-expect-error we need to generate `.d.ts` file for this module.
import { walkProgram } from '../../dist/generated/lazy/walk.mjs';

// Buffer with typed array views of itself stored as properties
interface BufferWithArrays extends Uint8Array {
  uint32: Uint32Array;
  float64: Float64Array;
}

// Buffers cache.
//
// All buffers sent from Rust are stored in this array, indexed by `bufferId` (also sent from Rust).
// Buffers are only added to this array, never removed, so no buffers will be garbage collected
// until the process exits.
const buffers: (BufferWithArrays | null)[] = [];

// Text decoder, for decoding source text from buffer
const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

// Run rules on a file.
export function lintFile(filePath: string, bufferId: number, buffer: Uint8Array | null, ruleIds: number[]): string {
  // If new buffer, add it to `buffers` array. Otherwise, get existing buffer from array.
  // Do this before checks below, to make sure buffer doesn't get garbage collected when not expected
  // if there's an error.
  // TODO: Is this enough to guarantee soundness?
  if (buffer === null) {
    // Rust will only send a `bufferId` alone, if it previously sent a buffer with this same ID
    buffer = buffers[bufferId]!;
  } else {
    assertIs<BufferWithArrays>(buffer);
    const { buffer: arrayBuffer, byteOffset } = buffer;
    buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
    buffer.float64 = new Float64Array(arrayBuffer, byteOffset);

    for (let i = bufferId - buffers.length; i >= 0; i--) {
      buffers.push(null);
    }
    buffers[bufferId] = buffer;
  }
  assertIs<BufferWithArrays>(buffer);

  if (typeof filePath !== 'string' || filePath.length === 0) {
    throw new Error('expected filePath to be a non-zero length string');
  }
  if (!Array.isArray(ruleIds) || ruleIds.length === 0) {
    throw new Error('Expected `ruleIds` to be a non-zero len array');
  }

  // Get visitors for this file from all rules
  initCompiledVisitor();
  for (let i = 0; i < ruleIds.length; i++) {
    const ruleId = ruleIds[i];
    const { rule, context } = registeredRules[ruleId];
    setupContextForFile(context, i, filePath);
    const visitor = rule.create(context);
    addVisitorToCompiled(visitor);
  }
  const needsVisit = finalizeCompiledVisitor();

  // Visit AST.
  // Skip this if no visitors visit any nodes.
  // Some rules seen in the wild return an empty visitor object from `create` if some initial check fails
  // e.g. file extension is not one the rule acts on.
  if (needsVisit) {
    const { uint32 } = buffer,
      programPos = uint32[DATA_POINTER_POS_32],
      sourceByteLen = uint32[(programPos + SOURCE_LEN_OFFSET) >> 2];

    const sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));
    const sourceIsAscii = sourceText.length === sourceByteLen;
    const ast = {
      buffer,
      sourceText,
      sourceByteLen,
      sourceIsAscii,
      nodes: new Map(),
      token: TOKEN,
    };

    walkProgram(programPos, ast, compiledVisitor);
  }

  // Send diagnostics back to Rust
  const ret = JSON.stringify(diagnostics);
  diagnostics.length = 0;
  return ret;
}
