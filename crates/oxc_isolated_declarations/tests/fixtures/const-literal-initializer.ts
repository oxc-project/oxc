// Keep the literal initializer, like tsc
export const directTrue = true;
export const directFalse = false;
export const parenTrue = (true);
export const parenNumber = (1);
export const negativeNumber = -1;
export const directString = "x";
export const templateNoExpr = `x`;
export const directBigInt = 1n;
export const negativeBigInt = -1n;

export namespace ns {
  export const nested = true;
}

const local = false;
export { local };

// Widen to the primitive type, like tsc
export let letTrue = true;
export var varTrue = true;
export let letNumber = 1;

// Keep the type annotation
export const annotated: boolean = true;
