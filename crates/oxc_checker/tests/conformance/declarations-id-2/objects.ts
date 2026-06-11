const label = "widget";
const extras = { color: "red" };

// Clean: object literal with literal-typed properties.
export const ok_obj = { id: 1, name: "widget" };

// Clean: annotation removes the need for inference.
export const ok_shorthand_annotated: { label: string } = { label };

// Error: shorthand property in exported object literal.
export const bad_shorthand = { label };

// Error: spread assignment in exported object literal.
export const bad_spread = { ...extras, id: 2 };
