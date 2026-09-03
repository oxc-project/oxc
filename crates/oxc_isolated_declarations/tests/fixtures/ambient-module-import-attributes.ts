declare module "*.css" with { type: "css" } {
  export const stylesheet: CSSStyleSheet;
}

declare module "*.text" with { type: "text" };

declare module "*.config" with { readonly mode: `strict` } {}
