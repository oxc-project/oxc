function Component({ stop }) {
  let count = 0;
  const cb = () => count++;
  const start = () => { if (stop) start(); }; start();
  return null;
}
