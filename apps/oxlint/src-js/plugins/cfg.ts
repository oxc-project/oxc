/*
 * CFG (control flow graph) aka Code Path Analysis (what ESLint calls it).
 *
 * Function to construct CFG, and walk AST and call visitors for CFG events.
 */

// @ts-expect-error - internal module of ESLint with no types
import CodePathAnalyzer from "../../node_modules/eslint/lib/linter/code-path-analysis/code-path-analyzer.js";
// @ts-expect-error - internal module of ESLint with no types
import Traverser from "../../node_modules/eslint/lib/shared/traverser.js";

import visitorKeys from "../generated/keys.ts";
import {
  LEAF_NODE_TYPES_COUNT,
  NODE_TYPE_IDS_MAP,
  NODE_TYPES_COUNT,
  TYPE_IDS_COUNT,
} from "../generated/type_ids.ts";
import { ancestors } from "../generated/walk.js";
import { debugAssert, typeAssertIs } from "../utils/asserts.ts";

import type { EnterExit, VisitFn } from "./visitor.ts";
import type { Node, Program } from "../generated/types.d.ts";
import type { CompiledVisitors } from "../generated/walk.js";

/**
 * Offset added to type IDs for exit visits to distinguish them from enter visits.
 * Using 256 as it's a power of 2 and larger than the maximum type ID (171).
 *
 * Type ID encoding:
 * - Enter visit (nodes): 0 to NODE_TYPES_COUNT-1 (0-164)
 * - Call method (CFG events): NODE_TYPES_COUNT to TYPE_IDS_COUNT-1 (165-171)
 * - Exit visit (non-leaf nodes): typeId + EXIT_TYPE_ID_OFFSET (256+)
 */
const EXIT_TYPE_ID_OFFSET = 256;

debugAssert(
  EXIT_TYPE_ID_OFFSET >= TYPE_IDS_COUNT,
  "`EXIT_TYPE_ID_OFFSET` must be >= `TYPE_IDS_COUNT`",
);

// Struct of Arrays (SoA) pattern for step storage.
// Using 2 arrays instead of an array of objects improves memory locality and V8 optimization.

/**
 * Encoded type IDs for each step.
 * - For enter visits: node type ID (0-164)
 * - For CFG events: event type ID (165-171)
 * - For exit visits: node type ID + EXIT_TYPE_ID_OFFSET (256+)
 */
const stepTypeIds: number[] = [];

/**
 * Step data for each step.
 * - For visit steps (enter/exit): the AST node
 * - For call steps (CFG events): the args array
 */
const stepData: (Node | unknown[])[] = [];

/**
 * Reset state for walking AST with CFG.
 *
 * If walking AST completes without error, `walkProgramWithCfg` will reset the state itself.
 * So it's only necessary to call this function if an error occurs during AST walking.
 */
