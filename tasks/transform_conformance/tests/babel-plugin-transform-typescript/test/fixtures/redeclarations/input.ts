// CASE 1: redeclaration of VariableDeclaration
import { A } from './a';
const A: A = 0;
export {A};

// CASE 2: redeclaration of TypeAlias
import { T } from "./t";
type T = number;
export { T }

// CASE 3: redeclaration of VariableDeclaration and TypeAlias
import { B } from './b';
const B: B = 0;
type B = number;
export { B }
