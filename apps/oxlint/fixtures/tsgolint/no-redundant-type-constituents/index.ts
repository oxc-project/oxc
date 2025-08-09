// Examples of incorrect code for no-redundant-type-constituents rule

// unknown is redundant in unions
type T1 = string | unknown;

// any is redundant in unions
type T2 = string | any;

// never is redundant in unions
type T3 = string | never;