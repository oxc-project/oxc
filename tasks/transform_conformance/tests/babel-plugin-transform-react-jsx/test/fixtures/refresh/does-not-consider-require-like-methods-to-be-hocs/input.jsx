// None of these were declared in this file.
// It's bad to register them because that would trigger
// modules to execute in an environment with inline requires.
// So we expect the transform to skip all of them even though
// they are used in JSX.

const A = require('A');
const B = foo ? require('X') : require('Y');
const C = requireCond(gk, 'C');
const D = import('D');

export default function App() {
  return (
    <div>
      <A />
      <B />
      <C />
      <D />
    </div>
  );
}
