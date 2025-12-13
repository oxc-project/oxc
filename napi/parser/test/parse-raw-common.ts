// Constants used in both main thread and worker in raw transfer tests.

import { join as pathJoin } from "node:path";

export const TEST_TYPE_TEST262 = 0;
export const TEST_TYPE_JSX = 1;
export const TEST_TYPE_TS = 2;
export const TEST_TYPE_FIXTURE = 3;
export const TEST_TYPE_INLINE_FIXTURE = 4;

export const TEST_TYPE_MAIN_MASK = 7;
export const TEST_TYPE_RANGE_PARENT = 8;
export const TEST_TYPE_LAZY = 16;
export const TEST_TYPE_PRETTY = 32;

export const ROOT_DIR_PATH = pathJoin(import.meta.dirname, "../../..");
export const TARGET_DIR_PATH = pathJoin(ROOT_DIR_PATH, "target");
export const TEST262_SHORT_DIR_PATH = "tasks/coverage/test262/test";
export const TEST262_DIR_PATH = pathJoin(ROOT_DIR_PATH, TEST262_SHORT_DIR_PATH);
export const TS_SHORT_DIR_PATH = "tasks/coverage/typescript";
export const TS_DIR_PATH = pathJoin(ROOT_DIR_PATH, TS_SHORT_DIR_PATH);
export const ACORN_TEST262_DIR_PATH = pathJoin(
  ROOT_DIR_PATH,
  "tasks/coverage/estree-conformance/tests/test262/test",
);
export const JSX_SHORT_DIR_PATH = "tasks/coverage/estree-conformance/tests/acorn-jsx/pass";
export const JSX_DIR_PATH = pathJoin(ROOT_DIR_PATH, JSX_SHORT_DIR_PATH);
const TS_ESTREE_SHORT_DIR_PATH = "tasks/coverage/estree-conformance/tests/typescript";
export const TS_ESTREE_DIR_PATH = pathJoin(ROOT_DIR_PATH, TS_ESTREE_SHORT_DIR_PATH);
export const TEST262_SNAPSHOT_PATH = pathJoin(
  ROOT_DIR_PATH,
  "tasks/coverage/snapshots/estree_test262.snap",
);
export const JSX_SNAPSHOT_PATH = pathJoin(
  ROOT_DIR_PATH,
  "tasks/coverage/snapshots/estree_acorn_jsx.snap",
);
export const TS_SNAPSHOT_PATH = pathJoin(
  ROOT_DIR_PATH,
  "tasks/coverage/snapshots/estree_typescript.snap",
);
