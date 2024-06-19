// Correct
export function fnDeclGood(p: T = [], rParam = ""): void { };
export function fnDeclGood2(p: T = [], rParam?: number): void { };

// Incorrect
export function fnDeclBad<T>(p: T = [], rParam: T = "", r2: T): void { }
export function fnDeclBad2<T>(p: T = [], r2: T): void { }
export function fnDeclBad3<T>(p: T = [], rParam?: T, r2: T): void { }
