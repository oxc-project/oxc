type T = Record<string, string>;
type A = { [k in string as k]: T[k] };
