const arg = 1;
// the typeof references the parameter, not the variable
type A = (arg: string) => typeof arg;
