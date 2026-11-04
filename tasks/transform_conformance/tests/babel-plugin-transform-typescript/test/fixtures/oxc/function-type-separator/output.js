const a = true, b = 1, c = 2;
const f = a ? b : () => b ? c : (x = 1) => x;
console.log(typeof f);
