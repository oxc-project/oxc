import { walk } from 'oxc-walker';
import { parseSyncRawFromBuffer } from '../../napi/parser/index.js';

export default function run(sourceText, sourceBuffer, iterations, shouldWalk) {
  for (let i = 0; i < iterations; i++) {
    const { program } = parseSyncRawFromBuffer('foo.js', sourceText, sourceBuffer);
    if (shouldWalk) walkAst(program);
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
