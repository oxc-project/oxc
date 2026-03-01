// This file will trigger the no-multi-comp rule, that's expected.
const ComponentOne = () => <div>A</div>;
const ComponentTwo = () => <div>B</div>;

// This triggers typescript/no-inferrable-types, that's expected.
// But it should NOT trigger no-unused-vars since correctness is off.
const a: number = 5;

export { ComponentOne, ComponentTwo };
