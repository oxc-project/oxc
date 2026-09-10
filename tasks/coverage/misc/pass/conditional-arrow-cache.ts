const a = true, b = 1, c = 2;
const f: number | ((x: number) => any) = a ? (b) : x => x ? c : (): any => b;

// The first colon ends the consequent; the alternate contains a typed arrow.
a ? (b) : x => x ? c : (): number => b;
a ? (b) : x => x ? c : y => y ? c : (): any => b;

// Keep colons that terminate conditional consequents distinct from return types.
a ? (b) : x => x;
a ? x => ({ b }) : y => ({ c });
a ? (x): any => x : b;
