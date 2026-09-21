// Stable re-export of `prettier/plugins/graphql`. See `libs/plugin-resolution.ts`.
// Prettier declares no default in its types, so the runtime default is read off
// the namespace instead of re-exported.
import * as mod from "prettier/plugins/graphql";

export * from "prettier/plugins/graphql";

const defaultExport: unknown = (mod as { default?: unknown }).default ?? mod;
export default defaultExport;
