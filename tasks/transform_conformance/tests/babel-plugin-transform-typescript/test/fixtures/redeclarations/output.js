// CASE 1: redeclaration of VariableDeclaration
const A = 0;
export { A };

// CASE 2: redeclaration of TypeAlias
import { T } from "./t";
export { T };

// CASE 3: redeclaration of VariableDeclaration and TypeAlias
const B = 0;
export { B };
