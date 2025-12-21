/*
 * CFG (control flow graph) aka Code Path Analysis (what ESLint calls it).
 *
 * Function to construct CFG, and walk AST and call visitors for CFG events.
 */

// @ts-expect-error - internal module of ESLint with no types
import CodePathAnalyzer from "../../node_modules/eslint/lib/linter/code-path-analysis/code-path-analyzer.js";
// @ts-expect-error - internal module of ESLint with no types
import Traverser from "../../node_modules/eslint/lib/shared/traverser.js";

import { VisitNodeStep, CallMethodStep } from "@eslint/plugin-kit";
import visitorKeys from "../generated/keys.ts";
import { LEAF_NODE_TYPES_COUNT, NODE_TYPE_IDS_MAP } from "../generated/type_ids.ts";
import { ancestors } from "../generated/walk.js";
import { debugAssert, typeAssertIs } from "../utils/asserts.ts";

import type { EnterExit, VisitFn } from "./visitor.ts";
import type { Node, Program } from "../generated/types.d.ts";
import type { CompiledVisitors } from "../generated/walk.js";

/**
 * Step to walk AST.
 */
type Step = VisitNodeStep | CallMethodStep;

const STEP_KIND_VISIT = 1;

const STEP_PHASE_ENTER = 1;
const STEP_PHASE_EXIT = 2;

// Array of steps to walk AST.
// Singleton array which is re-used for each walk, and emptied after each walk.
const steps: Step[] = [];

/**
 * Reset state for walking AST with CFG.
 *
 * If walking AST completes without error, `walkProgramWithCfg` will reset the state itself.
 * So it's only necessary to call this function if an error occurs during AST walking.
 */
export function resetCfgWalk(): void {
  steps.length = 0;
}

/**
 * Walk AST with CFG (control flow graph) events.
 *
 * Use this function to walk AST instead of `walkProgram`, when visitor listens for CFG events.
 *
 * It's much slower than `walkProgram`, so prefer `walkProgram` unless visitor includes handlers for CFG events.
 *
 * It walks the whole AST twice:
 *
 * 1. First time to build the CFG graph.
 *    In this first pass, it builds a list of steps to walk AST (including visiting nodes and CFG events).
 *    This list is stored in `steps` array.
 *
 * 2. Visit AST with provided visitor.
 *    Run through the steps, in order, calling visit functions for each step.
 *
 * TODO: This is copied from ESLint and is not very efficient. We could improve its performance in many ways.
 * See TODO comments in the code below for some ideas for optimization.
 *
 * @param ast - AST
 * @param visitors - Visitors array
 */
export function walkProgramWithCfg(ast: Program, visitors: CompiledVisitors): void {
  // Get the steps that need to be run to walk the AST
  prepareSteps(ast);

  // Walk the AST
  const stepsLen = steps.length;
  debugAssert(stepsLen > 0, "`steps` should not be empty");

  for (let i = 0; i < stepsLen; i++) {
    const step = steps[i];
    if (step.kind === STEP_KIND_VISIT) {
      const node = step.target as Node;

      if (step.phase === STEP_PHASE_ENTER) {
        // Enter node - can be leaf or non-leaf node
        const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;
        const visit = visitors[typeId];
        if (typeId < LEAF_NODE_TYPES_COUNT) {
          // Leaf node
          if (visit !== null) {
            typeAssertIs<VisitFn>(visit);
            visit(node);
          }
          // Don't add node to `ancestors`, because we don't visit them on exit
        } else {
          // Non-leaf node
          if (visit !== null) {
            typeAssertIs<EnterExit>(visit);
            const { enter } = visit;
            if (enter !== null) enter(node);
          }

          ancestors.unshift(node);
        }
      } else {
        // Exit non-leaf node
        ancestors.shift();

        const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;
        const enterExit = visitors[typeId];
        if (enterExit !== null) {
          typeAssertIs<EnterExit>(enterExit);
          const { exit } = enterExit;
          if (exit !== null) exit(node);
        }
      }
    } else {
      const eventId = NODE_TYPE_IDS_MAP.get(step.target)!;
      const visit = visitors[eventId];
      if (visit !== null) {
        (visit as any).apply(undefined, step.args);
      }
    }
  }

  // Reset `steps` array
  steps.length = 0;
}

