declare module "*.css" with { type: "css"; "format": "module" } {
  export const stylesheet: CSSStyleSheet;
}
declare module "*.empty" with {} {}
declare module "*.text" with { type: "text" };
declare module "*.no-attributes" {}
