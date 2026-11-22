namespace N { export enum E { A } }
const original = N.E;
namespace N { export enum E { B = 1 } }
expect(N.E).toBe(original);
expect(N.E).toEqual({ 0: "A", 1: "B", A: 0, B: 1 });
namespace N { export enum E { C = "c" } export enum E { D = "d" } }
expect(N.E).toBe(original);
expect(N.E).toEqual({ 0: "A", 1: "B", A: 0, B: 1, C: "c", D: "d" });

namespace Outer.Inner { export enum E { A } }
const nestedOriginal = Outer.Inner.E;
namespace Outer.Inner { export enum E { B = 1 } }
expect(Outer.Inner.E).toBe(nestedOriginal);
expect(Outer.Inner.E).toEqual({ 0: "A", 1: "B", A: 0, B: 1 });

namespace Private { enum E { A } export const first = E; }
namespace Private { enum E { B = 1 } export const second = E; }
expect(Private.first).not.toBe(Private.second);
expect(Private.first).toEqual({ 0: "A", A: 0 });
expect(Private.second).toEqual({ 1: "B", B: 1 });
