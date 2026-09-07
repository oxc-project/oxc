type FixedBeforeOptional = [...a: [string], b?: number];
type FixedBeforeRest = [...a: [string], ...b: number[]];
type RestBeforeFixed = [...a: string[], ...b: [number]];
type RestBeforeRequired = [...a: string[], b: number];
type VariadicBeforeOptional<T extends unknown[]> = [...a: T, b?: number];
type VariadicBeforeRest<T extends unknown[]> = [...a: T, ...b: string[]];
type RestBeforeVariadic<T extends unknown[]> = [...a: string[], ...b: T];
type MultipleVariadic<T extends unknown[], U extends unknown[]> = [...a: T, ...b: U];
