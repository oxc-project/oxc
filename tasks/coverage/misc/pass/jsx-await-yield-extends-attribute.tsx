async function f() {
  return <await extends disabled />;
}
function* g() {
  return <yield extends disabled />;
}
function* h() {
  return <await extends unknown>() => 0;
}
