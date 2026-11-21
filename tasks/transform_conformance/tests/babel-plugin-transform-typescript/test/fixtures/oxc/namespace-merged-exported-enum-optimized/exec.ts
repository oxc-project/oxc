namespace N { export enum E { A } export const enum C { A } }
const original = N.E;
const originalConst = N.C;
namespace N { export enum E { B = 1 } export const enum C { B = 1 } }
expect(N.E).toBe(original);
expect(N.C).toBe(originalConst);
expect(N.E).toEqual({ 0: "A", 1: "B", A: 0, B: 1 });
expect(N.C).toEqual({ 0: "A", 1: "B", A: 0, B: 1 });
