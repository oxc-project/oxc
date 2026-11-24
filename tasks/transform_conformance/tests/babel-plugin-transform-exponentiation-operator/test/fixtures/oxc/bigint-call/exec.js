const two = BigInt(2);
expect(two ** BigInt(254)).toBe(BigInt(1) << BigInt(254));
expect(BigInt(3) ** two).toBe(BigInt(9));

let x = BigInt(3);
x **= BigInt(2);
expect(x).toBe(BigInt(9));

expect(Number(2) ** 10).toBe(1024);

function shadowed(BigInt) {
  return BigInt(2) ** 10;
}
expect(shadowed(Number)).toBe(1024);
