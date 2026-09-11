declare module "*.css" with { readonly type: "css" } {
  const stylesheet: CSSStyleSheet;
  export default stylesheet;
}

declare module "*.json" with { readonly type: "json", readonly kind: "data" } {}

declare module "*.config" with { readonly "mode": `strict` };
