// Patch `WeakMap` before any plugins are loaded
import "./weak_map.ts";

export { lintFile } from "./lint.ts";
export { lintFileWithJsParser } from "./lint_js_parser.ts";
export { loadPlugin } from "./load.ts";
export { loadParser } from "./parsers.ts";
export { setupRuleConfigs } from "./config.ts";
