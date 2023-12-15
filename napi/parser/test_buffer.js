const oxc = require('./index');
const assert = require('assert');
const flexbuffers = require('flatbuffers/js/flexbuffers');
const file = require('fs').readFileSync(__dirname + '/index.js', 'utf8');

function testBuffer() {
  const buffer = oxc.parseSyncBuffer(file);
  const ref = flexbuffers.toReference(buffer.buffer);
  assert(ref.isMap());
  assert.equal(ref.get('type').stringValue(), 'Program');
  const body = ref.get('body');
  assert(body.isVector());
}

function testJSON() {
  const ret = oxc.parseSync(file);
  const program = JSON.parse(ret.program);
  assert(typeof program === 'object');
  assert.equal(program.type, 'Program');
  assert(Array.isArray(program.body));
}

function benchmark(func, time) {
  console.time(func.name);
  for (let i = 0; i < time; i++) {
    func();
  }
  console.timeEnd(func.name)
}

benchmark(testJSON, 10000);
benchmark(testBuffer, 10000);
