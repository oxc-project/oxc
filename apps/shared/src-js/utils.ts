export const isObject = (v: unknown): v is object =>
  typeof v === "object" && v !== null && !Array.isArray(v);
