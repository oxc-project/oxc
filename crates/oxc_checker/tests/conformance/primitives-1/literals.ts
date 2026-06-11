export {};

// String literal unions
type Direction = "north" | "south";
const ok_dir: Direction = "north";
const bad_dir: Direction = "east";

// Number literal unions
type Bit = 0 | 1;
const ok_bit: Bit = 1;
const bad_bit: Bit = 2;

// Boolean literal types
type Affirm = true;
const ok_affirm: Affirm = true;
const bad_affirm: Affirm = false;

// Bigint literal types
type BigOne = 1n;
const ok_bigone: BigOne = 1n;
const bad_bigone: BigOne = 2n;

// number and bigint are distinct primitives
const bad_num_from_bigint: number = 1n;
const bad_bigint_from_num: bigint = 1;
const ok_bigint: bigint = 9007199254740993n;
