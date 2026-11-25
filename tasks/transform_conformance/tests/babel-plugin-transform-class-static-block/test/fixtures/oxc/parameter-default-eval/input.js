function preserveEval(
  { value = (eval("var exposed = 7"), class { static {} }) } = {},
  other = exposed,
) {
  return other;
}
const result = preserveEval({});
