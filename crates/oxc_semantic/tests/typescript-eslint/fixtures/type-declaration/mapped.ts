type T = Record<string, string>;
type A = { [k in string]: T[k] };
