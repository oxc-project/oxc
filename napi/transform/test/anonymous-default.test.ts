import { expect, it } from "vitest";
import { HelperMode, transformSync } from "../index";

const helperRoot = new URL("../../../npm/runtime/src/helpers/esm/", import.meta.url);
const helperImports = ["decorate", "decorateParam", "defineProperty"]
  .map((name) => `import ${name} from ${JSON.stringify(new URL(`${name}.js`, helperRoot).href)};`)
  .join("\n");

it.each([
  ["es2015", "class"],
  ["es2015", "constructor"],
  ["es2015", "member"],
  ["es2022", "class"],
  ["es2022", "constructor"],
  ["es2022", "member"],
])("runs anonymous decorated default exports (%s, %s)", async (target, kind) => {
  const input = `
      export const calls: string[] = [];
      function dec(...args: any[]): any {
        calls.push(args.length === 1 ? "class" : typeof args[2] === "number" ? "constructor" : "member");
      }
      ${kind === "class" ? "@dec" : ""}
      export default class {
        ${kind === "member" ? "@dec" : ""} static value = 42;
        field = 1;
        ${kind === "constructor" ? "constructor(@dec value: number) {}" : ""}
      }
    `;
  const { code, errors } = transformSync("test.ts", input, {
    target,
    decorator: { legacy: true },
    helpers: { mode: HelperMode.External },
  });
  expect(errors).toEqual([]);
  const source = `${helperImports}\nconst babelHelpers = { decorate, decorateParam, defineProperty };\n${code}`;
  const { default: Class, calls } = await import(
    `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`
  );
  expect(Class.value).toBe(42);
  expect(new Class().field).toBe(1);
  expect(calls).toEqual([kind]);
});
