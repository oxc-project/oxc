// https://github.com/microsoft/TypeScript/issues/63527
// Expressions of the form `a ## b as T $$ c` are only valid when `as`/`satisfies`
// can be erased without changing the meaning, i.e. when `$$` does not bind tighter
// than `##`. The cases below are all erasable and must keep parsing.

const x01 = 1 as number * 2;
const x02 = 1 as any as number * 2;
const x05 = 1 as number + 1 * 2;
const x06 = 1 as any as number + 1 * 2;

const x07 = 1 * 1 as number + 2;
const x08 = 1 * 1 as any as number + 2;
const x09 = 1 as number * 1 + 2;
const x10 = 1 as any as number * 1 + 2;

const x11 = (1 + 1 as number) * 2;
const x12 = (1 + 1 as any as number) * 2;
const x13 = (1 as number + 1) * 2;
const x14 = (1 as any as number + 1) * 2;

const x15 = 1 + 1 as number === 2;
const x16 = 1 + 1 as any as number === 2;
const x17 = 1 + 1 as number > 2;
const x18 = 1 + 1 as any as number > 2;
const x19 = 1 + 1 as number >= 2;
const x20 = 1 + 1 as any as number >= 2;

const x21 = 1 + 1 as number >> 2;
const x22 = 1 + 1 as any as number >> 2;

const y01 = 1 satisfies number * 2;
const y02 = 1 satisfies any satisfies number * 2;
const y05 = 1 satisfies number + 1 * 2;
const y06 = 1 satisfies any satisfies number + 1 * 2;

const y07 = 1 * 1 satisfies number + 2;
const y08 = 1 * 1 satisfies any satisfies number + 2;
const y09 = 1 satisfies number * 1 + 2;
const y10 = 1 satisfies any satisfies number * 1 + 2;

const y11 = (1 + 1 satisfies number) * 2;
const y12 = (1 + 1 satisfies any satisfies number) * 2;
const y13 = (1 satisfies number + 1) * 2;
const y14 = (1 satisfies any satisfies number + 1) * 2;

const y15 = 1 + 1 satisfies number === 2;
const y16 = 1 + 1 satisfies any satisfies number === 2;
const y17 = 1 + 1 satisfies number > 2;
const y18 = 1 + 1 satisfies any satisfies number > 2;
const y19 = 1 + 1 satisfies number >= 2;
const y20 = 1 + 1 satisfies any satisfies number >= 2;

const y21 = 1 + 1 satisfies number >> 2;
const y22 = 1 + 1 satisfies any satisfies number >> 2;
