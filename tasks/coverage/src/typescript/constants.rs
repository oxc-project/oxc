// We are using TSC's Baseline tests as fixtures to measure TypeScript coverage.
//
// However, TSC's Baseline tests are originally "snapshot" tests for TSC itself and include various perspectives.
// For example:
// - Tests that verify syntax as a parser
// - Tests that aim to check type checking and type inference results
// - Tests that verify behavior across numerous compiler options and ES versions
// - Tests that consist of multiple files and verify project-level behavior
// - Tests that verify transpilation results from TS to JS
// - Tests that verify type definition file generation
// - etc...
//
// That's said, not all test cases are directly useful for OXC.
//
// In this coverage measurement, what we're interested in as the OXC pipeline is:
// - Parser: Parse correct syntax without errors, and detecting invalid syntax
// - Semantic: Detect issues with some checks supported
//
// Also note that OXC only supports the latest ES version. (TSC defaults to ES5!)
//
// For these reasons, we want to exclude test cases that are not of interest,
// such as tests requiring type inference, tests related to compiler options, etc.
//
// Otherwise, the "Negative Passed" numbers would remain low,
// and things we can support would be buried in "Expect Syntax Error" lines.
//
// To achieve this, this file defines two lists:
// - `NOT_SUPPORTED_TEST_PATHS`: Exclusion list by test file names
//   - Referenced when determining whether to parse as test targets
// - `NOT_SUPPORTED_ERROR_CODES`: Exclusion list by error codes like TS1234
//   - Referenced in tests that expect errors among the test targets
//   - Referenced to exclude error codes that TSC expects to emit but OXC doesn't support
//     - Multiple error codes are often emitted from a single file
//   - If error codes remain even after excluding these, OXC also needs to report some kind of error
//
// NOTE:
// - Test file(path)s have a `.ts` extension, but they are not necessarily single TS file test cases
//   - They may contain special comments like `@filename foo.js`
//   - This indicates project-level test cases with multiple file units
//   - We process each unit as an independent file individually
//     - (There might be cases where we want to exclude on an individual unit basis?)
//   - `@filename` may specify extensions not supported as `SourceType`
//     - These are also not test targets
// - Also, tests may include variations based on compiler options like `@target: es5,es6`
//   - (Currently, it's not possible to exclude these variations individually)
// - The same error code may be emitted from different TSC components
//   - Sometimes detectable at parse time, sometimes revealed by type inference results
//   - When OXC has limited support for these, we cannot ignore it by error code alone
//     - We have to ignore by file path, then manually add some parts separately to `misc` cases

// spellchecker:off
pub static NOT_SUPPORTED_TEST_PATHS: phf::Set<&'static str> = phf::phf_set![
    // TSC: "use strict" with non-simple parameter list is allowed in ES5
    // OXC: Always ESNext, so not allowed
    "functionWithUseStrictAndSimpleParameterList.ts",
    "parameterInitializerBeforeDestructuringEmit.ts",
    // TSC: RegExp `u` flag with `@target: es5,es6`, and `u` flag is invalid in ES5
    // OXC: Always ESNext, so `u` flag is always valid, never reports error
    "unicodeExtendedEscapesInRegularExpressions01.ts",
    "unicodeExtendedEscapesInRegularExpressions02.ts",
    "unicodeExtendedEscapesInRegularExpressions03.ts",
    "unicodeExtendedEscapesInRegularExpressions04.ts",
    "unicodeExtendedEscapesInRegularExpressions05.ts",
    "unicodeExtendedEscapesInRegularExpressions06.ts",
    "unicodeExtendedEscapesInRegularExpressions08.ts",
    "unicodeExtendedEscapesInRegularExpressions09.ts",
    "unicodeExtendedEscapesInRegularExpressions10.ts",
    "unicodeExtendedEscapesInRegularExpressions11.ts",
    "unicodeExtendedEscapesInRegularExpressions13.ts",
    "unicodeExtendedEscapesInRegularExpressions15.ts",
    "unicodeExtendedEscapesInRegularExpressions16.ts",
    "unicodeExtendedEscapesInRegularExpressions18.ts",
    // TSC: Reports TS18010, invalid accessibility modifier from JSDoc
    // OXC: Does not reflect JSDoc
    "privateNamesIncompatibleModifiersJs.ts",
    // TSC: Exporting JSDoc type annotations from `.js` file
    // OXC: Does not support JSDoc types
    "importingExportingTypes.ts",
    // TSC: Reports TS5052(complains compilerOptions are invalid), also implies TS2564 but NOT reported
    // OXC: We do not care about compiler options and report TS2564 correctly
    "esDecorators-emitDecoratorMetadata.ts",
    // TSC: `let` as variable name is allowed under `@target:es6`
    // OXC: Always ESNext, so `let` is a reserved
    "downlevelLetConst6.ts",
    "VariableDeclaration6_es6.ts",
    // TSC: TS2339 should be reported for `.js` file with `checkJs: true`. But REPL shows error but here isn't...
    // OXC: Reports TS2339 correctly
    "privateIdentifierExpando.ts",
    // TSC: Does not report errors since `.js` file with `checkJs: false`
    // OXC: Reports errors
    "plainJSRedeclare3.ts",
    // TSC: Parse without error, they support BOM
    // OXC: We do not ignore or exclude BOM, will be invalid character error
    "bom-utf16be.ts",
    // TSC: This is just a binary file, but their test project skips reading
    // OXC: Try to parse, and fail
    "TransportStream.ts",
];
// spellchecker:on

