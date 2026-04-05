import { NODE_TYPE_IDS_MAP, NODE_TYPES_COUNT } from "../generated/type_ids.ts";
import { ancestors } from "../generated/walk.js";
import { debugAssert } from "../utils/asserts.ts";
import { EXIT_FLAG, IDENTIFIER_COUNT_INCREMENT, parseSelector, wrapVisitFnWithSelectorMatch } from "./selector.ts";
import { getVisitorKeysForNode } from "./source_code.ts";
// @ts-expect-error - internal module of ESLint with no types
import CodePathAnalyzer from "../../node_modules/eslint/lib/linter/code-path-analysis/code-path-analyzer.js";

import type { VisitorObject } from "../generated/visitor.d.ts";
import type { Node as TraversalNode } from "../generated/types.d.ts";
import type { Node as VisitNode } from "./types.ts";
import type { VisitFn, EnterExit, CfgVisitFn } from "./visitor.ts";

interface VisitProp {
  fn: VisitFn;
  specificity: number;
  selectorStr: string;
}

interface CompilingEntry {
  enter: VisitProp[];
  exit: VisitProp[];
}

export interface ExternalCompiledVisitor {
  byType: Map<string, CompilingEntry>;
  wildcard: CompilingEntry;
  selectors: CompilingEntry;
  cfg: Map<string, CfgVisitFn>;
  hasCfg: boolean;
  cache: Map<string, EnterExit | null>;
}

type ExternalCfgStep =
  | { kind: "enter"; node: VisitNode }
  | { kind: "exit"; node: VisitNode }
  | { kind: "cfg"; eventName: string; args: unknown[] };

const DIRECT_VISITOR_NAME_PATTERN = /^[A-Za-z_$][A-Za-z0-9_$]*$/;

function createCompilingEntry(): CompilingEntry {
  return { enter: [], exit: [] };
}

function addVisitProp(entry: CompilingEntry, visitProp: VisitProp, isExit: boolean): void {
  (isExit ? entry.exit : entry.enter).push(visitProp);
}

function isCfgVisitorName(name: string): boolean {
  const typeId = NODE_TYPE_IDS_MAP.get(name);
  return typeId !== undefined && typeId >= NODE_TYPES_COUNT;
}

export function compileExternalVisitors(
  visitors: VisitorObject[],
): ExternalCompiledVisitor | null {
  const byType = new Map<string, CompilingEntry>();
  const wildcard = createCompilingEntry();
  const selectors = createCompilingEntry();
  const cfgEntries = new Map<string, CfgVisitFn[]>();
  let hasActiveVisitors = false;
  let hasCfg = false;

  for (let visitorIndex = 0, visitorsLen = visitors.length; visitorIndex < visitorsLen; visitorIndex++) {
    const visitor = visitors[visitorIndex];
    if (visitor === null || typeof visitor !== "object") {
      throw new TypeError("Visitor returned from `create` method must be an object");
    }

    const keys = Object.keys(visitor);
    if (keys.length === 0) continue;
    hasActiveVisitors = true;

    for (let keyIndex = 0, keysLen = keys.length; keyIndex < keysLen; keyIndex++) {
      const rawName = keys[keyIndex];
      const visitFn = visitor[rawName] as VisitFn;
      if (typeof visitFn !== "function") {
        throw new TypeError(`'${rawName}' property of visitor object is not a function`);
      }

      let name = rawName;
      let specificity = 0;
      const isExit = name.endsWith(":exit");
      if (isExit) {
        name = name.slice(0, -5);
        specificity = EXIT_FLAG;
      }

      if (isCfgVisitorName(name)) {
        if (isExit) throw new Error(`Invalid visitor key: \`${name}:exit\``);

        let cfgVisitFns = cfgEntries.get(name);
        if (cfgVisitFns === undefined) {
          cfgVisitFns = [];
          cfgEntries.set(name, cfgVisitFns);
        }
        cfgVisitFns.push(visitFn as CfgVisitFn);
        hasCfg = true;
        continue;
      }

      if (name === "*") {
        addVisitProp(wildcard, { fn: visitFn, specificity, selectorStr: name }, isExit);
        continue;
      }

      if (DIRECT_VISITOR_NAME_PATTERN.test(name)) {
        let entry = byType.get(name);
        if (entry === undefined) {
          entry = createCompilingEntry();
          byType.set(name, entry);
        }
        addVisitProp(
          entry,
          {
            fn: visitFn,
            specificity: specificity | IDENTIFIER_COUNT_INCREMENT,
            selectorStr: name,
          },
          isExit,
        );
        continue;
      }

      const selector = parseSelector(name);
      addVisitProp(
        selectors,
        {
          fn: wrapVisitFnWithSelectorMatch(visitFn, selector.esquerySelector),
          specificity: specificity | selector.specificity,
          selectorStr: name,
        },
        isExit,
      );
    }
  }

  if (hasActiveVisitors === false) return null;

  const cfg = new Map<string, CfgVisitFn>();
  for (const [eventName, cfgVisitFns] of cfgEntries) {
    const mergedCfgVisitFn = mergeCfgVisitFns(cfgVisitFns);
    if (mergedCfgVisitFn !== null) cfg.set(eventName, mergedCfgVisitFn);
  }

  return {
    byType,
    wildcard,
    selectors,
    cfg,
    hasCfg,
    cache: new Map(),
  };
}

function sortVisitProps(a: VisitProp, b: VisitProp): number {
  const diff = a.specificity - b.specificity;
  if (diff !== 0) return diff;
  return a.selectorStr === b.selectorStr ? 0 : a.selectorStr < b.selectorStr ? -1 : 1;
}

