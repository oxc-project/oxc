/*
 * CFG (control flow graph) aka Code Path Analysis (what ESLint calls it).
 *
 * Function to construct CFG, and walk AST and call visitors for CFG events.
 */

// @ts-expect-error - internal module of ESLint with no types
import CodePathAnalyzer from "../../node_modules/eslint/lib/linter/code-path-analysis/code-path-analyzer.js";

import visitorKeys from "../generated/keys.ts";
import { LEAF_NODE_TYPES_COUNT, NODE_TYPE_IDS_MAP } from "../generated/type_ids.ts";
import { ancestors } from "../generated/walk.js";
import { debugAssert, typeAssertIs } from "../utils/asserts.ts";

import type { EnterExit, VisitFn } from "./visitor.ts";
import type { Node, Program } from "../generated/types.d.ts";
import type { CompiledVisitors } from "../generated/walk.js";

/**
 * Step type encoding:
 * - 0 = enter visit (visiting a node, enter phase)
 * - 1 = exit visit (visiting a node, exit phase)
 * - 2 = call method (CFG event)
 */
const STEP_TYPE_ENTER_VISIT = 0;
const STEP_TYPE_EXIT_VISIT = 1;
const STEP_TYPE_CALL_METHOD = 2;

// Struct of Arrays (SoA) pattern for step storage.
// Using separate arrays for each property improves memory locality and V8 optimization.

/** Step types: 0=enter visit, 1=exit visit, 2=call method */
const stepTypes: number[] = [];

/** For visit steps: target node. For call steps: null */
const stepTargets: (Node | null)[] = [];

/** Pre-computed type IDs (node type ID or CFG event ID) */
const stepTypeIds: number[] = [];

/** For call steps only: args array. For visit steps: null */
const stepArgs: (unknown[] | null)[] = [];

/**
 * Reset state for walking AST with CFG.
 *
 * If walking AST completes without error, `walkProgramWithCfg` will reset the state itself.
 * So it's only necessary to call this function if an error occurs during AST walking.
 */
export function resetCfgWalk(): void {
  stepTypes.length = 0;
  stepTargets.length = 0;
  stepTypeIds.length = 0;
  stepArgs.length = 0;
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
 *    This list is stored in the SoA arrays (stepTypes, stepTargets, stepTypeIds, stepArgs).
 *
 * 2. Visit AST with provided visitor.
 *    Run through the steps, in order, calling visit functions for each step.
 *
 * @param ast - AST
 * @param visitors - Visitors array
 */
export function walkProgramWithCfg(ast: Program, visitors: CompiledVisitors): void {
  // Get the steps that need to be run to walk the AST
  prepareSteps(ast);

  // Walk the AST
  const stepsLen = stepTypes.length;
  debugAssert(stepsLen > 0, "`stepTypes` should not be empty");

  for (let i = 0; i < stepsLen; i++) {
    const stepType = stepTypes[i];
    const typeId = stepTypeIds[i];

    if (stepType === STEP_TYPE_ENTER_VISIT) {
      // Enter node - can be leaf or non-leaf node
      const node = stepTargets[i]!;
      const visit = visitors[typeId];

      if (typeId < LEAF_NODE_TYPES_COUNT) {
        // Leaf node
        if (visit != null) {
          typeAssertIs<VisitFn>(visit);
          visit(node);
        }
        // Don't add node to `ancestors`, because we don't visit them on exit
      } else {
        // Non-leaf node
        if (visit != null) {
          typeAssertIs<EnterExit>(visit);
          const { enter } = visit;
          if (enter != null) enter(node);
        }

        ancestors.unshift(node);
      }
    } else if (stepType === STEP_TYPE_EXIT_VISIT) {
      // Exit non-leaf node
      ancestors.shift();

      const enterExit = visitors[typeId];
      if (enterExit != null) {
        typeAssertIs<EnterExit>(enterExit);
        const { exit } = enterExit;
        if (exit != null) exit(stepTargets[i]!);
      }
    } else {
      // Call method (CFG event)
      const visit = visitors[typeId];
      if (visit != null && typeof visit === "function") {
        visit.apply(undefined, stepArgs[i]);
      }
    }
  }

  // Reset all SoA arrays
  stepTypes.length = 0;
  stepTargets.length = 0;
  stepTypeIds.length = 0;
  stepArgs.length = 0;
}

// Pre-computed array check for performance
const { isArray } = Array;

/**
 * Lightweight AST traverser for CFG building.
 * This is a simplified version that only calls enter/leave callbacks,
 * without building ancestors array or other overhead.
 *
 * @param node - AST node to traverse
 * @param enter - Callback for entering a node
 * @param leave - Callback for leaving a node
 */
function traverseNode(
  node: Node | null | undefined,
  enter: (node: Node) => void,
  leave: (node: Node) => void,
): void {
  if (node == null) return;

  if (isArray(node)) {
    const len = node.length;
    for (let i = 0; i < len; i++) {
      traverseNode(node[i], enter, leave);
    }
    return;
  }

  // Enter the node
  enter(node);

  // Traverse children using visitorKeys
  const keys = visitorKeys[node.type as keyof typeof visitorKeys];
  if (keys != null) {
    const keysLen = keys.length;
    for (let i = 0; i < keysLen; i++) {
      const child = (node as any)[keys[i]];
      if (child != null) {
        traverseNode(child, enter, leave);
      }
    }
  }

  // Leave the node
  leave(node);
}

/**
 * Walk AST and put a list of all steps to walk AST into the SoA arrays.
 * @param ast - AST
 */
function prepareSteps(ast: Program) {
  debugAssert(stepTypes.length === 0, "`stepTypes` should be empty at start of `prepareSteps`");

  // Length of step arrays after entering each node.
  // Used in debug build to check that no leaf nodes emit CFG events (see below).
  // Minifier removes this var in release build.
  let stepsLenAfterEnter = 0;

  // Create `CodePathAnalyzer`.
  // It stores steps to walk AST in the SoA arrays.
  const analyzer = new CodePathAnalyzer({
    enterNode(node: Node) {
      const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;

      stepTypes.push(STEP_TYPE_ENTER_VISIT);
      stepTargets.push(node);
      stepTypeIds.push(typeId);
      stepArgs.push(null);

      if (DEBUG) stepsLenAfterEnter = stepTypes.length;
    },

    leaveNode(node: Node) {
      const typeId = NODE_TYPE_IDS_MAP.get(node.type)!;

      if (typeId >= LEAF_NODE_TYPES_COUNT) {
        // Non-leaf node
        stepTypes.push(STEP_TYPE_EXIT_VISIT);
        stepTargets.push(node);
        stepTypeIds.push(typeId);
        stepArgs.push(null);
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
        if (DEBUG && stepTypes.length !== stepsLenAfterEnter) {
          const eventNames: string[] = [];
          for (let j = stepsLenAfterEnter; j < stepTypes.length; j++) {
            if (stepTypes[j] === STEP_TYPE_CALL_METHOD) {
              // Get event name from the CFG event ID
              // We need to reverse lookup the event name from typeId
              // Since stepArgs contains the args for CFG events, we use a different approach
              const eventTypeId = stepTypeIds[j];
              // Find the event name by iterating NODE_TYPE_IDS_MAP
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

      stepTypes.push(STEP_TYPE_CALL_METHOD);
      stepTargets.push(null);
      stepTypeIds.push(typeId);
      stepArgs.push(args);
    },
  });

  // Walk AST using our lightweight traverser instead of ESLint's Traverser
  traverseNode(
    ast,
    (node) => analyzer.enterNode(node),
    (node) => analyzer.leaveNode(node),
  );
}
