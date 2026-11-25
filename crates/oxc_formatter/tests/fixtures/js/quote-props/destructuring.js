const { text, 'class': c, 'data-tid': d } = props;
const { a, 'b': b } = props;
function f({ text, 'class': c, 'data-tid': d }) {}
({ text, 'class': c, 'data-tid': d } = props);
({ text, class: c, 'data-tid': d } = props);
const { 1: a, 'a-b': b } = p;
({ 1: a, 'a-b': b } = p);

// Nested patterns and objects are handled independently
const { 'data-tid': { inner }, x = { y: 1 } } = props;
const o = { 'a-b': 1, f({ z: w }) {} };
