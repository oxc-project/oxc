// second hop of the export * chain: base -> middle -> top
export * from "./middle";
export type { Config as TopConfig } from "./middle";
