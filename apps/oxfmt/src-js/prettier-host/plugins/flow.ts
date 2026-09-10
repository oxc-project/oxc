// Stable re-export of `prettier/plugins/flow`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/flow";

export * from "prettier/plugins/flow";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
