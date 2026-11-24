const two = BigInt(2);

two ** BigInt(254); // bail
BigInt(254) ** two; // bail
x **= BigInt(2); // bail

a ** Number(2); // transform

function shadowed(BigInt) {
  return BigInt(2) ** 2; // transform
}
