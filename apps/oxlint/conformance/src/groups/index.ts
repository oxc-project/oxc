import type { TestGroup } from "../index.ts";

import eslint from "./eslint.ts";
import reactHooks from "./react_hooks.ts";
import stylistic from "./stylistic.ts";

export const TEST_GROUPS: TestGroup[] = [eslint, reactHooks, stylistic];
