import { transformSync, HelperMode } from './index.js';

const tests = [
  ['var Foo = @dec class {}; Foo', 'simple class expression'],
  ['var Foo = (x => x)(@dec("") class {}); Foo', 'iife wrapped'],
  ['@dec class Foo {}; Foo', 'class declaration'],
];

for (const [body, label] of tests) {
  try {
    const result = transformSync('test.ts', body, {
      sourceType: 'script',
      target: 'esnext',
      decorator: {},
      helpers: { mode: HelperMode.External },
    });
    console.log('--- ' + label + ' ---');
    console.log('Input:', JSON.stringify(body));
    console.log('Output:', result.code);
  } catch(e) {
    console.log('--- ' + label + ' --- ERROR:', e.message);
  }
}
