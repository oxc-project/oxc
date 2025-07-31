import { createRequire } from 'node:module';

// Import methods and objects from `oxc-parser`.
// Use `require` not `import` as `oxc-parser` uses `require` internally,
// and need to make sure get same instance of modules as it uses internally,
// otherwise `TOKEN` here won't be same `TOKEN` as used within `oxc-parser`.
const require = createRequire(import.meta.url);
// const { TOKEN } = require('../dist/parser/raw-transfer/lazy-common.cjs'),
// walkProgram = require('../dist/parser/generated/lazy/walk.cjs');

const deserialize = require('../../../parser/generated/deserialize/js.js'),
  { TOKEN } = require('../../../parser/raw-transfer/lazy-common.js'),
  walkProgram = require('../../../parser/generated/lazy/walk.js'),
  { DATA_POINTER_POS_32, SOURCE_LEN_OFFSET } = require('../../../parser/generated/constants.js'),
  { NODE_TYPE_IDS_MAP, NODE_TYPES_COUNT } = require('../../../parser/generated/lazy/types.js');

// ID of this worker
let workerId;

// `true` if logging is enabled
let log = false;

// Buffer used to transfer ASTs
let buffer;

/**
 * Store flag for whether logging is enabled.
 * @param {number} id - Worker ID
 * @param {boolean} shouldLog - `true` if logging is enabled
 */
export function setWorkerIdAndLog(id, shouldLog) {
  workerId = id;
  log = shouldLog;
}

/**
 * Store buffer.
 * @param {Uint8Array} uint8Array - Buffer
 */
export function storeBuffer(uint8Array) {
  if (log) console.log('> Received buffer on JS worker', workerId);
  buffer = uint8Array;
  const { buffer: arrayBuffer, byteOffset } = buffer;
  buffer.uint32 = new Uint32Array(arrayBuffer, byteOffset);
  buffer.float64 = new Float64Array(arrayBuffer, byteOffset);
}

const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true });

/**
 * Run workload.
 * @param {number} visitorId - Visitor to use. If 0, use eager deserialization.
 */
export function workload(visitorId) {
  if (visitorId === 0) {
    workloadEager();
  } else {
    workloadLazy(visitorId);
  }

  if (log) console.log('> Finished job on JS worker', workerId);
}

function workloadEager() {
  if (log) console.log('> Start job (eager) on JS worker', workerId);

  const { uint32 } = buffer,
    programPos = uint32[DATA_POINTER_POS_32],
    sourceByteLen = uint32[(programPos + SOURCE_LEN_OFFSET) >> 2];

  const sourceText = textDecoder.decode(buffer.subarray(0, sourceByteLen));

  deserialize(buffer, sourceText, sourceByteLen);
}

const emptyVisitor = [];
for (let i = NODE_TYPES_COUNT; i !== 0; i--) {
  emptyVisitor.push(null);
}

const debuggerVisitor = [...emptyVisitor];
debuggerVisitor[NODE_TYPE_IDS_MAP.get('DebuggerStatement')] = _ident => {};

const identVisitor = [...emptyVisitor];
identVisitor[NODE_TYPE_IDS_MAP.get('IdentifierName')] = _ident => {};
identVisitor[NODE_TYPE_IDS_MAP.get('IdentifierReference')] = _ident => {};
identVisitor[NODE_TYPE_IDS_MAP.get('BindingIdentifier')] = _ident => {};
identVisitor[NODE_TYPE_IDS_MAP.get('LabelIdentifier')] = _ident => {};

const visitors = [
  null,
  emptyVisitor,
  debuggerVisitor,
  identVisitor,
];

const visitorNames = [
  null,
  'empty',
  'debugger',
  'ident',
];

function workloadLazy(visitorId) {
  if (log) {
    console.log(`> Start job (${visitorNames[visitorId]} visitor) on JS worker`, workerId);
  }

  // TODO
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

  const visitor = visitors[visitorId];

  walkProgram(programPos, ast, visitor);
}
