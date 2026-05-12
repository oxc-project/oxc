enum c_num { x = 123 }
enum d_num { x = 123 }

console.log([
  c_num?.x,
  d_num?.['x'],
]);