function mergeVisitProps(visitProps: VisitProp[]): VisitFn | null {
  const numVisitFns = visitProps.length;
  if (numVisitFns === 0) return null;
  if (numVisitFns === 1) return visitProps[0]!.fn;

  const sortedVisitProps = [...visitProps].sort(sortVisitProps);
  return (node: VisitNode) => {
    for (let i = 0; i < numVisitFns; i++) sortedVisitProps[i]!.fn(node);
  };
}

function mergeCfgVisitFns(cfgVisitFns: CfgVisitFn[]): CfgVisitFn | null {
  const cfgVisitFnsLen = cfgVisitFns.length;
  if (cfgVisitFnsLen === 0) return null;
  if (cfgVisitFnsLen === 1) return cfgVisitFns[0]!;

  return (...args: unknown[]) => {
    for (let i = 0; i < cfgVisitFnsLen; i++) cfgVisitFns[i]!(...args);
  };
}

function getMergedEntryForType(
  type: string,
  visitor: ExternalCompiledVisitor,
): EnterExit | null {
  const cachedEntry = visitor.cache.get(type);
  if (cachedEntry !== undefined) return cachedEntry;

  const directEntry = visitor.byType.get(type);
  const enter = mergeVisitProps([
    ...(visitor.wildcard.enter),
    ...(directEntry?.enter ?? []),
    ...(visitor.selectors.enter),
  ]);
  const exit = mergeVisitProps([
    ...(visitor.wildcard.exit),
    ...(directEntry?.exit ?? []),
    ...(visitor.selectors.exit),
  ]);

  const mergedEntry = enter === null && exit === null ? null : { enter, exit };
  visitor.cache.set(type, mergedEntry);
  return mergedEntry;
}

function walkExternalNode(
  node: unknown,
  visitors: ExternalCompiledVisitor,
): void {
  if (node == null) return;

  if (Array.isArray(node)) {
    for (let i = 0, len = node.length; i < len; i++) walkExternalNode(node[i], visitors);
    return;
  }

  if (typeof node !== "object") return;

  const type = (node as { type?: unknown }).type;
  if (typeof type !== "string") return;

  const nodeRecord = node as Record<string, unknown> & { type: string };
  const enterExit = getMergedEntryForType(type, visitors);
  let exit: VisitFn | null = null;
  if (enterExit !== null) {
    exit = enterExit.exit;
    enterExit.enter?.(nodeRecord as unknown as VisitNode);
  }

  ancestors.unshift(nodeRecord as unknown as TraversalNode);
  const ancestorsLen = DEBUG ? ancestors.length : 0;

  const keys = getVisitorKeysForNode(nodeRecord);
  for (let i = 0, len = keys.length; i < len; i++) {
    walkExternalNode(nodeRecord[keys[i]!], visitors);
  }

  debugAssert(
    ancestors.length === ancestorsLen,
    `\`ancestors\` is out of sync with external traversal while visiting \`${type}\``,
  );
  ancestors.shift();
  exit?.(nodeRecord as unknown as VisitNode);
}

export function walkExternalProgram(
  program: VisitNode,
  visitors: ExternalCompiledVisitor,
): void {
  walkExternalNode(program, visitors);
}

function prepareExternalCfgSteps(program: VisitNode): ExternalCfgStep[] {
  const steps: ExternalCfgStep[] = [];

  const analyzer = new CodePathAnalyzer({
    enterNode(node: VisitNode) {
      steps.push({ kind: "enter", node });
    },

    leaveNode(node: VisitNode) {
      steps.push({ kind: "exit", node });
    },

    emit(eventName: string, args: unknown[]) {
      steps.push({ kind: "cfg", eventName, args });
    },
  });

  traverseExternalNodeWithCfg(
    program,
    analyzer.enterNode.bind(analyzer) as (node: VisitNode) => void,
    analyzer.leaveNode.bind(analyzer) as (node: VisitNode) => void,
  );

  return steps;
}

function traverseExternalNodeWithCfg(
  node: unknown,
  enter: (node: VisitNode) => void,
  leave: (node: VisitNode) => void,
): void {
  if (node == null) return;

  if (Array.isArray(node)) {
    for (let i = 0, len = node.length; i < len; i++) {
      traverseExternalNodeWithCfg(node[i], enter, leave);
    }
    return;
  }

  if (typeof node !== "object") return;

  const type = (node as { type?: unknown }).type;
  if (typeof type !== "string") return;

  const nodeRecord = node as Record<string, unknown> & { type: string };
  enter(nodeRecord as unknown as VisitNode);

  const keys = getVisitorKeysForNode(nodeRecord);
  for (let i = 0, len = keys.length; i < len; i++) {
    traverseExternalNodeWithCfg(nodeRecord[keys[i]!], enter, leave);
  }

  leave(nodeRecord as unknown as VisitNode);
}

export function walkExternalProgramWithCfg(
  program: VisitNode,
  visitors: ExternalCompiledVisitor,
): void {
  const steps = prepareExternalCfgSteps(program);

  for (let i = 0, stepsLen = steps.length; i < stepsLen; i++) {
    const step = steps[i]!;

    if (step.kind === "cfg") {
      visitors.cfg.get(step.eventName)?.(...step.args);
      continue;
    }

    const node = step.node as unknown as TraversalNode;

    if (step.kind === "enter") {
      const enterExit = getMergedEntryForType(step.node.type, visitors);
      enterExit?.enter?.(step.node);
      ancestors.unshift(node);
      continue;
    }

    debugAssert(
      ancestors[0] === node,
      `\`ancestors\` is out of sync with external CFG traversal while exiting \`${step.node.type}\``,
    );
    ancestors.shift();

    const enterExit = getMergedEntryForType(step.node.type, visitors);
    enterExit?.exit?.(step.node);
  }
}
