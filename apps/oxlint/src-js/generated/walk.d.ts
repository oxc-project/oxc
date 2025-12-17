// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

import type { Node, Program } from "./types.d.ts";

type VisitFn = ((node: Node) => void) | null;
type EnterExitVisitor = { enter: VisitFn; exit: VisitFn } | null;
type CompiledVisitors = (VisitFn | EnterExitVisitor)[];

export declare function walkProgram(program: Program, visitors: CompiledVisitors): void;
export declare const ancestors: Node[];
