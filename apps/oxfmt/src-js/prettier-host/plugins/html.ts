// Stable re-export of `prettier/plugins/html`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/html";

export * from "prettier/plugins/html";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
