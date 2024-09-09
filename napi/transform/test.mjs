import assert from 'assert';
import oxc from './index.js';

console.log(`Testing on ${process.platform}-${process.arch}`);

function test(ret, expected) {
  console.log(ret.code);
  console.log(ret.map);
  for (const error of ret.errors) {
    console.log(error);
  }
  assert.equal(ret.code, expected.code);
  assert.deepEqual(ret.map, expected.map);
  assert(ret.errors.length == 0);
}

test(oxc.isolatedDeclaration('test.ts', 'class A {}', { sourcemap: true }), {
  code: 'declare class A {}\n',
  map: {
    mappings: 'AAAA,cAAM,EAAE,CAAE',
    names: [],
    sources: ['test.ts'],
    sourcesContent: ['class A {}'],
    version: 3,
  },
});

test(oxc.transform('test.ts', 'class A<T> {}', { sourcemap: true }), {
  code: 'class A {}\n',
  map: {
    mappings: 'AAAA,MAAM,EAAK,CAAE',
    names: [],
    sources: ['test.ts'],
    sourcesContent: ['class A<T> {}'],
    version: 3,
  },
});