// spellchecker:off
pub static NOT_SUPPORTED_ERROR_CODES: phf::Set<&'static str> = phf::phf_set![
    "1006",  // A file cannot have a reference to itself.
    "1055", // Type 'PromiseAlias' is not a valid async function return type in ES5 because it does not refer to a Promise-compatible constructor value.
    "1058", // The return type of an async function must either be a valid promise or must not contain a callable 'then' member.
    "1062", // Type is referenced directly or indirectly in the fulfillment callback of its own 'then' method.
    "1064", // The return type of an async function or method must be the global Promise<T> type. Did you mean to write 'Promise<string>'?
    "1065", // The return type of an async function or method must be the global Promise<T> type.
    "1084", // Invalid 'reference' directive syntax.
    "1147", // Import declarations in a namespace cannot reference a module.
    "1148", // Cannot use imports, exports, or module augmentations when '--module' is 'none'.
    "1166", // A computed property name in a class property declaration must have a simple literal type or a 'unique symbol' type.
    "1169", // A computed property name in an interface must refer to an expression whose type is a literal type or a 'unique symbol' type.
    "1170", // A computed property name in a type literal must refer to an expression whose type is a literal type or a 'unique symbol' type.
    "1192", // Module '"b"' has no default export.
    "1196", // Catch clause variable type annotation must be 'any' or 'unknown' if specified.
    "1202", // Import assignment cannot be used when targeting ECMAScript modules. Consider using 'import * as ns from "mod"', 'import {a} from "mod"', 'import d from "mod"', or another module format instead.
    "1203", // Export assignment cannot be used when targeting ECMAScript modules. Consider using 'export default' or another module format instead.
    "1205", // Re-exporting a type when 'isolatedModules' is enabled requires using 'export type'.
    "1216", // Identifier expected. '__esModule' is reserved as an exported marker when transforming ECMAScript modules.
    "1218", // Export assignment is not supported when '--module' flag is 'system'.
    "1238", // Unable to resolve signature of class decorator when called as an expression.
    "1240", // Unable to resolve signature of property decorator when called as an expression.
    "1241", // Unable to resolve signature of method decorator when called as an expression.
    "1250", // Function declarations are not allowed inside blocks in strict mode when targeting 'ES5'.
    "1251", // Function declarations are not allowed inside blocks in strict mode when targeting 'ES5'. Class definitions are automatically in strict mode.
    "1252", // Function declarations are not allowed inside blocks in strict mode when targeting 'ES5'. Modules are automatically in strict mode.
    "1259", // Module '"b"' can only be default-imported using the 'esModuleInterop' flag
    "1269", // Cannot use 'export import' on a type or type-only namespace when 'isolatedModules' is enabled.
    "1270", // Decorator function return type 'C' is not assignable to type 'void | TypedPropertyDescriptor<() => void>'.
    "1271", // Decorator function return type is 'OmniDecorator' but is expected to be 'void' or 'any'.
    "1272", // A type referenced in a decorated signature must be imported with 'import type' or a namespace import when 'isolatedModules' and 'emitDecoratorMetadata' are enabled.
    "1280", // Namespaces are not allowed in global script files when 'isolatedModules' is enabled. If this file is not intended to be a global script, set 'moduleDetection' to 'force' or add an empty 'export {}' statement.
    "1281", // Cannot access 'A' from another file without qualification when 'isolatedModules' is enabled. Use 'Enum.A' instead.
    "1282", // An 'export =' declaration must reference a value when 'verbatimModuleSyntax' is enabled, but 'I' only refers to a type.
    "1283", // An 'export =' declaration must reference a real value when 'verbatimModuleSyntax' is enabled, but 'J' resolves to a type-only declaration.
    "1284", // An 'export default' must reference a value when 'verbatimModuleSyntax' is enabled, but 'I' only refers to a type.
    "1285", // An 'export default' must reference a real value when 'verbatimModuleSyntax' is enabled, but 'C' resolves to a type-only declaration.
    "1286", // ECMAScript imports and exports cannot be written in a CommonJS file under 'verbatimModuleSyntax'.
    "1287", // A top-level 'export' modifier cannot be used on value declarations in a CommonJS module when 'verbatimModuleSyntax' is enabled.
    "1288", // An import alias cannot resolve to a type or type-only declaration when 'verbatimModuleSyntax' is enabled.
    "1292", // 'T' resolves to a type and must be marked type-only in this file before re-exporting when 'isolatedModules' is enabled. Consider using 'export type { T as default }'.
    "1293", // ECMAScript module syntax is not allowed in a CommonJS module when 'module' is set to 'preserve'.
    "1294", // This syntax is not allowed when 'erasableSyntaxOnly' is enabled.
    "1295", // ECMAScript imports and exports cannot be written in a CommonJS file under 'verbatimModuleSyntax'. Adjust the 'type' field in the nearest 'package.json' to make this file an ECMAScript module, or adjust your 'verbatimModuleSyntax', 'module', and 'moduleResolution' settings in TypeScript.
    "1313", // The body of an 'if' statement cannot be the empty statement.
    "1320", // Type of 'await' operand must either be a valid promise or must not contain a callable 'then' member.
    "1323", // Dynamic imports are only supported when the '--module' flag is set to 'es2020', 'es2022', 'esnext', 'commonjs', 'amd', 'system', 'umd', 'node16', 'node18', 'node20', or 'nodenext'.
    "1324", // Dynamic imports only support a second argument when the '--module' option is set to 'esnext', 'node16', 'node18', 'node20', 'nodenext', or 'preserve'.
    "1329", // 'dec' accepts too few arguments to be used as a decorator here. Did you mean to call it first and write '@dec()'?
    "1330", // A property of an interface or type literal whose type is a 'unique symbol' type must be 'readonly'.
    "1331", // A property of a class whose type is a 'unique symbol' type must be both 'static' and 'readonly'.
    "1332", // A variable whose type is a 'unique symbol' type must be 'const'.
    "1333", // 'unique symbol' types may not be used on a variable declaration with a binding name.
    "1335", // 'unique symbol' types are not allowed here.
    "1337", // An index signature parameter type cannot be a literal type or generic type. Consider using a mapped object type instead.
    "1340", // Module './test' does not refer to a type, but is used as a type here. Did you mean 'typeof import('./test')'?
    "1343", // The 'import.meta' meta-property is only allowed when the '--module' option is 'es2020', 'es2022', 'esnext', 'system', 'node16', 'node18', 'node20', or 'nodenext'.
    "1345", // An expression of type 'void' cannot be tested for truthiness.
    "1360", // Type '{}' does not satisfy the expected type 'T1'.
    "1361", // 'D' cannot be used as a value because it was imported using 'import type'.
    "1362", // 'Foo' cannot be used as a value because it was exported using 'export type'.
    "1378", // Top-level 'await' expressions are only allowed when the 'module' option is set to 'es2022', 'esnext', 'system', 'node16', 'node18', 'node20', 'nodenext', or 'preserve', and the 'target' option is set to 'es2017' or higher.
    "1380", // An import alias cannot reference a declaration that was imported using 'import type'.
    "1432", // Top-level 'for await' loops are only allowed when the 'module' option is set to 'es2022', 'esnext', 'system', 'node16', 'node18', 'node20', 'nodenext', or 'preserve', and the 'target' option is set to 'es2017' or higher.
    "1448", // 'CC' resolves to a type-only declaration and must be re-exported using a type-only re-export when 'isolatedModules' is enabled.
    "1453", // `resolution-mode` should be either `require` or `import`.
    "1454", // `resolution-mode` can only be set for type-only imports.
    "1470", // The 'import.meta' meta-property is not allowed in files which will build into CommonJS output.
    "1471", // Module './' cannot be imported using this construct. The specifier only resolves to an ES module, which cannot be imported with 'require'. Use an ECMAScript import instead.
    "1479", // The current file is a CommonJS module whose imports will produce 'require' calls; however, the referenced file is an ECMAScript module and cannot be imported with 'require'. Consider writing a dynamic 'import("package")' call instead.
    "1484", // 'I' is a type and must be imported using a type-only import when 'verbatimModuleSyntax' is enabled.
    "1485", // 'AClass' resolves to a type-only declaration and must be imported using a type-only import when 'verbatimModuleSyntax' is enabled.
    "1501", // This regular expression flag is only available when targeting 'es2024' or later.
    "1503", // Named capturing groups are only available when targeting 'ES2018' or later.
    "1528", // Any Unicode property that would possibly match more than a single character is only available when the Unicode Sets (v) flag is set.
    "1529", // Unknown Unicode property name or value.
    "1530", // Unicode property value expressions are only available when the Unicode (u) flag or the Unicode Sets (v) flag is set.
    "1533", // This backreference refers to a group that does not exist. There are only 4 capturing groups in this regular expression.
    "1534", // This backreference refers to a group that does not exist. There are no capturing groups in this regular expression.
    "1537", // Decimal escape sequences and backreferences are not allowed in a character class.
    "1538", // Unicode escape sequences are only available when the Unicode (u) flag or the Unicode Sets (v) flag is set.
    "1539", // A 'bigint' literal cannot be used as a property name.
    "1541", // Type-only import of an ECMAScript module from a CommonJS module must have a 'resolution-mode' attribute.
    "1542", // Type import of an ECMAScript module from a CommonJS module must have a 'resolution-mode' attribute.
    "1543", // Importing a JSON file into an ECMAScript module requires a 'type: "json"' import attribute when 'module' is set to 'Node18'.
    "1544", // Named imports from a JSON file into an ECMAScript module are not allowed when 'module' is set to 'Node16'.
    "2011", // Cannot convert 'string' to 'number'.
    "2209", // The project root is ambiguous, but is required to resolve export map entry '.' in file 'package.json'. Supply the `rootDir` compiler option to disambiguate.
    "2210", // The project root is ambiguous, but is required to resolve import map entry '.' in file 'package.json'. Supply the `rootDir` compiler option to disambiguate.
    "2301", // Initializer of instance member variable 'y' cannot reference identifier 'aaa' declared in the constructor.
    "2302", // Static members cannot reference class type parameters.
    "2303", // Circular definition of import alias 'A'.
    "2304", // Cannot find name 'a'.
    "2305", // Module '"./b"' has no exported member 'default'.
    "2306", // File '/node_modules/@types/react/index.d.ts' is not a module.
    "2307", // Cannot find module './SubModule' or its corresponding type declarations.
    "2308", // Module "./b" has already exported a member named '__foo'. Consider explicitly re-exporting to resolve the ambiguity.
    "2310", // Type 'M2' recursively references itself as a base type.
    "2312", // An interface can only extend an object type or intersection of object types with statically known members.
    "2313", // Type parameter 'K' has a circular constraint.
    "2314", // Generic type 'Array<T>' requires 1 type argument(s).
    "2315", // Type 'D' is not generic.
    "2317", // Global type 'Array' must have 1 type parameter(s).
    "2318", // Cannot find global type 'AsyncDisposable'.
    "2320", // Interface 'Z' cannot simultaneously extend types 'X' and 'Y'.
    "2322", // Type 'number' is not assignable to type 'string'.
    "2328", // Types of parameters 'f' and 'f' are incompatible.
    "2340", // Only public and protected methods of the base class are accessible via the 'super' keyword.
    "2341", // Property 'sfn' is private and only accessible within class 'clodule<T>'.
    "2343", // This syntax requires an imported helper named '__esDecorate' which does not exist in 'tslib'.
    "2344", // Type 'number' does not satisfy the constraint 'string'.
    "2345", // Argument of type '(string | number | boolean)[]' is not assignable to parameter of type '[string, number, boolean]'.
    "2347", // Untyped function calls may not accept type arguments.
    "2348", // Value of type 'new () => string' is not callable. Did you mean to include 'new'?
    "2349", // This expression is not callable.
    "2350", // Only a void function can be called with the 'new' keyword.
    "2351", // This expression is not constructable.
    "2352", // Conversion of type 'string' to type 'number' may be a mistake because neither type sufficiently overlaps with the other. If this was intentional, convert the expression to 'unknown' first.
    "2353", // Object literal may only specify known properties, and 'trueness' does not exist in type 'Action'.
    "2354", // This syntax requires an imported helper but module 'tslib' cannot be found.
    "2355", // A function whose declared type is neither 'undefined', 'void', nor 'any' must return a value.
    "2358", // The left-hand side of an 'instanceof' expression must be of type 'any', an object type or a type parameter.
    "2359", // The right-hand side of an 'instanceof' expression must be either of type 'any', a class, function, or other type assignable to the 'Function' interface type, or an object type with a 'Symbol.hasInstance' method.
    "2362", // The left-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.
    "2363", // The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.
    "2365", // Operator '+' cannot be applied to types 'boolean' and 'boolean'.
    "2366", // Function lacks ending return statement and return type does not include 'undefined'.
    "2367", // This comparison appears to be unintentional because the types '0' and '1' have no overlap.
    "2370", // A rest parameter must be of an array type.
    "2375", // Type '{ value: undefined; }' is not assignable to type 'A' with 'exactOptionalPropertyTypes: true'. Consider adding 'undefined' to the types of the target's properties.
    "2403", // Subsequent variable declarations must have the same type.  Variable 'x' must be of type 'any', but here has type 'any[]'.
    "2407", // The right-hand side of a 'for...in' statement must be of type 'any', an object type or a type parameter, but here has type 'Color.Blue'.
    "2409", // Return type of constructor signature must be assignable to the instance type of the class.
    "2411", // Property 'y' of type 'string' is not assignable to 'string' index type 'number'.
    "2412", // Type 'undefined' is not assignable to type 'string' with 'exactOptionalPropertyTypes: true'. Consider adding 'undefined' to the type of the target.
    "2413", // '`a${string}a`' index type '"c"' is not assignable to '`${string}a`' index type '"b"'.
    "2415", // Class 'L' incorrectly extends base class 'G'.
    "2416", // Property 'num' in type 'WrongTypePropertyImpl' is not assignable to the same property in base type 'WrongTypeProperty'.
    "2417", // Class static side 'typeof Conestoga' incorrectly extends base class static side 'typeof Wagon'.
    "2418", // Type of computed property's value is '"str"', which is not assignable to type '"sym"'.
    "2420", // Class 'C' incorrectly implements interface 'I'.
    "2422", // A class can only implement an object type or intersection of object types with statically known members.
    "2423", // Class 'A' defines instance member function 'm', but extended class 'B' defines it as instance member accessor.
    "2425", // Class 'Good' defines instance member property 'f', but extended class 'Baad' defines it as instance member function.
    "2426", // Class 'Base' defines instance member accessor 'x', but extended class 'Derived' defines it as instance member function.
    "2430", // Interface 'Bar' incorrectly extends interface 'Foo'.
    "2433", // A namespace declaration cannot be in a different file from a class or function with which it is merged.
    "2445", // Property 'p' is protected and only accessible within class 'C3' and its subclasses.
    "2446", // Property 'x' is protected and only accessible through an instance of class 'Derived1'. This is an instance of class 'Base'.
    "2447", // The '&' operator is not allowed for boolean types. Consider using '&&' instead.
    "2448", // Block-scoped variable 'v' used before its declaration.
    "2449", // Class 'A' used before its declaration.
    "2450", // Enum 'E' used before its declaration.
    "2454", // Variable 'getValue' is used before being assigned.
    "2456", // Type alias 'A' circularly references itself.
    "2458", // An AMD module cannot have multiple name assignments.
    "2459", // Module '"./a"' declares 'bar' locally, but it is not exported.
    "2460", // Module '"./a"' declares 'bar' locally, but it is exported as 'baz'.
    "2461", // Type 'number' is not an array type.
    "2467", // A computed property name cannot reference a type parameter from its containing type.
    "2468", // Cannot find global value 'Promise'.
    "2469", // The '+' operator cannot be applied to type 'symbol'.
    "2488", // Type '() => any' must have a '[Symbol.iterator]()' method that returns an iterator.
    "2490", // The type returned by the 'next()' method of an iterator must have a 'value' property.
    "2493", // Tuple type '[string, number]' of length '2' has no element at index '2'.
    "2495", // Type 'MyStringIterator' is not an array type or a string type.
    "2497", // This module can only be referenced with ECMAScript imports/exports by turning on the 'esModuleInterop' flag and referencing its default export.
    "2498", // Module '"interface"' uses 'export =' and cannot be used with 'export *'.
    "2502", // 'foo' is referenced directly or indirectly in its own type annotation.
    "2503", // Cannot find namespace 'no'.
    "2504", // Type '{ [Symbol.asyncIterator](_: number): AsyncGenerator<number, void, unknown>; }' must have a '[Symbol.asyncIterator]()' method that returns an async iterator.
    "2506", // 'C' is referenced directly or indirectly in its own base expression.
    "2507", // Type 'TFunction' is not a constructor function type.
    "2508", // No base constructor has the specified number of type arguments.
    "2509", // Base constructor return type 'never' is not an object type or intersection of object types with statically known members.
    "2510", // Base constructors must all have the same return type.
    "2511", // Cannot create an instance of an abstract class.
    "2512", // Overload signatures must all be abstract or non-abstract.
    "2513", // Abstract method 'foo' in class 'B' cannot be accessed via super expression.
    "2514", // A tuple type cannot be indexed with a negative value.
    "2515", // Non-abstract class 'C' does not implement inherited abstract member next from class 'Iterator<number, undefined, unknown>'.
    "2516", // All declarations of an abstract method must be consecutive.
    "2527", // The inferred type of 'A1' references an inaccessible 'unique symbol' type. A type annotation is necessary.
    "2531", // Object is possibly 'null'.
    "2532", // Object is possibly 'undefined'.
    "2533", // Object is possibly 'null' or 'undefined'.
    "2534", // A function returning 'never' cannot have a reachable end point.
    "2536", // Type '"0.0"' cannot be used to index type 'T'.
    "2537", // Type '{ bar: string; }' has no matching index signature for type 'string'.
    "2538", // Type '[]' cannot be used as an index type.
    "2540", // Cannot assign to 'ro' because it is a read-only property.
    "2542", // Index signature in type 'DeepReadonlyArray<Part>' only permits reading.
    "2545", // A mixin class must have a constructor with a single rest parameter of type 'any[]'.
    "2548", // Type 'number' is not an array type or does not have a '[Symbol.iterator]()' method that returns an iterator.
    "2550", // Property 'setBigInt64' does not exist on type 'DataView<ArrayBuffer>'. Do you need to change your target library? Try changing the 'lib' compiler option to 'es2020' or later.
    "2551", // Property 'fng2' does not exist on type 'typeof A'. Did you mean 'fng'?
    "2552", // Cannot find name 'tupel'. Did you mean 'tuple'?
    "2554", // Expected 0 arguments, but got 1.
    "2555", // Expected at least 1 arguments, but got 0.
    "2556", // A spread argument must either have a tuple type or be passed to a rest parameter.
    "2558", // Expected 0 type arguments, but got 1.
    "2559", // Type 'D' has no properties in common with type 'C'.
    "2560", // Value of type '() => { timeout: number; }' has no properties in common with type 'Settings'. Did you mean to call it?
    "2561", // Object literal may only specify known properties, but 'colour' does not exist in type 'CSSProps'. Did you mean to write 'color'?
    "2563", // The containing function or module body is too large for control flow analysis.
    "2564", // Property 'b' has no initializer and is not definitely assigned in the constructor.
    "2565", // Property 'blah2' is used before being assigned.
    "2571", // Object is of type 'unknown'.
    "2574", // A rest element type must be an array type.
    "2575", // No overload expects 2 arguments, but overloads do exist that expect either 1 or 3 arguments.
    "2576", // Property 'bar' does not exist on type 'C2'. Did you mean to access the static member 'C2.bar' instead?
    "2577", // Return type annotation circularly references itself.
    "2578", // Unused '@ts-expect-error' directive.
    "2580", // Cannot find name 'module'. Do you need to install type definitions for node? Try `npm i --save-dev @types/node`.
    "2581", // Cannot find name '$'. Do you need to install type definitions for jQuery? Try `npm i --save-dev @types/jquery`.
    "2582", // Cannot find name 'describe'. Do you need to install type definitions for a test runner? Try `npm i --save-dev @types/jest` or `npm i --save-dev @types/mocha`.
    "2583", // Cannot find name 'BigInt'. Do you need to change your target library? Try changing the 'lib' compiler option to 'es2020' or later.
    "2584", // Cannot find name 'console'. Do you need to change your target library? Try changing the 'lib' compiler option to include 'dom'.
    "2585", // 'Symbol' only refers to a type, but is being used as a value here. Do you need to change your target library? Try changing the 'lib' compiler option to es2015 or later.
    "2589", // Type instantiation is excessively deep and possibly infinite.
    "2590", // Expression produces a union type that is too complex to represent.
    "2591", // Cannot find name 'module'. Do you need to install type definitions for node? Try `npm i --save-dev @types/node` and then add 'node' to the types field in your tsconfig.
    "2595", // 'Foo' can only be imported by using a default import.
    "2596", // 'Foo' can only be imported by turning on the 'esModuleInterop' flag and using a default import.
    "2597", // 'Foo' can only be imported by using a 'require' call or by using a default import.
    "2598", // 'Foo' can only be imported by using a 'require' call or by turning on the 'esModuleInterop' flag and using a default import.
    "2604", // JSX element type 'BaseComponent' does not have any construct or call signatures.
    "2607", // JSX element class does not support attributes because it does not have a 'pr' property.
    "2608", // The global type 'JSX.ElementAttributesProperty' may not have more than one property.
    "2609", // JSX spread child must be an array type.
    "2610", // 'a' is defined as an accessor in class 'C1', but is overridden here in 'C2' as an instance property.
    "2611", // 'p' is defined as a property in class 'A', but is overridden here in 'B' as an accessor.
    "2613", // Module '"db"' has no default export. Did you mean to use 'import { db } from "db"' instead?
    "2614", // Module '"es6ImportDefaultBindingFollowedWithNamedImport1_0"' has no exported member 'a'. Did you mean to use 'import a from "es6ImportDefaultBindingFollowedWithNamedImport1_0"' instead?
    "2615", // Type of property '"each"' circularly references itself in mapped type '{ [P in keyof ListWidget]: undefined extends ListWidget[P] ? never : P; }'.
    "2616", // 'Foo' can only be imported by using 'import Foo = require("./a")' or a default import.
    "2617", // 'a' can only be imported by using 'import a = require("./es6ImportNamedImportNoNamedExports_0")' or by turning on the 'esModuleInterop' flag and using a default import.
    "2628", // Cannot assign to 'A' because it is an enum.
    "2629", // Cannot assign to 'f' because it is a class.
    "2630", // Cannot assign to 'foo' because it is a function.
    "2631", // Cannot assign to 'M' because it is a namespace.
    "2635", // Type '<N extends string, QR>(queries: { [QK in keyof QR]: any; }) => (state?: { queries: QR; }) => { queries: QR; }' has no signatures for which the type argument list is applicable.
    "2636", // Type 'Controller<sub-T>' is not assignable to type 'Controller<super-T>' as implied by variance annotation.
    "2638", // Type '{}' may represent a primitive value, which is not permitted as the right operand of the 'in' operator.
    "2639", // React components cannot include JSX namespace names
    "2649", // Cannot augment module 'lib' with value exports because it resolves to a non-module entity.
    "2650", // Non-abstract class expression is missing implementations for the following members of 'A': 'm1', 'm2', 'm3', 'm4' and 2 more.
    "2652", // Merged declaration 'Decl' cannot include a default export declaration. Consider adding a separate 'export default Decl' declaration instead.
    "2653", // Non-abstract class expression does not implement inherited abstract member 'foo' from class 'A'.
    "2654", // Non-abstract class 'C' is missing implementations for the following members of 'B': 'prop', 'readonlyProp', 'm', 'mismatch'.
    "2655", // Non-abstract class 'B' is missing implementations for the following members of 'A': 'm1', 'm2', 'm3', 'm4' and 2 more.
    "2659", // 'super' is only allowed in members of object literal expressions when option 'target' is 'ES2015' or higher.
    "2661", // Cannot export 'string'. Only local declarations can be exported from a module.
    "2662", // Cannot find name 'foo'. Did you mean the static member 'C.foo'?
    "2663", // Cannot find name 'foo'. Did you mean the instance member 'this.foo'?
    "2664", // Invalid module name in augmentation, module 'ext' cannot be found.
    "2665", // Invalid module name in augmentation. Module 'foo' resolves to an untyped module at '/node_modules/foo/index.js', which cannot be augmented.
    "2671", // Cannot augment module './file1' because it resolves to a non-module entity.
    "2673", // Constructor of class 'D' is private and only accessible within the class declaration.
    "2674", // Constructor of class 'E' is protected and only accessible within the class declaration.
    "2675", // Cannot extend a class 'BaseC'. Class constructor is marked as private.
    "2677", // A type predicate's type must be assignable to its parameter's type.
    "2678", // Type 'Choice.Unknown' is not comparable to type 'Choice.Yes'.
    "2679", // A function that is called with the 'new' keyword cannot have a 'this' type that is 'void'.
    "2683", // 'this' implicitly has type 'any' because it does not have a type annotation.
    "2684", // The 'this' context of type 'EPromise<number, string>' is not assignable to method's 'this' of type 'EPromise<never, string>'.
    "2686", // 'Puppeteer' refers to a UMD global, but the current file is a module. Consider adding an import instead.
    "2687", // All declarations of 'x' must have identical modifiers.
    "2688", // Cannot find type definition file for 'react'.
    "2689", // Cannot extend an interface 'Base'. Did you mean 'implements'?
    "2690", // 'K' only refers to a type, but is being used as a value here. Did you mean to use 'P in K'?
    "2693", // 'I' only refers to a type, but is being used as a value here.
    "2694", // Namespace 'foo.bar.baz' has no exported member 'bar'.
    "2695", // Left side of comma operator is unused and has no side effects.
    "2696", // The 'Object' type is assignable to very few other types. Did you mean to use the 'any' type instead?
    "2698", // Spread types may only be created from object types.
    "2700", // Rest types may only be created from object types.
    "2702", // 'db' only refers to a type, but is being used as a namespace here.
    "2704", // The operand of a 'delete' operator cannot be a read-only property.
    "2705", // An async function or method in ES5 requires the 'Promise' constructor.  Make sure you have a declaration for the 'Promise' constructor or include 'ES2015' in your '--lib' option.
    "2707", // Generic type 'i09<T, U, V>' requires between 2 and 3 type arguments.
    "2708", // Cannot use namespace 'M' as a value.
    "2709", // Cannot use namespace 'M' as a type.
    "2710", // 'children' are specified twice. The attribute named 'children' will be overwritten.
    "2712", // A dynamic import call in ES5 requires the 'Promise' constructor.  Make sure you have a declaration for the 'Promise' constructor or include 'ES2015' in your '--lib' option.
    "2713", // Cannot access 'Foo.bar' because 'Foo' is a type, but not a namespace. Did you mean to retrieve the type of the property 'bar' in 'Foo' with 'Foo["bar"]'?
    "2715", // Abstract property 'prop' in class 'AbstractClass' cannot be accessed in the constructor.
    "2716", // Type parameter 'T' has a circular default.
    "2717", // Subsequent property declarations must have the same type.  Property 'a' must be of type '() => number', but here has type 'number'.
    "2719", // Type 'T' is not assignable to type 'T'. Two different types with this name exist, but they are unrelated.
    "2720", // Class 'C' incorrectly implements class 'A'. Did you mean to extend 'A' and inherit its members as a subclass?
    "2721", // Cannot invoke an object which is possibly 'null'.
    "2722", // Cannot invoke an object which is possibly 'undefined'.
    "2723", // Cannot invoke an object which is possibly 'null' or 'undefined'.
    "2724", // '"./a"' has no exported member named 'assertNevar'. Did you mean 'assertNever'?
    "2725", // Class name cannot be 'Object' when targeting ES5 and above with module CommonJS.
    "2729", // Property 'prop' is used before its initialization.
    "2731", // Implicit conversion of a 'symbol' to a 'string' will fail at runtime. Consider wrapping this expression in 'String(...)'.
    "2732", // Cannot find module './b.json'. Consider using '--resolveJsonModule' to import module with '.json' extension.
    "2736", // Operator '+' cannot be applied to type 'bigint'.
    "2737", // BigInt literals are not available when targeting lower than ES2020.
    "2739", // Type 'undefined[]' is missing the following properties from type 'C1': IM1, C1M1
    "2740", // Type '{ one: number; }' is missing the following properties from type 'any[]': length, pop, push, concat, and 16 more.
    "2741", // Property '2' is missing in type 'StrNum' but required in type '[number, number, number]'.
    "2742", // The inferred type of 'x' cannot be named without a reference to 'foo/node_modules/nested'. This is likely not portable. A type annotation is necessary.
    "2743", // No overload expects 2 type arguments, but overloads do exist that expect either 1 or 3 type arguments.
    "2744", // Type parameter defaults can only reference previously declared type parameters.
    "2745", // This JSX tag's 'children' prop expects type '((x: number) => string)[]' which requires multiple children, but only a single child was provided.
    "2746", // This JSX tag's 'children' prop expects a single child of type 'Element', but multiple children were provided.
    "2747", // 'Comp' components don't accept text as child elements. Text in JSX has the type 'string', but the expected type of 'children' is 'Element | Element[]'.
    "2748", // Cannot access ambient const enums when 'isolatedModules' is enabled.
    "2749", // 'originalZZZ' refers to a value, but is being used as a type here. Did you mean 'typeof originalZZZ'?
    "2763", // Cannot iterate value because the 'next' method of its iterator expects type 'string', but for-of will always send 'undefined'.
    "2764", // Cannot iterate value because the 'next' method of its iterator expects type 'string', but array spread will always send 'undefined'.
    "2765", // Cannot iterate value because the 'next' method of its iterator expects type 'string', but array destructuring will always send 'undefined'.
    "2766", // Cannot delegate iteration to value because the 'next' method of its iterator expects type 'string', but the containing generator will always send 'boolean'.
    "2767", // The 'return' property of an iterator must be a method.
    "2769", // No overload matches this call.
    "2774", // This condition will always return true since this function is always defined. Did you mean to call it instead?
    "2775", // Assertions require every name in the call target to be declared with an explicit type annotation.
    "2776", // Assertions require the call target to be an identifier or qualified name.
    "2783", // 'a' is specified more than once, so this usage will be overwritten.
    "2786", // 'MySFC' cannot be used as a JSX component.
    "2790", // The operand of a 'delete' operator must be optional.
    "2791", // Exponentiation cannot be performed on 'bigint' values unless the 'target' option is set to 'es2016' or later.
    "2792", // Cannot find module 'foo'. Did you mean to set the 'moduleResolution' option to 'nodenext', or to add aliases to the 'paths' option?
    "2796", // It is likely that you are missing a comma to separate these two template expressions. They form a tagged template expression which cannot be invoked.
    "2797", // A mixin class that extends from a type variable containing an abstract construct signature must also be declared 'abstract'.
    "2799", // Type produces a tuple type that is too large to represent.
    "2800", // Expression produces a tuple type that is too large to represent.
    "2801", // This condition will always return true since this 'Promise<number>' is always defined.
    "2802", // Type 'Set<string>' can only be iterated through when using the '--downlevelIteration' flag or with a '--target' of 'es2015' or higher.
    "2803", // Cannot assign to private method '#m'. Private methods are not writable.
    "2806", // Private accessor was defined without a getter.
    "2807", // This syntax requires an imported helper named '__spreadArray' with 3 parameters, which is not compatible with the one in 'tslib'. Consider upgrading your version of 'tslib'.
    "2808", // A get accessor must be at least as accessible as the setter
    "2812", // Property 'textContent' does not exist on type 'Element'. Try changing the 'lib' compiler option to include 'dom'.
    "2813", // Class declaration cannot implement overload list for 'c2'.
    "2814", // Function with bodies can only merge with classes that are ambient.
    "2818", // Duplicate identifier 'Reflect'. Compiler reserves name 'Reflect' when emitting 'super' references in static initializers.
    "2820", // Type '"strong"' is not assignable to type 'T1'. Did you mean '"string"'?
    "2821", // Import assertions are only supported when the '--module' option is set to 'esnext', 'node18', 'node20', 'nodenext', or 'preserve'.
    "2823", // Import attributes are only supported when the '--module' option is set to 'esnext', 'node18', 'node20', 'nodenext', or 'preserve'.
    "2833", // Cannot find namespace 'b'. Did you mean 'B'?
    "2834", // Relative import paths need explicit file extensions in ECMAScript imports when '--moduleResolution' is 'node16' or 'nodenext'. Consider adding an extension to the import path.
    "2835", // Relative import paths need explicit file extensions in ECMAScript imports when '--moduleResolution' is 'node16' or 'nodenext'. Did you mean './foo.mjs'?
    "2836", // Import assertions are not allowed on statements that compile to CommonJS 'require' calls.
    "2838", // All declarations of 'U' must have identical constraints.
    "2839", // This condition will always return 'false' since JavaScript compares objects by reference, not value.
    "2840", // An interface cannot extend a primitive type like 'string'. It can only extend other named object types.
    "2842", // 'alias' is an unused renaming of 'name'. Did you intend to use it as a type annotation?
    "2844", // Type of instance member variable 'b' cannot reference identifier 'x' declared in the constructor.
    "2845", // This condition will always return 'false'.
    "2846", // A declaration file cannot be imported without 'import type'. Did you mean to import an implementation file './a.js' instead?
    "2850", // The initializer of a 'using' declaration must be either an object with a '[Symbol.dispose]()' method, or be 'null' or 'undefined'.
    "2851", // The initializer of an 'await using' declaration must be either an object with a '[Symbol.asyncDispose]()' or '[Symbol.dispose]()' method, or be 'null' or 'undefined'.
    "2854", // Top-level 'await using' statements are only allowed when the 'module' option is set to 'es2022', 'esnext', 'system', 'node16', 'node18', 'node20', 'nodenext', or 'preserve', and the 'target' option is set to 'es2017' or higher.
    "2855", // Class field 'x' defined by the parent class is not accessible in the child class via super.
    "2856", // Import attributes are not allowed on statements that compile to CommonJS 'require' calls.
    "2859", // Excessive complexity comparing types 'T1 & T2' and 'T1 | null'.
    "2860", // The left-hand side of an 'instanceof' expression must be assignable to the first argument of the right-hand side's '[Symbol.hasInstance]' method.
    "2861", // An object's '[Symbol.hasInstance]' method must return a boolean value for it to be used on the right-hand side of an 'instanceof' expression.
    "2862", // Type 'T' is generic and can only be indexed for reading.
    "2863", // A class cannot extend a primitive type like 'number'. Classes can only extend constructable values.
    "2864", // A class cannot implement a primitive type like 'number'. It can only implement other named object types.
    "2865", // Import 'T' conflicts with local value, so must be declared with a type-only import when 'isolatedModules' is enabled.
    "2867", // Cannot find name 'Bun'. Do you need to install type definitions for Bun? Try `npm i --save-dev @types/bun`.
    "2868", // Cannot find name 'Bun'. Do you need to install type definitions for Bun? Try `npm i --save-dev @types/bun` and then add 'bun' to the types field in your tsconfig.
    "2869", // Right operand of ?? is unreachable because the left operand is never nullish.
    "2871", // This expression is always nullish.
    "2872", // This kind of expression is always truthy.
    "2873", // This kind of expression is always falsy.
    "2874", // This JSX tag requires 'React' to be in scope, but it could not be found.
    "2875", // This JSX tag requires the module path 'react/jsx-runtime' to exist, but none could be found. Make sure you have types for the appropriate package installed.
    "2876", // This relative import path is unsafe to rewrite because it looks like a file name, but actually resolves to "./foo.ts/index.ts".
    "2877", // This import uses a '.ts' extension to resolve to an input TypeScript file, but will not be rewritten during emit because it is not a relative path.
    "2879", // Using JSX fragments requires fragment factory 'React' to be in scope, but it could not be found.
    "2881", // This expression is never nullish.
    "4023", // Exported variable 'foo' has or is using name 'Foo' from external module "type" but cannot be named.
    "4025", // Exported variable 'b' has or is using private name 'a'.
    "4032", // Property 'val' of exported interface has or is using name 'I' from private module '"a"'.
    "4081", // Exported type alias 'MyClass' has or is using private name 'myClass'.
    "4094", // Property '_assertIsStripped' of exported anonymous class type may not be private or protected.
    "4104", // The type 'readonly string[]' is 'readonly' and cannot be assigned to the mutable type 'string[]'.
    "4105", // Private or protected member 'a' cannot be accessed on a type parameter.
    "4109", // Type arguments for 'NumArray' circularly reference themselves.
    "4110", // Tuple type arguments circularly reference themselves.
    "4111", // Property 'foo' comes from an index signature, so it must be accessed with ['foo'].
    "4112", // This member cannot have an 'override' modifier because its containing class 'C' does not extend another class.
    "4113", // This member cannot have an 'override' modifier because it is not declared in the base class 'B'.
    "4114", // This member must have an 'override' modifier because it overrides a member in the base class 'B'.
    "4115", // This parameter property must have an 'override' modifier because it overrides a member in base class 'B'.
    "4116", // This member must have an 'override' modifier because it overrides an abstract method that is declared in the base class 'AB'.
    "4117", // This member cannot have an 'override' modifier because it is not declared in the base class 'A'. Did you mean 'doSomething'?
    "4118", // The type of this node cannot be serialized because its property '[timestampSymbol]' cannot be serialized.
    "4119", // This member must have a JSDoc comment with an '@override' tag because it overrides a member in the base class 'A'.
    "4121", // This member cannot have a JSDoc comment with an '@override' tag because its containing class 'C' does not extend another class.
    "4122", // This member cannot have a JSDoc comment with an '@override' tag because it is not declared in the base class 'A'.
    "4123", // This member cannot have a JSDoc comment with an 'override' tag because it is not declared in the base class 'A'. Did you mean 'doSomething'?
    "5009", // Cannot find the common subdirectory path for the input files.
    "5052", // Option 'checkJs' cannot be specified without specifying option 'allowJs'.
    "5053", // Option 'mapRoot' cannot be specified with option 'inlineSourceMap'.
    "5055", // Cannot write file 'a.d.ts' because it would overwrite input file.
    "5056", // Cannot write file 'a.js' because it would be overwritten by multiple input files.
    "5059", // Invalid value for '--reactNamespace'. 'my-React-Lib' is not a valid identifier.
    "5061", // Pattern 'too*many*asterisks' can have at most one '*' character.
    "5062", // Substitution '*2*' in pattern '*1*' can have at most one '*' character.
    "5063", // Substitutions for pattern '*' should be an array.
    "5064", // Substitution '1' for pattern '*' has incorrect type, expected 'string', got 'number'.
    "5066", // Substitutions for pattern 'foo' shouldn't be an empty array.
    "5067", // Invalid value for 'jsxFactory'. 'Element.createElement=' is not a valid identifier or qualified-name.
    "5069", // Option 'emitDeclarationOnly' cannot be specified without specifying option 'declaration' or option 'composite'.
    "5070", // Option '--resolveJsonModule' cannot be specified when 'moduleResolution' is set to 'classic'.
    "5071", // Option '--resolveJsonModule' cannot be specified when 'module' is set to 'none', 'system', or 'umd'.
    "5074", // Option '--incremental' can only be specified using tsconfig, emitting to single file or when option '--tsBuildInfoFile' is specified.
    "5088", // The inferred type of 'foo' references a type with a cyclic structure which cannot be trivially serialized. A type annotation is necessary.
    "5090", // Non-relative paths are not allowed when 'baseUrl' is not set. Did you forget a leading './'?
    "5091", // Option 'preserveConstEnums' cannot be disabled when 'isolatedModules' is enabled.
    "5095", // Option 'bundler' can only be used when 'module' is set to 'preserve' or to 'es2015' or later.
    "5097", // An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.
    "5098", // Option 'customConditions' can only be used when 'moduleResolution' is set to 'node16', 'nodenext', or 'bundler'.
    "5101", // Option 'noImplicitUseStrict' is deprecated and will stop functioning in TypeScript 5.5. Specify compilerOption '"ignoreDeprecations": "5.0"' to silence this error.
    "5102", // Option 'noImplicitUseStrict' has been removed. Please remove it from your configuration.
    "5103", // Invalid value for '--ignoreDeprecations'.
    "5105", // Option 'verbatimModuleSyntax' cannot be used when 'module' is set to 'UMD', 'AMD', or 'System'.
    "5107", // Option 'target=ES3' is deprecated and will stop functioning in TypeScript 5.5. Specify compilerOption '"ignoreDeprecations": "5.0"' to silence this error.
    "5108", // Option 'target=ES3' has been removed. Please remove it from your configuration.
    "5109", // Option 'moduleResolution' must be set to 'Node16' (or left unspecified) when option 'module' is set to 'Node18'.
    "5110", // Option 'module' must be set to 'Node16' when option 'moduleResolution' is set to 'Node16'.
    "6053", // File 'invalid.ts' not found.
    "6054", // File 'b.js.map' has an unsupported extension. The only supported extensions are '.ts', '.tsx', '.d.ts', '.js', '.jsx', '.cts', '.d.cts', '.cjs', '.mts', '.d.mts', '.mjs'.
    "6082", // Only 'amd' and 'system' modules are supported alongside --outFile.
    "6131", // Cannot compile modules using option 'outFile' unless the '--module' flag is 'amd' or 'system'.
    "6133", // 'f1' is declared but its value is never read.
    "6137", // Cannot import type declaration files. Consider importing 'foo-bar' instead of '@types/foo-bar'.
    "6138", // Property 'used' is declared but its value is never read.
    "6142", // Module '/foo' was resolved to '/foo.jsx', but '--jsx' is not set.
    "6192", // All imports in import declaration are unused.
    "6196", // 'i1' is declared but never used.
    "6198", // All destructured elements are unused.
    "6199", // All variables are unused.
    "6200", // Definitions of the following identifiers conflict with those in another file: A, B, C, D, E, F, G, H, I
    "6205", // All type parameters are unused.
    "6229", // Tag 'MyComp4' expects at least '4' arguments, but the JSX factory 'React.createElement' provides at most '2'.
    "6231", // Could not resolve the path 'invalid' with the extensions: '.ts', '.tsx', '.d.ts', '.js', '.jsx', '.cts', '.d.cts', '.cjs', '.mts', '.d.mts', '.mjs'.
    "6232", // Declaration augments declaration in another file. This cannot be serialized.
    "6234", // This expression is not callable because it is a 'get' accessor. Did you mean to use it without '()'?
    "6263", // Module './dir/native.node' was resolved to 'dir/native.d.node.ts', but '--allowArbitraryExtensions' is not set.
    "6379", // Composite projects may not disable incremental compilation.
    "7005", // Variable 'x' implicitly has an 'any' type.
    "7006", // Parameter 'x' implicitly has an 'any' type.
    "7008", // Member 'v' implicitly has an 'any' type.
    "7009", // 'new' expression, whose target lacks a construct signature, implicitly has an 'any' type.
    "7010", // 'temp', which lacks return-type annotation, implicitly has an 'any' return type.
    "7011", // Function expression, which lacks return-type annotation, implicitly has an 'any' return type.
    "7012", // This overload implicitly returns the type 'any' because it lacks a return type annotation.
    "7013", // Construct signature, which lacks return-type annotation, implicitly has an 'any' return type.
    "7014", // Function type, which lacks return-type annotation, implicitly has an 'any' return type.
    "7015", // Element implicitly has an 'any' type because index expression is not of type 'number'.
    "7016", // Could not find a declaration file for module './b'. '/src/b.js' implicitly has an 'any' type.
    "7017", // Element implicitly has an 'any' type because type 'typeof globalThis' has no index signature.
    "7018", // Object literal's property 's' implicitly has an 'any' type.
    "7019", // Rest parameter 'r' implicitly has an 'any[]' type.
    "7020", // Call signature, which lacks return-type annotation, implicitly has an 'any' return type.
    "7022", // 'value1' implicitly has type 'any' because it does not have a type annotation and is referenced directly or indirectly in its own initializer.
    "7023", // 'next' implicitly has return type 'any' because it does not have a return type annotation and is referenced directly or indirectly in one of its return expressions.
    "7024", // Function implicitly has return type 'any' because it does not have a return type annotation and is referenced directly or indirectly in one of its return expressions.
    "7025", // Generator implicitly has yield type 'any'. Consider supplying a return type annotation.
    "7026", // JSX element implicitly has type 'any' because no interface 'JSX.IntrinsicElements' exists.
    "7027", // Unreachable code detected.
    "7028", // Unused label.
    "7029", // Fallthrough case in switch.
    "7030", // Not all code paths return a value.
    "7031", // Binding element 'a5' implicitly has an 'any' type.
    "7032", // Property 'message' implicitly has type 'any', because its set accessor lacks a parameter type annotation.
    "7033", // Property 'message' implicitly has type 'any', because its get accessor lacks a return type annotation.
    "7034", // Variable 'x' implicitly has type 'any[]' in some locations where its type cannot be determined.
    "7036", // Dynamic import's specifier must be of type 'string', but here has type 'null'.
    "7039", // Mapped object type implicitly has an 'any' template type.
    "7041", // The containing arrow function captures the global value of 'this'.
    "7052", // Element implicitly has an 'any' type because type '{ get: (key: string) => string; }' has no index signature. Did you mean to call 'c.get'?
    "7053", // Element implicitly has an 'any' type because expression of type 'string' can't be used to index type '{}'.
    "7055", // 'h', which lacks return-type annotation, implicitly has an 'any' yield type.
    "7056", // The inferred type of this node exceeds the maximum length the compiler will serialize. An explicit type annotation is needed.
    "7057", // 'yield' expression implicitly results in an 'any' type because its containing generator lacks a return-type annotation.
    "8021", // JSDoc '@typedef' tag should either have a type annotation or be followed by '@property' or '@member' tags.
    "8022", // JSDoc '@extends' is not attached to a class.
    "8023", // JSDoc '@extends Mismatch' does not match the 'extends B' clause.
    "8024", // JSDoc '@param' tag has name 's', but there is no parameter with that name.
    "8026", // Expected A<T> type arguments; provide these with an '@extends' tag.
    "8029", // JSDoc '@param' tag has name 'rest', but there is no parameter with that name. It would match 'arguments' if it had an array type.
    "8030", // The type of a function declaration must match the function's signature.
    "8032", // Qualified name 'xyz.p' is not allowed without a leading '@param {object} xyz'.
    "8033", // A JSDoc '@typedef' comment may not contain multiple '@type' tags.
    "8039", // A JSDoc '@template' tag may not follow a '@typedef', '@callback', or '@overload' tag
    "9005", // Declaration emit for this file requires using private name 'Sub'. An explicit type annotation may unblock declaration emit.
    "9006", // DeclaDeclaration emit for this file requires using private name 'Item' from module '"some-mod"'. An explicit type annotation may unblock declaration emit.
    "9007", // FunctDeclaion must have an explicit return type annotation with --isolatedDeclarations.
    "9008", // MethoDeclad must have an explicit return type annotation with --isolatedDeclarations.
    "9009", // At leDeclaast one accessor must have an explicit type annotation with --isolatedDeclarations.
    "9010", // VariaDeclable must have an explicit type annotation with --isolatedDeclarations.
    "9011", // ParamDeclaeter must have an explicit type annotation with --isolatedDeclarations.
    "9012", // PropeDeclarty must have an explicit type annotation with --isolatedDeclarations.
    "9013", // ExpreDeclassion type can't be inferred with --isolatedDeclarations.
    "9015", // ObjecDeclats that contain spread assignments can't be inferred with --isolatedDeclarations.
    "9016", // ObjecDeclats that contain shorthand properties can't be inferred with --isolatedDeclarations.
    "9017", // Only Declaconst arrays can be inferred with --isolatedDeclarations.
    "9018", // ArrayDeclas with spread elements can't inferred with --isolatedDeclarations.
    "9019", // BindiDeclang elements can't be exported directly with --isolatedDeclarations.
    "9020", // Enum Declamember initializers must be computable without references to external symbols with --isolatedDeclarations.
    "9021", // ExtenDeclads clause can't contain an expression with --isolatedDeclarations.
    "9022", // InferDeclaence from class expressions is not supported with --isolatedDeclarations.
    "9023", // AssigDeclaning properties to functions without declaring them is not supported with --isolatedDeclarations. Add an explicit declaration for the properties assigned to this function.
    "9026", // DeclaDeclaration emit for this file requires preserving this import for augmentations. This is not supported with --isolatedDeclarations.
    "9037", // DefauDeclalt exports can't be inferred with --isolatedDeclarations.
    "9038", // CompuDeclated property names on class or object literals cannot be inferred with --isolatedDeclarations.
    "17004", // Cannot use JSX unless the '--jsx' flag is provided.
    "17016", // The 'jsxFragmentFactory' compiler option must be provided to use JSX fragments with the 'jsxFactory' compiler option.
    "17017", // An @jsxFrag pragma is required when using an @jsx pragma with JSX fragments.
    "18028", // Private identifiers are only available when targeting ECMAScript 2015 and higher.
    "18033", // Type 'Number' is not assignable to type 'number' as required for computed enum member values.
    "18035", // Invalid value for 'jsxFragmentFactory'. '234' is not a valid identifier or qualified-name.
    "18042", // 'Prop' is a type and cannot be imported in JavaScript files. Use 'import("./component").Prop' in a JSDoc type annotation.
    "18043", // Types cannot appear in export declarations in JavaScript files.
    "18045", // Properties with the 'accessor' modifier are only available when targeting ECMAScript 2015 and higher.
    "18046", // 'x' is of type 'unknown'.
    "18047", // 'x' is possibly 'null'.
    "18048", // 'x' is possibly 'undefined'.
    "18049", // 'x' is possibly 'null' or 'undefined'.
    "18055", // 'A.a' has a string type, but must have syntactically recognizable string syntax when 'isolatedModules' is enabled.
    "18057", // String literal import and export names are not supported when the '--module' flag is set to 'es2015' or 'es2020'.
];
// spellchecker:on
