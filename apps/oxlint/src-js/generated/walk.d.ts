// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

import type { Node, Program } from "./types.d.ts";
import type { VisitFn, EnterExit } from "../plugins/visitor.ts";

type CompiledVisitors = (VisitFn | EnterExit | null)[];

export declare function walkProgram(program: Program, visitors: CompiledVisitors): void;
export declare const ancestors: Node[];
