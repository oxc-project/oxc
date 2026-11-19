import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

const require = createRequire(import.meta.url);
const { exports: packageExports } = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
);
const upstream = JSON.parse(readFileSync(new URL("../upstream.json", import.meta.url), "utf8"));

test("vendored helpers match the upstream manifest and documented patches", () => {
  const helperDir = new URL("../src/helpers/", import.meta.url);
  for (const [path, hash] of Object.entries(upstream.files)) {
    const contents = readFileSync(new URL(path, helperDir));
    const expected = upstream.patches[path]?.sha256 ?? hash;
    assert.equal(createHash("sha256").update(contents).digest("hex"), expected, path);
    if (!path.includes("/") && path.endsWith(".js")) {
      const name = path.slice(0, -3);
      const expectedExport = upstream.internalHelpers.includes(path)
        ? undefined
        : {
            node: `./src/helpers/${path}`,
            import: `./src/helpers/esm/${path}`,
            default: `./src/helpers/${path}`,
          };
      assert.deepEqual(packageExports[`./helpers/${name}`], expectedExport);
      assert.equal(packageExports[`./helpers/esm/${name}`], `./src/helpers/esm/${path}`);
    }
  }
  const expectedFiles = [
    ...Object.keys(upstream.files),
    ...[...upstream.customHelpers, ...upstream.legacyHelpers].flatMap((name) => [
      name,
      `esm/${name}`,
    ]),
  ].sort();
  assert.deepEqual(
    readdirSync(helperDir, { recursive: true })
      .filter((name) => /\.(js|json)$/.test(name))
      .sort(),
    expectedFiles,
  );
  assert.equal(
    packageExports["./helpers/tsRewriteRelativeImportExtensions"].import,
    "./src/helpers/esm/tsRewriteRelativeImportExtensions.js",
  );
});

const helperNames = Object.keys(packageExports)
  .filter((name) => name.startsWith("./helpers/") && !name.startsWith("./helpers/esm/"))
  .map((name) => name.slice("./helpers/".length));

test("all helper exports load in CommonJS and ESM", async () => {
  await Promise.all(
    helperNames.map(async (name) => {
      const specifier = `@oxc-project/runtime/helpers/${name}`;
      const commonjs = require(specifier);
      assert.equal(typeof commonjs, "function", specifier);
      assert.equal(commonjs.default, commonjs, specifier);
      assert.equal((await import(specifier)).default, commonjs, specifier);
    }),
  );
  await Promise.all(
    Object.keys(packageExports)
      .filter((name) => name.startsWith("./helpers/esm/"))
      .map(async (name) => {
        const specifier = `@oxc-project/runtime/${name.slice(2)}`;
        assert.equal(typeof (await import(specifier)).default, "function", specifier);
      }),
  );
});

