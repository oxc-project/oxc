import * as wasm from "./oxc_parser_wasm_bg.wasm";
import { __wbg_set_wasm } from "./oxc_parser_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./oxc_parser_wasm_bg.js";
