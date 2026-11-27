// The `for await` lowering moves the iterable into the init of the new `for` statement.
// Scopes of functions, methods and getters in the iterable must move with it.
// The left side moves into the new loop body, which reuses the for-of scope.
async function f() {
  for await (const x of [function () {}, () => {}]) {}
  for await (x of [{ m() {}, get v() { return 1; } }]) {}
  for await ({ ...x } of [{ get v() { return 1; } }]) {}
  label: for await (const x of [class { m() {} }]) {}
  if (x) for await (const x of [function () {}]) {}
  for await (const { a = () => {} } of []) {}
  for await (const [b = function () {}] of []) {}
  for await ({ a = () => {} } of []) {}
  for await ({ a = function () {}, ...rest } of []) {}
}

async function* g() {
  for await (const x of [function () {}, () => {}]) {}
  for await (x of [{ m() {}, get v() { return 1; } }]) {}
  for await (const { a = () => {} } of []) {}
  for await (const [b = function () {}] of []) {}
  for await ({ a = () => {} } of []) {}
  for await ({ a = function () {}, ...rest } of []) {}
}

for await (const x of [function () {}, () => {}]) {}
for await (x of [{ m() {}, get v() { return 1; } }]) {}
for await ({ ...x } of [{ get v() { return 1; } }]) {}
for await (const { a = () => {} } of []) {}
for await (const [b = function () {}] of []) {}
for await ({ a = () => {} } of []) {}
for await ({ a = function () {}, ...rest } of []) {}