for (const mode of ["commonjs", "esm"]) {
  const load = async (name) =>
    mode === "commonjs"
      ? require(`@oxc-project/runtime/helpers/${name}`)
      : (await import(`@oxc-project/runtime/helpers/esm/${name}`)).default;

  test(`${mode}: object rest works without Array.prototype.includes`, async () => {
    const objectWithoutProperties = await load("objectWithoutProperties");
    const objectWithoutPropertiesLoose = await load("objectWithoutPropertiesLoose");
    const excluded = runInNewContext("Array.prototype.includes = undefined; ['excluded']");
    const symbol = Symbol("kept");
    const source = Object.assign(Object.create({ inherited: 0 }), {
      excluded: 1,
      kept: 2,
      [symbol]: 3,
    });
    Object.defineProperty(source, "hidden", { value: 4 });
    assert.deepEqual(objectWithoutProperties(source, excluded), { kept: 2, [symbol]: 3 });
    assert.deepEqual(objectWithoutPropertiesLoose(source, excluded), { kept: 2 });
    assert.deepEqual(objectWithoutProperties(source, ["excluded", symbol]), { kept: 2 });
  });

  test(`${mode}: async-from-sync throw calls throw instead of return`, async () => {
    const asyncIterator = await load("asyncIterator");
    const calls = [];
    const error = new Error("injected");
    const iterable = {
      [Symbol.iterator]() {
        return this;
      },
      next() {
        return { value: 1, done: false };
      },
      throw(value) {
        calls.push(["throw", value]);
        return { value: Promise.resolve("recovered"), done: false };
      },
      return(value) {
        calls.push(["return", value]);
        return { value, done: true };
      },
    };
    const iterator = asyncIterator(iterable);
    assert.deepEqual(await iterator.throw(error), { value: "recovered", done: false });
    assert.deepEqual(await iterator.return("finished"), { value: "finished", done: true });
    assert.deepEqual(calls, [
      ["throw", error],
      ["return", "finished"],
    ]);
  });

  test(`${mode}: returning through finally awaits preserves cleanup yields`, async () => {
    const wrapAsyncGenerator = await load("wrapAsyncGenerator");
    const awaitAsyncGenerator = await load("awaitAsyncGenerator");
    const transformed = wrapAsyncGenerator(function* () {
      try {
        yield "body";
      } finally {
        yield awaitAsyncGenerator(Promise.resolve());
        yield "cleanup";
      }
    });
    async function* native() {
      try {
        yield "body";
      } finally {
        await Promise.resolve();
        yield "cleanup";
      }
    }
    const actual = transformed();
    const expected = native();
    assert.deepEqual(await actual.next(), await expected.next());
    const result = await actual.return("finished");
    assert.deepEqual(result, await expected.return("finished"));
    assert.deepEqual(result, { value: "cleanup", done: false });
    assert.deepEqual(await actual.next(), await expected.next());
  });

  test(`${mode}: async generators serialize queued next, throw and return requests`, async () => {
    const wrapAsyncGenerator = await load("wrapAsyncGenerator");
    const awaitAsyncGenerator = await load("awaitAsyncGenerator");
    const transformed = wrapAsyncGenerator(function* () {
      try {
        yield yield awaitAsyncGenerator(Promise.resolve("first"));
      } catch (error) {
        yield error.message;
      }
      return "done";
    });
    async function* native() {
      try {
        yield await Promise.resolve("first");
      } catch (error) {
        yield error.message;
      }
      return "done";
    }
    const requests = (iterator) => [
      iterator.next(),
      iterator.throw(new Error("injected")),
      iterator.next(),
      iterator.return("finished"),
    ];
    assert.deepEqual(
      await Promise.all(requests(transformed())),
      await Promise.all(requests(native())),
    );
  });

  test(`${mode}: return during async delegation completes the iterator`, async () => {
    const wrapAsyncGenerator = await load("wrapAsyncGenerator");
    const asyncGeneratorDelegate = await load("asyncGeneratorDelegate");
    const asyncIterator = await load("asyncIterator");
    const transformed = wrapAsyncGenerator(function* () {
      return yield* asyncGeneratorDelegate(asyncIterator(["body", "unused"]));
    });
    async function* native() {
      return yield* ["body", "unused"];
    }
    const actual = transformed();
    const expected = native();
    assert.deepEqual(await actual.next(), await expected.next());
    assert.deepEqual(await actual.return("finished"), await expected.return("finished"));
    assert.deepEqual(await actual.next(), await expected.next());
  });

  test(`${mode}: property keys work without Symbol`, async () => {
    const toPropertyKey = await load("toPropertyKey");
    const originalSymbol = globalThis.Symbol;
    let actual;
    try {
      globalThis.Symbol = undefined;
      actual = toPropertyKey({ toString: () => "key" });
    } finally {
      globalThis.Symbol = originalSymbol;
    }
    assert.equal(actual, "key");
  });

  test(`${mode}: function names match native computed methods`, async () => {
    const setFunctionName = await load("setFunctionName");
    for (const key of ["method", Symbol(), Symbol(""), Symbol("method")]) {
      const expected = { [key]() {} }[key].name;
      assert.equal(setFunctionName(function () {}, key).name, expected);
      assert.equal(setFunctionName(function () {}, key, "get").name, `get ${expected}`);
    }
  });

  test(`${mode}: array destructuring respects custom iterators and closes early`, async () => {
    const arrayWithHoles = await load("arrayWithHoles");
    const toArray = await load("toArray");
    const slicedToArray = await load("slicedToArray");
    const events = [];
    const source = ["array value"];
    source[Symbol.iterator] = function* () {
      try {
        yield "iterator value";
        yield "second";
      } finally {
        events.push("closed");
      }
    };
    assert.equal(arrayWithHoles(source), undefined);
    assert.deepEqual(toArray(source), ["iterator value", "second"]);
    assert.deepEqual(slicedToArray(source, 1), ["iterator value"]);
    assert.deepEqual(events, ["closed", "closed"]);
    const legacyIterable = { "@@iterator": source[Symbol.iterator] };
    assert.deepEqual(toArray(legacyIterable), ["iterator value", "second"]);
  });

  test(`${mode}: for-of does not close an iterator when reading its value throws`, async () => {
    const createForOfIteratorHelper = await load("createForOfIteratorHelper");
    const error = new Error("value getter");
    const events = [];
    const iterable = {
      [Symbol.iterator]() {
        return {
          next() {
            return {
              done: false,
              get value() {
                throw error;
              },
            };
          },
          return() {
            events.push("closed");
            return { done: true };
          },
        };
      },
    };
    const native = () => {
      for (const value of iterable) void value;
    };
    const transformed = () => {
      const iterator = createForOfIteratorHelper(iterable);
      try {
        iterator.s();
        for (let step; !(step = iterator.n()).done;) void step.value;
      } catch (error) {
        iterator.e(error);
      } finally {
        iterator.f();
      }
    };
    assert.throws(native, (thrown) => thrown === error);
    assert.deepEqual(events, []);
    assert.throws(transformed, (thrown) => thrown === error);
    assert.deepEqual(events, []);
  });

  test(`${mode}: named regexp replacements match native behavior`, async () => {
    const wrapRegExp = await load("wrapRegExp");
    for (const replacement of ["$<named>", "$<missing>", "$<missing", "$<named>/$<missing>"]) {
      const actual = "a".replace(wrapRegExp(/(a)/, { named: 1 }), replacement);
      assert.equal(actual, "a".replace(/(?<named>a)/, replacement));
    }
  });

  test(`${mode}: namespace interop preserves accessors, default and identity`, async () => {
    const interopRequireWildcard = await load("interopRequireWildcard");
    let value = 1;
    const source = {
      get named() {
        return value;
      },
    };
    const namespace = interopRequireWildcard(source);
    assert.equal(Object.getPrototypeOf(namespace), null);
    assert.equal(namespace.default, source);
    assert.equal(namespace.named, 1);
    value = 2;
    assert.equal(namespace.named, 2);
    assert.equal(interopRequireWildcard(source), namespace);
    assert.equal(interopRequireWildcard(null).default, null);
    const esModule = { __esModule: true, default: "default" };
    assert.equal(interopRequireWildcard(esModule), esModule);
    assert.equal(interopRequireWildcard(esModule, true).default, esModule);
  });

  test(`${mode}: new regenerator helpers execute lowered generators and async functions`, async () => {
    const regenerator = await load("regenerator");
    const regeneratorAsync = await load("regeneratorAsync");
    const regeneratorAsyncGen = await load("regeneratorAsyncGen");
    const OverloadYield = await load("OverloadYield");
    function body(context) {
      if (context.n === 0) {
        context.n = 1;
        return "yielded";
      }
      return context.a(2, "finished");
    }
    const iterator = regenerator().w(body);
    assert.deepEqual(iterator.next(), { value: "yielded", done: false });
    assert.deepEqual(iterator.next(), { value: "finished", done: true });
    assert.deepEqual(await regeneratorAsyncGen(body).next(), { value: "yielded", done: false });
    const value = await regeneratorAsync((context) => {
      if (context.n === 0) {
        context.n = 1;
        return new OverloadYield(Promise.resolve(42), 0);
      }
      return context.a(2, context.v);
    });
    assert.equal(value, 42);
  });

  test(`${mode}: relative TypeScript import extensions are rewritten`, async () => {
    const rewrite = await load("tsRewriteRelativeImportExtensions");
    for (const [input, expected] of [
      ["./module.ts", "./module.js"],
      ["../module.mts", "../module.mjs"],
      ["./module.cts", "./module.cjs"],
      ["./view.tsx", "./view.js"],
      ["./module.d.ts", "./module.d.ts"],
      ["package/module.ts", "package/module.ts"],
    ]) {
      assert.equal(rewrite(input), expected);
    }
    assert.equal(rewrite("./view.tsx", true), "./view.jsx");
  });

  test(`${mode}: TypeScript legacy decorator helpers keep their calling convention`, async () => {
    const decorate = await load("decorate");
    const decorateParam = await load("decorateParam");
    const decorateMetadata = await load("decorateMetadata");
    const calls = [];
    class Target {
      method(value) {
        return value;
      }
    }
    decorate([decorateParam(0, (...args) => calls.push(args))], Target.prototype, "method", null);
    assert.deepEqual(calls, [[Target.prototype, "method", 0]]);
    assert.equal(new Target().method("value"), "value");
    assert.equal(decorateMetadata("design:type", Function), undefined);
  });
}
