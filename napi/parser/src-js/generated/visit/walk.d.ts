// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/estree_visit.rs`.

import type * as ESTree from "@oxc-project/types";

type VisitFn = (node: ESTree.Node) => void;
type EnterExit = { enter: VisitFn; exit: VisitFn };
type CompiledVisitors = (VisitFn | EnterExit | null)[];

export declare function walkProgram(program: ESTree.Program, visitors: CompiledVisitors): void;
