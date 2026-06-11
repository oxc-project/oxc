// as const on an object export: every property becomes a readonly literal
export const ok_config = {
  name: "oxc",
  version: 3,
  flags: ["fast", "safe"],
} as const;

// as const on an array export: readonly tuple of literals
export const ok_levels = [10, 20, 30] as const;

export type Level = (typeof ok_levels)[number];
