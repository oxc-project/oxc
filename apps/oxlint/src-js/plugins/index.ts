// Patch `WeakMap` before any plugins are loaded
import "./weak_map.ts";

export { lintFile } from "./lint.ts";
export { loadPlugin } from "./load.ts";
export { setupRuleConfigs } from "./config.ts";
