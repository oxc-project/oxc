// Stable re-export of `prettier/plugins/typescript`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/typescript";

export * from "prettier/plugins/typescript";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
