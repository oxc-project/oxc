import { visitorKeys } from "@typescript-eslint/visitor-keys";

const keys = Object.entries(visitorKeys).map(([name, keys]) => ({ name, keys }));
// oxlint-disable-next-line no-console
console.log(JSON.stringify(keys));
