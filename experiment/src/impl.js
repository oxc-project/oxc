import { walk } from 'oxc-walker';
import { parseSyncRawFromBuffer, parseSyncRawFromBufferNoDeser } from '../../napi/parser/index.js';
import { NO_DESER, NO_WALK, WALK } from './variants.js';

export default function run(sourceText, sourceBuffer, iterations, variant) {
  switch (variant) {
    case NO_WALK:
      runWithoutWalk(sourceText, sourceBuffer, iterations);
      break;
    case WALK:
      runWithWalk(sourceText, sourceBuffer, iterations);
      break;
    case NO_DESER:
      runWithoutDeser(sourceText, sourceBuffer, iterations);
      break;
    default:
      throw new Error('Invalid variant');
  }
}

function runWithoutWalk(sourceText, sourceBuffer, iterations) {
  for (let i = 0; i < iterations; i++) {
    const { program } = parseSyncRawFromBuffer('foo.js', sourceText, sourceBuffer);
  }
}

function runWithWalk(sourceText, sourceBuffer, iterations) {
  for (let i = 0; i < iterations; i++) {
    const { program } = parseSyncRawFromBuffer('foo.js', sourceText, sourceBuffer);
    walkAst(program);
  }
}

function runWithoutDeser(sourceText, sourceBuffer, iterations) {
  for (let i = 0; i < iterations; i++) {
    parseSyncRawFromBufferNoDeser('foo.js', sourceBuffer);
  }
}

function walkAst(program) {
  let identCount = 0;
  walk(program, {
    enter(node) {
      if (node.type === 'Identifier') {
        identCount++;
      }
    },
  });
}
