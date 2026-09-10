// Declares no `languages`, only a parser override, the shape import sorters use.
// Nothing can route to it, so oxfmt reports that it has no effect.
export const parsers = {
  babel: { parse: (text) => ({ text }), astFormat: "estree", locStart: () => 0, locEnd: () => 0 },
};