export function resetCfgWalk(): void {
  stepTypeIds.length = 0;
  stepData.length = 0;
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
 *    This list is stored in the SoA arrays (stepTypeIds, stepData).
 *
 * 2. Visit AST with provided visitor.
 *    Run through the steps, in order, calling visit functions for each step.
 *
 * TODO: This is was originally copied from ESLint, and has been adapted for better performance.
 * But we could further improve its performance in many ways.
 * See TODO comments in the code below for some ideas for optimization.
 *
 * @param ast - AST
 * @param visitors - Visitors array
 */
export function walkProgramWithCfg(ast: Program, visitors: CompiledVisitors): void {
  // Get the steps that need to be run to walk the AST
  prepareSteps(ast);

  // Walk the AST
  const stepsLen = stepTypeIds.length;
  debugAssert(stepsLen > 0, "`stepTypeIds` should not be empty");

  for (let i = 0; i < stepsLen; i++) {
    const typeId = stepTypeIds[i];

    if (typeId < NODE_TYPES_COUNT) {
      // Enter visit - node type ID is used directly
      const node = stepData[i] as Node;
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
    } else if (typeId < EXIT_TYPE_ID_OFFSET) {
      // Call method (CFG event) - event type ID is in range NODE_TYPES_COUNT to EXIT_TYPE_ID_OFFSET-1
      const visit = visitors[typeId];
      if (visit !== null) {
        (visit as any).apply(undefined, stepData[i] as unknown[]);
      }
    } else {
      // Exit non-leaf node - type ID has EXIT_TYPE_ID_OFFSET added
      const actualTypeId = typeId - EXIT_TYPE_ID_OFFSET;
      const node = stepData[i] as Node;

      ancestors.shift();

      const enterExit = visitors[actualTypeId];
      if (enterExit !== null) {
        typeAssertIs<EnterExit>(enterExit);
        const { exit } = enterExit;
        if (exit !== null) exit(node);
      }
    }
  }

  // Reset SoA arrays
  stepTypeIds.length = 0;
  stepData.length = 0;
}

/**
 * Walk AST and put a list of all steps to walk AST into the SoA arrays.
 * @param ast - AST
 */
function prepareSteps(ast: Program) {
  debugAssert(stepTypeIds.length === 0, "`stepTypeIds` should be empty at start of `prepareSteps`");

  // Length of arrays after entering each node.
  // Used in debug build to check that no leaf nodes emit CFG events (see below).
  // Minifier removes this var in release build.
  let stepsLenAfterEnter = 0;

  // Create `CodePathAnalyzer`.
  // It stores steps to walk AST using the SoA (Struct of Arrays) pattern.
  //
  // Type ID encoding:
  // - Enter visits: type ID directly (0-164 for node types)
  // - CFG events: type ID directly (165-171 for event types)
  // - Exit visits: type ID + EXIT_TYPE_ID_OFFSET (256+)
  //
  // This allows us to:
  // 1. Avoid repeated string-to-number conversions during step execution
  // 2. Reduce memory allocation by using 2 flat arrays instead of step objects
  //
  // We could further improve performance in several ways (in ascending order of complexity):
  // * Copy `CodePathAnalyzer` code into this repo and rewrite it to work entirely with type IDs instead of strings.
  //
  // TODO: Apply these optimizations (or at least some of them).
  const analyzer = new CodePathAnalyzer({
    enterNode(node: Node) {
      const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;
      stepTypeIds.push(typeId);
      stepData.push(node);

      if (DEBUG) stepsLenAfterEnter = stepTypeIds.length;
    },

    leaveNode(node: Node) {
      const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;

      if (typeId >= LEAF_NODE_TYPES_COUNT) {
        // Non-leaf node - add exit step with offset
        stepTypeIds.push(typeId + EXIT_TYPE_ID_OFFSET);
        stepData.push(node);
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
        if (DEBUG && stepTypeIds.length !== stepsLenAfterEnter) {
          const eventNames: string[] = [];
          for (let j = stepsLenAfterEnter; j < stepTypeIds.length; j++) {
            const eventTypeId = stepTypeIds[j];
            // CFG events have type IDs >= NODE_TYPES_COUNT and < EXIT_TYPE_ID_OFFSET
            if (eventTypeId >= NODE_TYPES_COUNT && eventTypeId < EXIT_TYPE_ID_OFFSET) {
              // Find the event name by reverse lookup
              for (const [name, id] of NODE_TYPE_IDS_MAP) {
                if (id === eventTypeId) {
                  eventNames.push(name);
                  break;
                }
              }
            }
          }
          throw new Error(
            `CFG events emitted during visiting leaf node \`${node.type}\`: ${eventNames.join(", ")}`,
          );
        }
      }
    },

    emit(eventName: string, args: unknown[]) {
      const typeId = NODE_TYPE_IDS_MAP.get(eventName)!;
      stepTypeIds.push(typeId);
      stepData.push(args);
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
