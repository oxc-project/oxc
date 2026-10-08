const a = true, b = 1, c = 2;

const original: number | ((x: number) => any) = a ? (b) : x => x ? c : (): any => b;
const unparenthesized = a ? b : x => x ? c : (): any => b;
const grouped = a ? (b) : (x => x ? c : (): any => b);
const unannotated = a ? (b) : x => x ? c : () => b;
const parameter = a ? (b) : x => x ? c : (y: number): any => b;

function async(): number { return b; }
const asyncCall = a ? async() : 0;
const asyncCallWithArrowAlternate = a ? async() : x => x;
const asyncArrow = a ? async (): Promise<number> => b : c;
