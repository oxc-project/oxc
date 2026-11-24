const two = BigInt(2);

two ** BigInt(254); // bail
BigInt(254) ** two; // bail
x **= BigInt(2); // bail

Math.pow(a, Number(2)); // transform

function shadowed(BigInt) {
  return Math.pow(BigInt(2), 2); // transform
}
