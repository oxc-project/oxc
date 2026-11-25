// First parameters take the function-type lookahead; later and generic parameters do not.
let firstAwait: (await: number) => void;
let laterAwait: (value: number, await: number) => void;
let genericAwait: <T>(await: T) => void;
let firstYield: (yield: number) => void;
let laterYield: (value: number, yield: number) => void;
let genericYield: <T>(yield: T) => void;
let optionalAwait: (await?: number) => void;
let optionalYield: (yield?: number) => void;
let untypedAwait: (await) => void;
let untypedYield: (yield) => void;

// Parenthesized types and computed properties must retain their interpretation.
type await = number;
type yield = number;
type AwaitType = (await);
type YieldType = (yield);
const key = "key";
interface Computed { [key]: unknown }

interface AwaitIndex { [await: string]: unknown }
interface YieldIndex { [yield: string]: unknown }
type AwaitIndexType = { [await: string]: unknown };
type YieldIndexType = { [yield: string]: unknown };
