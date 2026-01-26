/*
 * CFG (control flow graph) aka Code Path Analysis (what ESLint calls it).
 *
 * Function to construct CFG, and walk AST and call visitors for CFG events.
 */

// @ts-expect-error - internal module of ESLint with no types
import CodePathAnalyzer from "../../node_modules/eslint/lib/linter/code-path-analysis/code-path-analyzer.js";

import visitorKeys from "../generated/keys.ts";
import {
  LEAF_NODE_TYPES_COUNT,
  NODE_TYPE_IDS_MAP,
  NODE_TYPES_COUNT,
  TYPE_IDS_COUNT,
} from "../generated/type_ids.ts";
import { ancestors } from "../generated/walk.js";
import { debugAssert, debugAssertIsFunction } from "../utils/asserts.ts";

import type { EnterExit } from "./visitor.ts";
import type { Node, Program } from "../generated/types.d.ts";
import type { CompiledVisitors } from "../generated/walk.js";

/**
 * Offset added to type IDs for exit visits to distinguish them from enter visits.
 * Using 256 as it's a power of 2 and larger than the maximum type ID (171).
 *
 * Type ID encoding:
 * - Enter visit (nodes): 0 to NODE_TYPES_COUNT - 1 (0-164)
 * - Call method (CFG events): NODE_TYPES_COUNT to TYPE_IDS_COUNT - 1 (165-171)
 * - Exit visit (non-leaf nodes): Node type ID + EXIT_TYPE_ID_OFFSET (256+)
 */
const EXIT_TYPE_ID_OFFSET = 256;

debugAssert(
  EXIT_TYPE_ID_OFFSET >= TYPE_IDS_COUNT,
  "`EXIT_TYPE_ID_OFFSET` must be >= `TYPE_IDS_COUNT`",
);

// Struct of Arrays (SoA) pattern for step storage.
// Using 2 arrays instead of an array of objects reduces object creation.

/**
 * Encoded type IDs for each step.
 * - For enter visits: Node type ID (0-164)
 * - For CFG events: Event type ID (165-171)
 * - For exit visits: Node type ID + `EXIT_TYPE_ID_OFFSET` (256+)
 */
const stepTypeIds: number[] = [];

