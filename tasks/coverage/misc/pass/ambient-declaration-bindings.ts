declare var mergedVariable: number;
var mergedVariable = 1;
var reverseVariable = 2;
declare var reverseVariable: number;
declare function mergedFunction(value: number): number;
function mergedFunction(value: number) { return value; }
function overloaded(value: number): number;
function overloaded(value: number) { return value; }
declare enum MergedEnum { A }
enum MergedEnum { B = 1 }
namespace RuntimeMerged { export const value = 1; }
declare class RuntimeMerged {}
declare class ReverseRuntimeMerged {}
namespace ReverseRuntimeMerged { export const value = 2; }
const outer = 3;
function nested() {
    declare const outer: number;
    return () => outer;
}
function unbound() {
    declare const missing: number;
    {
        declare const missing: number;
        console.log(missing);
    }
    return missing;
}
declare const { destructured, nested: [arrayElement] }: { destructured: number; nested: number[] };
declare class AmbientClass {}
declare enum AmbientEnum { A }
declare enum SelfReferencingAmbient { A = SelfReferencingAmbient.B, B = 1 }
export declare enum ExportedSelfReferencingAmbient { A = ExportedSelfReferencingAmbient.B, B = 1 }
export declare const exported: number;
console.log(mergedVariable, reverseVariable, mergedFunction(1), overloaded(1), MergedEnum.B,
    nested(), unbound(), destructured, arrayElement, AmbientClass, AmbientEnum, exported);
