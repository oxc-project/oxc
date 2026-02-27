import { foo } from './dep.js';
import { bar } from './dep2.js';

export const result = foo + bar;
export default result;
