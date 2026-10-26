declare module "*.config" with { mode: `strict`, type: "config" } {}
declare module "*.css" with { type: `${kind}` } {}
declare module "*.text" with { type: "text" };
