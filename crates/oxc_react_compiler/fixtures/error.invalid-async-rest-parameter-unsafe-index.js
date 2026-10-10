function Component() {
  let count = 0;
  async function run(...callbacks) { await 0; callbacks[1](); } run(() => null, () => count++); return null;
}