/**
 * Walk AST and put a list of all steps to walk AST into `steps` array.
 * @param ast - AST
 */
function prepareSteps(ast: Program) {
  debugAssert(steps.length === 0, "`steps` should be empty at start of `prepareSteps`");

  // Length of `steps` array after entering each node.
  // Used in debug build to check that no leaf nodes emit CFG events (see below).
  // Minifier removes this var in release build.
  let stepsLenAfterEnter = 0;

  // Create `CodePathAnalyzer`.
  // It stores steps to walk AST.
  //
  // This is really inefficient code.
  // We could improve it in several ways (in ascending order of complexity):
  //
  // * Get rid of the bloated `VisitNodeStep` and `CallMethodStep` classes. Just use plain objects.
  // * Combine `step.kind` and `step.phase` into a single `step.type` property.
  // * Reduce object creation by storing steps as 2 arrays (struct of arrays pattern):
  //   * Array 1: Step type (number).
  //   * Array 2: Step data - AST node object for enter/exit node steps, args for CFG events.
  // * Alternatively, use a single array containing step objects as now, but recycle the objects
  //   (SoA option is probably better).
  // * Avoid repeated conversions from `type` (string) to `typeId` (number) when iterating through steps.
  //   * Generate separate `enterNode` / `exitNode` functions for each node type.
  //   * Set them on `analyzer.original` before calling `analyzer.enterNode` / `analyzer.exitNode`.
  //   * These functions would know the type ID of the node already, and then could store type ID in steps.
  //   * When iterating through steps, use that type ID instead of converting `node.type` to `typeId` every time.
  // * Copy `CodePathAnalyzer` code into this repo and rewrite it to work entirely with type IDs instead of strings.
  //
  // TODO: Apply these optimizations (or at least some of them).
  const analyzer = new CodePathAnalyzer({
    enterNode(node: Node) {
      steps.push(
        new VisitNodeStep({
          target: node,
          phase: STEP_PHASE_ENTER,
          args: [node],
        }),
      );

      if (DEBUG) stepsLenAfterEnter = steps.length;
    },

    leaveNode(node: Node) {
      const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;

      if (typeId >= LEAF_NODE_TYPES_COUNT) {
        // Non-leaf node
        steps.push(
          new VisitNodeStep({
            target: node,
            phase: STEP_PHASE_EXIT,
            args: [node],
          }),
        );
      } else {
        // Leaf node.
        // Don't add a step.

        // In debug build, check that no CFG events were emitted between entering and exiting this leaf node.
        // If they were, it would break our assumptions.
        // We combine enter and exit visit functions for leaf nodes into a single function which runs on entering node.
        // That's fine if there are no CFG events emitted between entering and exiting node.
        // But if CFG events were emitted between entering node and exiting node, then the order the rule's
        // visit functions are called in would be wrong.
        // `exit` visit fn would be called before the CFG event handlers, instead of after.
        if (DEBUG && steps.length !== stepsLenAfterEnter) {
          const eventNames = steps.slice(stepsLenAfterEnter).map((step) => step.target) as string[];
          throw new Error(
            `CFG events emitted during visiting leaf node \`${node.type}\`: ${eventNames.join(", ")}`,
          );
        }
      }
    },

    emit(eventName: string, args: any[]) {
      steps.push(
        new CallMethodStep({
          target: eventName,
          args,
        }),
      );
    },
  });

  // Walk AST passing enter and exit event to the `CodePathAnalyzer`
  //
  // TODO: Use a faster walker.
  // Could use our own `walkProgram`, though that builds `ancestors` array unnecessarily, which is probably slow.
  // Would be better to generate a separate walker for this purpose.
  Traverser.traverse(ast, {
    enter(node: Node) {
      analyzer.enterNode(node);
    },
    leave(node: Node) {
      analyzer.leaveNode(node);
    },
    visitorKeys,
  });
}
