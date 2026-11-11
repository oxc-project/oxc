// A key is short by its display width (wide characters count as 2), quoted or not.
// `古体诗` is 6 wide, so like `abcdef` it is not short, and the value moves to the next line.
x = {
  古体诗: "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned-about",
  "古体诗": "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned-about",
  [古体]: "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned-about",
  abcdef: "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned-about",
  古: "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned-about-xxxxxx",
};
var { 古体诗: 古 = "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned" } = x;
({ 古体诗: 古 = "https://prettier.io/docs/en/rationale.html#what-prettier-is-concerned" } = x);
