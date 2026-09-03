declare module "*.css" with { type: "css" } {
  const stylesheet: CSSStyleSheet;
  export default stylesheet;
}

declare module "*.sqlite" with { type: "sqlite"; embed: "true" } {
  const contents: Uint8Array;
  export default contents;
}

declare module "*.empty" with {} {}

declare module "*.text" with { "type": "text" };

declare module "*.config" with { readonly mode: `strict` } {}
