import type { TestGroup } from "../index.ts";

import eslint from "./eslint.ts";
import reactHooks from "./react_hooks.ts";
import stylistic from "./stylistic.ts";
import sonarjs from "./sonarjs.ts";
import e18e from "./e18e.ts";

export const TEST_GROUPS: TestGroup[] = [eslint, reactHooks, stylistic, sonarjs, e18e];
