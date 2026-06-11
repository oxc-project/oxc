// Literal unions as assignment targets.
type Direction = "north" | "south" | "east" | "west";
type Bit = 0 | 1;
type Flag = true | "auto";

const ok_dir: Direction = "north";
const bad_dir: Direction = "up";

const ok_bit: Bit = 1;
const bad_bit: Bit = 2;

const ok_flag_literal: Flag = "auto";
const ok_flag_bool: Flag = true;
const bad_flag: Flag = false;

// A `let` initialized from a literal widens to string, so it no longer
// satisfies the literal union target.
let widened_dir = "north";
const bad_widened: Direction = widened_dir;

// A `const` keeps its literal type and stays assignable.
const fresh_dir = "south";
const ok_fresh: Direction = fresh_dir;

export {};
