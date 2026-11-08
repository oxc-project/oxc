const o = "o";
const alias = o + "k";
enum E { A = o, B = 1, C = alias }
console.log(JSON.stringify(E));
