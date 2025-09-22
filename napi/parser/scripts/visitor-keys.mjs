import { visitorKeys } from '@typescript-eslint/visitor-keys';

const keys = Object.entries(visitorKeys).map(([name, keys]) => ({ name, keys }));
console.log(JSON.stringify(keys));
