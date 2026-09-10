const a = true, b = 1, c = 2;
const f = a ? (b) : (): any => b ? c : (x = 1): any => x;
console.log(typeof f);
