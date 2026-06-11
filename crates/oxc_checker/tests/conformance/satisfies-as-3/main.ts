import { ok_config, ok_levels, Level } from "./data";

// literal types survive the as const export
export const ok_name: "oxc" = ok_config.name;
export const ok_version: 3 = ok_config.version;
export const ok_first_flag: "fast" = ok_config.flags[0];

// tuple element types are exact literals
export const ok_last_level: 30 = ok_levels[2];
export const ok_some_level: Level = 20;

interface Endpoint {
  path: string;
  method: "GET" | "POST";
}

// as const + satisfies: checked against Endpoint, literals preserved
// (kept non-exported: only exported consts need annotations under isolatedDeclarations)
const ok_endpoint = {
  path: "/users",
  method: "GET",
} as const satisfies Endpoint;

export const ok_method: "GET" = ok_endpoint.method;
export const ok_widened_path: string = ok_endpoint.path;
