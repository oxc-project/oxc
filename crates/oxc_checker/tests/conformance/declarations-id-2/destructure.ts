const point = { x: 1, y: 2 };
const pair: [string, number] = ["a", 1];

// Errors: binding elements cannot be exported directly.
export const { x: bad_x, y: bad_y } = point;
export const [bad_name, bad_count] = pair;

// Silent: destructuring is fine when not exported.
const { x: ok_local_x } = point;

// Clean: exported with an explicit annotation instead.
export const ok_x: number = ok_local_x;