/**
 * Step data for each step.
 * - For visit steps (enter/exit): AST node
 * - For call steps (CFG events): Array of arguments to call CFG event handler with
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
 * We could further improve its performance by copying ESLint's `CodePathAnalyzer` into this repo,
 * and rewriting it to work entirely with type IDs instead of strings.
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
    let typeId = stepTypeIds[i];

    if (typeId < NODE_TYPES_COUNT) {
      // Enter node. `typeId` is node type ID.
      const node = stepData[i] as Node;
      const visit = visitors[typeId];

      if (typeId < LEAF_NODE_TYPES_COUNT) {
        // Leaf node
        if (visit !== null) {
          debugAssertIsFunction(visit);
          visit(node);
        }
        // Don't add node to `ancestors`, because we don't visit leaf nodes on exit
      } else {
        // Non-leaf node
        if (visit !== null) {
          debugAssertIsEnterExitObject(visit);
          const { enter } = visit;
          if (enter !== null) enter(node);
        }

        ancestors.unshift(node);
      }
    } else if (typeId >= EXIT_TYPE_ID_OFFSET) {
      // Exit non-leaf node. `typeId` is node type ID + `EXIT_TYPE_ID_OFFSET`.
      typeId -= EXIT_TYPE_ID_OFFSET;
      const node = stepData[i] as Node;

      ancestors.shift();

      const enterExit = visitors[typeId];
      if (enterExit !== null) {
        debugAssertIsEnterExitObject(enterExit);
        const { exit } = enterExit;
        if (exit !== null) exit(node);
      }
    } else {
      // Call method (CFG event). `typeId` is event type ID.
      debugAssert(Array.isArray(stepData[i]), "`stepData` should contain an array for CFG events");

      const visit = visitors[typeId];
      if (visit !== null) {
        debugAssertIsFunction(visit);
        visit.apply(undefined, stepData[i]);
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
  debugAssert(stepData.length === 0, "`stepData` should be empty at start of `prepareSteps`");

  // Length of arrays after entering each node.
  // Used in debug build to check that no leaf nodes emit CFG events (see below).
  // Minifier removes this var in release build.
  let stepsLenAfterEnter = 0;

  // Create `CodePathAnalyzer`.
  // It stores steps to walk AST using the SoA (Struct of Arrays) pattern.
  //
  // Type ID encoding:
  // - Enter visits: Node type ID directly (0-164 for node types)
  // - CFG events: Node type ID directly (165-171 for event types)
  // - Exit visits: Event type ID + `EXIT_TYPE_ID_OFFSET` (256+)
  //
  // This allows us to:
  // 1. Avoid repeated `NODE_TYPE_IDS_MAP` hash map lookups during step execution.
  // 2. Reduce object creation by using 2 flat arrays instead of step objects.
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
          for (let i = stepsLenAfterEnter; i < stepTypeIds.length; i++) {
            const typeId = stepTypeIds[i];
            if (typeId < NODE_TYPES_COUNT) {
              eventNames.push(`enter ${NODE_TYPE_IDS_MAP.get((stepData[i] as Node).type)}`);
            } else if (typeId >= EXIT_TYPE_ID_OFFSET) {
              eventNames.push(`exit ${NODE_TYPE_IDS_MAP.get((stepData[i] as Node).type)}`);
            } else {
              const eventName = NODE_TYPE_IDS_MAP.entries().find(([, id]) => id === typeId)![0];
              eventNames.push(eventName);
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

  // Walk AST, calling `analyzer` methods for each node
  traverseNode(ast, analyzer.enterNode.bind(analyzer), analyzer.leaveNode.bind(analyzer));

  debugAssert(
    stepTypeIds.length === stepData.length,
    "`stepTypeIds` and `stepData` should have the same length",
  );
}

/**
 * Lightweight AST traverser for CFG building.
 * This is a simplified version that only calls enter/leave callbacks,
 * without building ancestors array or other overhead that ESLint's `Traverser` has.
 *
 * @param node - AST node to traverse
 * @param enter - Callback for entering a node
 * @param leave - Callback for leaving a node
 */
function traverseNode(node: Node, enter: (node: Node) => void, leave: (node: Node) => void): void {
  enter(node);

  const keys = visitorKeys[node.type as keyof typeof visitorKeys];
  const keysLen = keys.length;
  for (let i = 0; i < keysLen; i++) {
    const child = (node as any)[keys[i]] as Node | (Node | null)[] | null;

    if (child === null) continue;

    if (Array.isArray(child)) {
      const len = child.length;
      for (let i = 0; i < len; i++) {
        const element = child[i];
        if (element !== null) traverseNode(element, enter, leave);
      }
    } else {
      traverseNode(child, enter, leave);
    }
  }

  leave(node);
}

/**
 * Debug assert that `enterExit` is an `EnterExit` object.
 * In release build, this function does nothing and is removed entirely by minifier.
 * @param enterExit - Object
 * @throws {TypeError} If `enterExit` is not an `EnterExit` object in debug build
 */
export function debugAssertIsEnterExitObject(enterExit: unknown): asserts enterExit is EnterExit {
  if (!DEBUG) return;

  if (!isEnterExit(enterExit)) throw new TypeError("Expected to be an `EnterExit` object");
}

/**
 * Check if an object is an `EnterExit` object.
 * @param obj - Object
 * @returns `true` if `obj` is an `EnterExit` object, `false` otherwise
 */
function isEnterExit(obj: any): obj is EnterExit {
  if (obj === null) return false;
  if (typeof obj !== "object") return false;
  if (Object.keys(obj).length !== 2) return false;
  if (obj.enter !== null && typeof obj.enter !== "function") return false;
  if (obj.exit !== null && typeof obj.exit !== "function") return false;
  return true;
}
