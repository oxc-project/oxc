//#endregion
//#region src-js/plugins/globals.d.ts
/**
 * Globals for the file being linted.
 *
 * Globals are deserialized from JSON, so can only contain JSON-compatible values.
 * Each global variable maps to "readonly", "writable", or "off".
 */
type Globals = Record<string, "readonly" | "writable" | "off">;
/**
 * Environments for the file being linted.
 *
 * Only includes environments that are enabled, so all properties are `true`.
 */
type Envs = Record<string, true>;
//#endregion
//#region ../../node_modules/.pnpm/@types+json-schema@7.0.15/node_modules/@types/json-schema/index.d.ts
// ==================================================================================================
// JSON Schema Draft 04
// ==================================================================================================
/**
 * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.1
 */
type JSONSchema4TypeName = "string" //
| "number" | "integer" | "boolean" | "object" | "array" | "null" | "any";
/**
 * @see https://tools.ietf.org/html/draft-zyp-json-schema-04#section-3.5
 */
type JSONSchema4Type = string //
| number | boolean | JSONSchema4Object | JSONSchema4Array | null;
// Workaround for infinite type recursion
interface JSONSchema4Object {
  [key: string]: JSONSchema4Type;
}
// Workaround for infinite type recursion
// https://github.com/Microsoft/TypeScript/issues/3496#issuecomment-128553540
interface JSONSchema4Array extends Array<JSONSchema4Type> {}
/**
 * Meta schema
 *
 * Recommended values:
 * - 'http://json-schema.org/schema#'
 * - 'http://json-schema.org/hyper-schema#'
 * - 'http://json-schema.org/draft-04/schema#'
 * - 'http://json-schema.org/draft-04/hyper-schema#'
 * - 'http://json-schema.org/draft-03/schema#'
 * - 'http://json-schema.org/draft-03/hyper-schema#'
 *
 * @see https://tools.ietf.org/html/draft-handrews-json-schema-validation-01#section-5
 */
type JSONSchema4Version = string;
/**
 * JSON Schema V4
 * @see https://tools.ietf.org/html/draft-zyp-json-schema-04
 */
interface JSONSchema4 {
  id?: string | undefined;
  $ref?: string | undefined;
  $schema?: JSONSchema4Version | undefined;
  /**
   * This attribute is a string that provides a short description of the
   * instance property.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.21
   */
  title?: string | undefined;
  /**
   * This attribute is a string that provides a full description of the of
   * purpose the instance property.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.22
   */
  description?: string | undefined;
  default?: JSONSchema4Type | undefined;
  multipleOf?: number | undefined;
  maximum?: number | undefined;
  exclusiveMaximum?: boolean | undefined;
  minimum?: number | undefined;
  exclusiveMinimum?: boolean | undefined;
  maxLength?: number | undefined;
  minLength?: number | undefined;
  pattern?: string | undefined;
  /**
   * May only be defined when "items" is defined, and is a tuple of JSONSchemas.
   *
   * This provides a definition for additional items in an array instance
   * when tuple definitions of the items is provided.  This can be false
   * to indicate additional items in the array are not allowed, or it can
   * be a schema that defines the schema of the additional items.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.6
   */
  additionalItems?: boolean | JSONSchema4 | undefined;
  /**
   * This attribute defines the allowed items in an instance array, and
   * MUST be a schema or an array of schemas.  The default value is an
   * empty schema which allows any value for items in the instance array.
   *
   * When this attribute value is a schema and the instance value is an
   * array, then all the items in the array MUST be valid according to the
   * schema.
   *
   * When this attribute value is an array of schemas and the instance
   * value is an array, each position in the instance array MUST conform
   * to the schema in the corresponding position for this array.  This
   * called tuple typing.  When tuple typing is used, additional items are
   * allowed, disallowed, or constrained by the "additionalItems"
   * (Section 5.6) attribute using the same rules as
   * "additionalProperties" (Section 5.4) for objects.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.5
   */
  items?: JSONSchema4 | JSONSchema4[] | undefined;
  maxItems?: number | undefined;
  minItems?: number | undefined;
  uniqueItems?: boolean | undefined;
  maxProperties?: number | undefined;
  minProperties?: number | undefined;
  /**
   * This attribute indicates if the instance must have a value, and not
   * be undefined. This is false by default, making the instance
   * optional.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.7
   */
  required?: boolean | string[] | undefined;
  /**
   * This attribute defines a schema for all properties that are not
   * explicitly defined in an object type definition. If specified, the
   * value MUST be a schema or a boolean. If false is provided, no
   * additional properties are allowed beyond the properties defined in
   * the schema. The default value is an empty schema which allows any
   * value for additional properties.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.4
   */
  additionalProperties?: boolean | JSONSchema4 | undefined;
  definitions?: {
    [k: string]: JSONSchema4;
  } | undefined;
  /**
   * This attribute is an object with property definitions that define the
   * valid values of instance object property values. When the instance
   * value is an object, the property values of the instance object MUST
   * conform to the property definitions in this object. In this object,
   * each property definition's value MUST be a schema, and the property's
   * name MUST be the name of the instance property that it defines.  The
   * instance property value MUST be valid according to the schema from
   * the property definition. Properties are considered unordered, the
   * order of the instance properties MAY be in any order.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.2
   */
  properties?: {
    [k: string]: JSONSchema4;
  } | undefined;
  /**
   * This attribute is an object that defines the schema for a set of
   * property names of an object instance. The name of each property of
   * this attribute's object is a regular expression pattern in the ECMA
   * 262/Perl 5 format, while the value is a schema. If the pattern
   * matches the name of a property on the instance object, the value of
   * the instance's property MUST be valid against the pattern name's
   * schema value.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.3
   */
  patternProperties?: {
    [k: string]: JSONSchema4;
  } | undefined;
  dependencies?: {
    [k: string]: JSONSchema4 | string[];
  } | undefined;
  /**
   * This provides an enumeration of all possible values that are valid
   * for the instance property. This MUST be an array, and each item in
   * the array represents a possible value for the instance value. If
   * this attribute is defined, the instance value MUST be one of the
   * values in the array in order for the schema to be valid.
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.19
   */
  enum?: JSONSchema4Type[] | undefined;
  /**
   * A single type, or a union of simple types
   */
  type?: JSONSchema4TypeName | JSONSchema4TypeName[] | undefined;
  allOf?: JSONSchema4[] | undefined;
  anyOf?: JSONSchema4[] | undefined;
  oneOf?: JSONSchema4[] | undefined;
  not?: JSONSchema4 | undefined;
  /**
   * The value of this property MUST be another schema which will provide
   * a base schema which the current schema will inherit from.  The
   * inheritance rules are such that any instance that is valid according
   * to the current schema MUST be valid according to the referenced
   * schema.  This MAY also be an array, in which case, the instance MUST
   * be valid for all the schemas in the array.  A schema that extends
   * another schema MAY define additional attributes, constrain existing
   * attributes, or add other constraints.
   *
   * Conceptually, the behavior of extends can be seen as validating an
   * instance against all constraints in the extending schema as well as
   * the extended schema(s).
   *
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-03#section-5.26
   */
  extends?: string | string[] | undefined;
  /**
   * @see https://tools.ietf.org/html/draft-zyp-json-schema-04#section-5.6
   */
  [k: string]: any;
  format?: string | undefined;
}
//#endregion
//#region src-js/plugins/json.d.ts
/**
 * A JSON value.
 */
type JsonValue = JsonObject | JsonValue[] | string | number | boolean | null;
/**
 * A JSON object.
 */
type JsonObject = {
  [key: string]: JsonValue;
};
//#endregion
//#region src-js/plugins/options.d.ts
/**
 * Options for a rule on a file.
 */
type Options = JsonValue[];
/**
 * Schema describing valid options for a rule.
 * `schema` property of `RuleMeta`.
 *
 * Can be one of:
 * - `JSONSchema4` - Full JSON Schema object (must have `type: "array"` at root).
 * - `JSONSchema4[]` - Array shorthand where each element describes corresponding options element.
 * - `false` - Opts out of schema validation (not recommended).
 */
type RuleOptionsSchema = JSONSchema4 | JSONSchema4[] | false;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/is-any.d.ts
/**
Returns a boolean for whether the given type is `any`.

@link https://stackoverflow.com/a/49928360/1490091

Useful in type utilities, such as disallowing `any`s to be passed to a function.

@example
```
import type {IsAny} from 'type-fest';

const typedObject = {a: 1, b: 2} as const;
const anyObject: any = {a: 1, b: 2};

function get<O extends (IsAny<O> extends true ? {} : Record<string, number>), K extends keyof O = keyof O>(object: O, key: K) {
	return object[key];
}

const typedA = get(typedObject, 'a');
//=> 1

const anyA = get(anyObject, 'a');
//=> any
```

@category Type Guard
@category Utilities
*/
type IsAny<T> = 0 extends 1 & NoInfer<T> ? true : false;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/is-optional-key-of.d.ts
/**
Returns a boolean for whether the given key is an optional key of type.

This is useful when writing utility types or schema validators that need to differentiate `optional` keys.

@example
```
import type {IsOptionalKeyOf} from 'type-fest';

type User = {
	name: string;
	surname: string;

	luckyNumber?: number;
};

type Admin = {
	name: string;
	surname?: string;
};

type T1 = IsOptionalKeyOf<User, 'luckyNumber'>;
//=> true

type T2 = IsOptionalKeyOf<User, 'name'>;
//=> false

type T3 = IsOptionalKeyOf<User, 'name' | 'luckyNumber'>;
//=> boolean

type T4 = IsOptionalKeyOf<User | Admin, 'name'>;
//=> false

type T5 = IsOptionalKeyOf<User | Admin, 'surname'>;
//=> boolean
```

@category Type Guard
@category Utilities
*/
type IsOptionalKeyOf<Type extends object, Key extends keyof Type> = IsAny<Type | Key> extends true ? never : Key extends keyof Type ? Type extends Record<Key, Type[Key]> ? false : true : false;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/optional-keys-of.d.ts
/**
Extract all optional keys from the given type.

This is useful when you want to create a new type that contains different type values for the optional keys only.

@example
```
import type {OptionalKeysOf, Except} from 'type-fest';

type User = {
	name: string;
	surname: string;

	luckyNumber?: number;
};

const REMOVE_FIELD = Symbol('remove field symbol');
type UpdateOperation<Entity extends object> = Except<Partial<Entity>, OptionalKeysOf<Entity>> & {
	[Key in OptionalKeysOf<Entity>]?: Entity[Key] | typeof REMOVE_FIELD;
};

const update1: UpdateOperation<User> = {
	name: 'Alice',
};

const update2: UpdateOperation<User> = {
	name: 'Bob',
	luckyNumber: REMOVE_FIELD,
};
```

@category Utilities
*/
type OptionalKeysOf<Type extends object> = Type extends unknown // For distributing `Type`
? (keyof { [Key in keyof Type as IsOptionalKeyOf<Type, Key> extends false ? never : Key]: never }) & keyof Type // Intersect with `keyof Type` to ensure result of `OptionalKeysOf<Type>` is always assignable to `keyof Type`
: never;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/required-keys-of.d.ts
/**
Extract all required keys from the given type.

This is useful when you want to create a new type that contains different type values for the required keys only or use the list of keys for validation purposes, etc...

@example
```
import type {RequiredKeysOf} from 'type-fest';

declare function createValidation<
	Entity extends object,
	Key extends RequiredKeysOf<Entity> = RequiredKeysOf<Entity>,
>(field: Key, validator: (value: Entity[Key]) => boolean): (entity: Entity) => boolean;

type User = {
	name: string;
	surname: string;
	luckyNumber?: number;
};

const validator1 = createValidation<User>('name', value => value.length < 25);
const validator2 = createValidation<User>('surname', value => value.length < 25);

// @ts-expect-error
const validator3 = createValidation<User>('luckyNumber', value => value > 0);
// Error: Argument of type '"luckyNumber"' is not assignable to parameter of type '"name" | "surname"'.
```

@category Utilities
*/
type RequiredKeysOf<Type extends object> = Type extends unknown // For distributing `Type`
? Exclude<keyof Type, OptionalKeysOf<Type>> : never;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/is-never.d.ts
/**
Returns a boolean for whether the given type is `never`.

@link https://github.com/microsoft/TypeScript/issues/31751#issuecomment-498526919
@link https://stackoverflow.com/a/53984913/10292952
@link https://www.zhenghao.io/posts/ts-never

Useful in type utilities, such as checking if something does not occur.

@example
```
import type {IsNever, And} from 'type-fest';

type A = IsNever<never>;
//=> true

type B = IsNever<any>;
//=> false

type C = IsNever<unknown>;
//=> false

type D = IsNever<never[]>;
//=> false

type E = IsNever<object>;
//=> false

type F = IsNever<string>;
//=> false
```

@example
```
import type {IsNever} from 'type-fest';

type IsTrue<T> = T extends true ? true : false;

// When a distributive conditional is instantiated with `never`, the entire conditional results in `never`.
type A = IsTrue<never>;
//=> never

// If you don't want that behaviour, you can explicitly add an `IsNever` check before the distributive conditional.
type IsTrueFixed<T> =
	IsNever<T> extends true ? false : T extends true ? true : false;

type B = IsTrueFixed<never>;
//=> false
```

@category Type Guard
@category Utilities
*/
type IsNever<T> = [T] extends [never] ? true : false;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/if.d.ts
/**
An if-else-like type that resolves depending on whether the given `boolean` type is `true` or `false`.

Use-cases:
- You can use this in combination with `Is*` types to create an if-else-like experience. For example, `If<IsAny<any>, 'is any', 'not any'>`.

Note:
- Returns a union of if branch and else branch if the given type is `boolean` or `any`. For example, `If<boolean, 'Y', 'N'>` will return `'Y' | 'N'`.
- Returns the else branch if the given type is `never`. For example, `If<never, 'Y', 'N'>` will return `'N'`.

@example
```
import type {If} from 'type-fest';

type A = If<true, 'yes', 'no'>;
//=> 'yes'

type B = If<false, 'yes', 'no'>;
//=> 'no'

type C = If<boolean, 'yes', 'no'>;
//=> 'yes' | 'no'

type D = If<any, 'yes', 'no'>;
//=> 'yes' | 'no'

type E = If<never, 'yes', 'no'>;
//=> 'no'
```

@example
```
import type {If, IsAny, IsNever} from 'type-fest';

type A = If<IsAny<unknown>, 'is any', 'not any'>;
//=> 'not any'

type B = If<IsNever<never>, 'is never', 'not never'>;
//=> 'is never'
```

@example
```
import type {If, IsEqual} from 'type-fest';

type IfEqual<T, U, IfBranch, ElseBranch> = If<IsEqual<T, U>, IfBranch, ElseBranch>;

type A = IfEqual<string, string, 'equal', 'not equal'>;
//=> 'equal'

type B = IfEqual<string, number, 'equal', 'not equal'>;
//=> 'not equal'
```

Note: Sometimes using the `If` type can make an implementation non–tail-recursive, which can impact performance. In such cases, it’s better to use a conditional directly. Refer to the following example:

@example
```
import type {If, IsEqual, StringRepeat} from 'type-fest';

type HundredZeroes = StringRepeat<'0', 100>;

// The following implementation is not tail recursive
type Includes<S extends string, Char extends string> =
	S extends `${infer First}${infer Rest}`
		? If<IsEqual<First, Char>,
			'found',
			Includes<Rest, Char>>
		: 'not found';

// Hence, instantiations with long strings will fail
// @ts-expect-error
type Fails = Includes<HundredZeroes, '1'>;
//           ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Error: Type instantiation is excessively deep and possibly infinite.

// However, if we use a simple conditional instead of `If`, the implementation becomes tail-recursive
type IncludesWithoutIf<S extends string, Char extends string> =
	S extends `${infer First}${infer Rest}`
		? IsEqual<First, Char> extends true
			? 'found'
			: IncludesWithoutIf<Rest, Char>
		: 'not found';

// Now, instantiations with long strings will work
type Works = IncludesWithoutIf<HundredZeroes, '1'>;
//=> 'not found'
```

@category Type Guard
@category Utilities
*/
type If<Type extends boolean, IfBranch, ElseBranch> = IsNever<Type> extends true ? ElseBranch : Type extends true ? IfBranch : ElseBranch;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/internal/type.d.ts
/**
An if-else-like type that resolves depending on whether the given type is `any` or `never`.

@example
```
// When `T` is a NOT `any` or `never` (like `string`) => Returns `IfNotAnyOrNever` branch
type A = IfNotAnyOrNever<string, 'VALID', 'IS_ANY', 'IS_NEVER'>;
//=> 'VALID'

// When `T` is `any` => Returns `IfAny` branch
type B = IfNotAnyOrNever<any, 'VALID', 'IS_ANY', 'IS_NEVER'>;
//=> 'IS_ANY'

// When `T` is `never` => Returns `IfNever` branch
type C = IfNotAnyOrNever<never, 'VALID', 'IS_ANY', 'IS_NEVER'>;
//=> 'IS_NEVER'
```

Note: Wrapping a tail-recursive type with `IfNotAnyOrNever` makes the implementation non-tail-recursive. To fix this, move the recursion into a helper type. Refer to the following example:

@example
```ts
import type {StringRepeat} from 'type-fest';

type NineHundredNinetyNineSpaces = StringRepeat<' ', 999>;

// The following implementation is not tail recursive
type TrimLeft<S extends string> = IfNotAnyOrNever<S, S extends ` ${infer R}` ? TrimLeft<R> : S>;

// Hence, instantiations with long strings will fail
// @ts-expect-error
type T1 = TrimLeft<NineHundredNinetyNineSpaces>;
//        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Error: Type instantiation is excessively deep and possibly infinite.

// To fix this, move the recursion into a helper type
type TrimLeftOptimised<S extends string> = IfNotAnyOrNever<S, _TrimLeftOptimised<S>>;

type _TrimLeftOptimised<S extends string> = S extends ` ${infer R}` ? _TrimLeftOptimised<R> : S;

type T2 = TrimLeftOptimised<NineHundredNinetyNineSpaces>;
//=> ''
```
*/
type IfNotAnyOrNever<T, IfNotAnyOrNever, IfAny = any, IfNever = never> = If<IsAny<T>, IfAny, If<IsNever<T>, IfNever, IfNotAnyOrNever>>;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/simplify.d.ts
/**
Useful to flatten the type output to improve type hints shown in editors. And also to transform an interface into a type to aide with assignability.

@example
```
import type {Simplify} from 'type-fest';

type PositionProps = {
	top: number;
	left: number;
};

type SizeProps = {
	width: number;
	height: number;
};

// In your editor, hovering over `Props` will show a flattened object with all the properties.
type Props = Simplify<PositionProps & SizeProps>;
```

Sometimes it is desired to pass a value as a function argument that has a different type. At first inspection it may seem assignable, and then you discover it is not because the `value`'s type definition was defined as an interface. In the following example, `fn` requires an argument of type `Record<string, unknown>`. If the value is defined as a literal, then it is assignable. And if the `value` is defined as type using the `Simplify` utility the value is assignable.  But if the `value` is defined as an interface, it is not assignable because the interface is not sealed and elsewhere a non-string property could be added to the interface.

If the type definition must be an interface (perhaps it was defined in a third-party npm package), then the `value` can be defined as `const value: Simplify<SomeInterface> = ...`. Then `value` will be assignable to the `fn` argument.  Or the `value` can be cast as `Simplify<SomeInterface>` if you can't re-declare the `value`.

@example
```
import type {Simplify} from 'type-fest';

interface SomeInterface {
	foo: number;
	bar?: string;
	baz: number | undefined;
}

type SomeType = {
	foo: number;
	bar?: string;
	baz: number | undefined;
};

const literal = {foo: 123, bar: 'hello', baz: 456};
const someType: SomeType = literal;
const someInterface: SomeInterface = literal;

declare function fn(object: Record<string, unknown>): void;

fn(literal); // Good: literal object type is sealed
fn(someType); // Good: type is sealed
// @ts-expect-error
fn(someInterface); // Error: Index signature for type 'string' is missing in type 'someInterface'. Because `interface` can be re-opened
fn(someInterface as Simplify<SomeInterface>); // Good: transform an `interface` into a `type`
```

@link https://github.com/microsoft/TypeScript/issues/15300
@see {@link SimplifyDeep}
@category Object
*/
type Simplify<T> = { [KeyType in keyof T]: T[KeyType] } & {};
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/is-equal.d.ts
/**
Returns a boolean for whether the two given types are equal.

@link https://github.com/microsoft/TypeScript/issues/27024#issuecomment-421529650
@link https://stackoverflow.com/questions/68961864/how-does-the-equals-work-in-typescript/68963796#68963796

Use-cases:
- If you want to make a conditional branch based on the result of a comparison of two types.

@example
```
import type {IsEqual} from 'type-fest';

// This type returns a boolean for whether the given array includes the given item.
// `IsEqual` is used to compare the given array at position 0 and the given item and then return true if they are equal.
type Includes<Value extends readonly any[], Item> =
	Value extends readonly [Value[0], ...infer rest]
		? IsEqual<Value[0], Item> extends true
			? true
			: Includes<rest, Item>
		: false;
```

@category Type Guard
@category Utilities
*/
type IsEqual<A, B> = [A] extends [B] ? [B] extends [A] ? _IsEqual<A, B> : false : false;
// This version fails the `equalWrappedTupleIntersectionToBeNeverAndNeverExpanded` test in `test-d/is-equal.ts`.
type _IsEqual<A, B> = (<G>() => G extends A & G | G ? 1 : 2) extends (<G>() => G extends B & G | G ? 1 : 2) ? true : false;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/omit-index-signature.d.ts
/**
Omit any index signatures from the given object type, leaving only explicitly defined properties.

This is the counterpart of `PickIndexSignature`.

Use-cases:
- Remove overly permissive signatures from third-party types.

This type was taken from this [StackOverflow answer](https://stackoverflow.com/a/68261113/420747).

It relies on the fact that an empty object (`{}`) is assignable to an object with just an index signature, like `Record<string, unknown>`, but not to an object with explicitly defined keys, like `Record<'foo' | 'bar', unknown>`.

(The actual value type, `unknown`, is irrelevant and could be any type. Only the key type matters.)

```
const indexed: Record<string, unknown> = {}; // Allowed

// @ts-expect-error
const keyed: Record<'foo', unknown> = {}; // Error
// TS2739: Type '{}' is missing the following properties from type 'Record<"foo" | "bar", unknown>': foo, bar
```

Instead of causing a type error like the above, you can also use a [conditional type](https://www.typescriptlang.org/docs/handbook/2/conditional-types.html) to test whether a type is assignable to another:

```
type Indexed = {} extends Record<string, unknown>
	? '✅ `{}` is assignable to `Record<string, unknown>`'
	: '❌ `{}` is NOT assignable to `Record<string, unknown>`';

type IndexedResult = Indexed;
//=> '✅ `{}` is assignable to `Record<string, unknown>`'

type Keyed = {} extends Record<'foo' | 'bar', unknown>
	? '✅ `{}` is assignable to `Record<\'foo\' | \'bar\', unknown>`'
	: '❌ `{}` is NOT assignable to `Record<\'foo\' | \'bar\', unknown>`';

type KeyedResult = Keyed;
//=> '❌ `{}` is NOT assignable to `Record<\'foo\' | \'bar\', unknown>`'
```

Using a [mapped type](https://www.typescriptlang.org/docs/handbook/2/mapped-types.html#further-exploration), you can then check for each `KeyType` of `ObjectType`...

```
type OmitIndexSignature<ObjectType> = {
	[KeyType in keyof ObjectType // Map each key of `ObjectType`...
	]: ObjectType[KeyType]; // ...to its original value, i.e. `OmitIndexSignature<Foo> == Foo`.
};
```

...whether an empty object (`{}`) would be assignable to an object with that `KeyType` (`Record<KeyType, unknown>`)...

```
type OmitIndexSignature<ObjectType> = {
	[KeyType in keyof ObjectType
	// Is `{}` assignable to `Record<KeyType, unknown>`?
	as {} extends Record<KeyType, unknown>
		? never // ✅ `{}` is assignable to `Record<KeyType, unknown>`
		: KeyType // ❌ `{}` is NOT assignable to `Record<KeyType, unknown>`
	]: ObjectType[KeyType];
};
```

If `{}` is assignable, it means that `KeyType` is an index signature and we want to remove it. If it is not assignable, `KeyType` is a "real" key and we want to keep it.

@example
```
import type {OmitIndexSignature} from 'type-fest';

type Example = {
	// These index signatures will be removed.
	[x: string]: any;
	[x: number]: any;
	[x: symbol]: any;
	[x: `head-${string}`]: string;
	[x: `${string}-tail`]: string;
	[x: `head-${string}-tail`]: string;
	[x: `${bigint}`]: string;
	[x: `embedded-${number}`]: string;

	// These explicitly defined keys will remain.
	foo: 'bar';
	qux?: 'baz';
};

type ExampleWithoutIndexSignatures = OmitIndexSignature<Example>;
//=> {foo: 'bar'; qux?: 'baz'}
```

@see {@link PickIndexSignature}
@category Object
*/
type OmitIndexSignature<ObjectType> = { [KeyType in keyof ObjectType as {} extends Record<KeyType, unknown> ? never : KeyType]: ObjectType[KeyType] };
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/pick-index-signature.d.ts
/**
Pick only index signatures from the given object type, leaving out all explicitly defined properties.

This is the counterpart of `OmitIndexSignature`.

@example
```
import type {PickIndexSignature} from 'type-fest';

declare const symbolKey: unique symbol;

type Example = {
	// These index signatures will remain.
	[x: string]: unknown;
	[x: number]: unknown;
	[x: symbol]: unknown;
	[x: `head-${string}`]: string;
	[x: `${string}-tail`]: string;
	[x: `head-${string}-tail`]: string;
	[x: `${bigint}`]: string;
	[x: `embedded-${number}`]: string;

	// These explicitly defined keys will be removed.
	['kebab-case-key']: string;
	[symbolKey]: string;
	foo: 'bar';
	qux?: 'baz';
};

type ExampleIndexSignature = PickIndexSignature<Example>;
// {
// 	[x: string]: unknown;
// 	[x: number]: unknown;
// 	[x: symbol]: unknown;
// 	[x: `head-${string}`]: string;
// 	[x: `${string}-tail`]: string;
// 	[x: `head-${string}-tail`]: string;
// 	[x: `${bigint}`]: string;
// 	[x: `embedded-${number}`]: string;
// }
```

@see {@link OmitIndexSignature}
@category Object
*/
type PickIndexSignature<ObjectType> = { [KeyType in keyof ObjectType as {} extends Record<KeyType, unknown> ? KeyType : never]: ObjectType[KeyType] };
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/merge.d.ts
// Merges two objects without worrying about index signatures.
type SimpleMerge<Destination, Source> = Simplify<{ [Key in keyof Destination as Key extends keyof Source ? never : Key]: Destination[Key] } & Source>;
/**
Merge two types into a new type. Keys of the second type overrides keys of the first type.

@example
```
import type {Merge} from 'type-fest';

type Foo = {
	[x: string]: unknown;
	[x: number]: unknown;
	foo: string;
	bar: symbol;
};

type Bar = {
	[x: number]: number;
	[x: symbol]: unknown;
	bar: Date;
	baz: boolean;
};

export type FooBar = Merge<Foo, Bar>;
//=> {
// 	[x: string]: unknown;
// 	[x: number]: number;
// 	[x: symbol]: unknown;
// 	foo: string;
// 	bar: Date;
// 	baz: boolean;
// }
```

Note: If you want a merge type that more accurately reflects the runtime behavior of object spread or `Object.assign`, refer to the {@link ObjectMerge} type.

@see {@link ObjectMerge}
@category Object
*/
type Merge<Destination, Source> = Destination extends unknown // For distributing `Destination`
? Source extends unknown // For distributing `Source`
? Simplify<SimpleMerge<PickIndexSignature<Destination>, PickIndexSignature<Source>> & SimpleMerge<OmitIndexSignature<Destination>, OmitIndexSignature<Source>>> : never // Should never happen
: never;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/internal/object.d.ts
/**
Merges user specified options with default options.

@example
```
type PathsOptions = {maxRecursionDepth?: number; leavesOnly?: boolean};
type DefaultPathsOptions = {maxRecursionDepth: 10; leavesOnly: false};
type SpecifiedOptions = {leavesOnly: true};

type Result = ApplyDefaultOptions<PathsOptions, DefaultPathsOptions, SpecifiedOptions>;
//=> {maxRecursionDepth: 10; leavesOnly: true}
```

@example
```
// Complains if default values are not provided for optional options

type PathsOptions = {maxRecursionDepth?: number; leavesOnly?: boolean};
type DefaultPathsOptions = {maxRecursionDepth: 10};
type SpecifiedOptions = {};

type Result = ApplyDefaultOptions<PathsOptions, DefaultPathsOptions, SpecifiedOptions>;
//                                              ~~~~~~~~~~~~~~~~~~~
// Property 'leavesOnly' is missing in type 'DefaultPathsOptions' but required in type '{ maxRecursionDepth: number; leavesOnly: boolean; }'.
```

@example
```
// Complains if an option's default type does not conform to the expected type

type PathsOptions = {maxRecursionDepth?: number; leavesOnly?: boolean};
type DefaultPathsOptions = {maxRecursionDepth: 10; leavesOnly: 'no'};
type SpecifiedOptions = {};

type Result = ApplyDefaultOptions<PathsOptions, DefaultPathsOptions, SpecifiedOptions>;
//                                              ~~~~~~~~~~~~~~~~~~~
// Types of property 'leavesOnly' are incompatible. Type 'string' is not assignable to type 'boolean'.
```

@example
```
// Complains if an option's specified type does not conform to the expected type

type PathsOptions = {maxRecursionDepth?: number; leavesOnly?: boolean};
type DefaultPathsOptions = {maxRecursionDepth: 10; leavesOnly: false};
type SpecifiedOptions = {leavesOnly: 'yes'};

type Result = ApplyDefaultOptions<PathsOptions, DefaultPathsOptions, SpecifiedOptions>;
//                                                                   ~~~~~~~~~~~~~~~~
// Types of property 'leavesOnly' are incompatible. Type 'string' is not assignable to type 'boolean'.
```
*/
type ApplyDefaultOptions<Options extends object, Defaults extends Simplify<Omit<Required<Options>, RequiredKeysOf<Options>> & Partial<Record<RequiredKeysOf<Options>, never>>>, SpecifiedOptions extends Options> = If<IsAny<SpecifiedOptions>, Defaults, If<IsNever<SpecifiedOptions>, Defaults, Simplify<Merge<Defaults, { [Key in keyof SpecifiedOptions as Key extends OptionalKeysOf<Options> ? undefined extends SpecifiedOptions[Key] ? never : Key : Key]: SpecifiedOptions[Key] }> & Required<Options>>>>;
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/except.d.ts
/**
Filter out keys from an object.

Returns `never` if `Exclude` is strictly equal to `Key`.
Returns `never` if `Key` extends `Exclude`.
Returns `Key` otherwise.

@example
```
type Filtered = Filter<'foo', 'foo'>;
//=> never
```

@example
```
type Filtered = Filter<'bar', string>;
//=> never
```

@example
```
type Filtered = Filter<'bar', 'foo'>;
//=> 'bar'
```

@see {Except}
*/
type Filter<KeyType, ExcludeType> = IsEqual<KeyType, ExcludeType> extends true ? never : (KeyType extends ExcludeType ? never : KeyType);
type ExceptOptions = {
  /**
  Disallow assigning non-specified properties.
  	Note that any omitted properties in the resulting type will be present in autocomplete as `undefined`.
  	@default false
  */
  requireExactProps?: boolean;
};
type DefaultExceptOptions = {
  requireExactProps: false;
};
/**
Create a type from an object type without certain keys.

We recommend setting the `requireExactProps` option to `true`.

This type is a stricter version of [`Omit`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-5.html#the-omit-helper-type). The `Omit` type does not restrict the omitted keys to be keys present on the given type, while `Except` does. The benefits of a stricter type are avoiding typos and allowing the compiler to pick up on rename refactors automatically.

This type was proposed to the TypeScript team, which declined it, saying they prefer that libraries implement stricter versions of the built-in types ([microsoft/TypeScript#30825](https://github.com/microsoft/TypeScript/issues/30825#issuecomment-523668235)).

@example
```
import type {Except} from 'type-fest';

type Foo = {
	a: number;
	b: string;
};

type FooWithoutA = Except<Foo, 'a'>;
//=> {b: string}

// @ts-expect-error
const fooWithoutA: FooWithoutA = {a: 1, b: '2'};
// errors: 'a' does not exist in type '{ b: string; }'

type FooWithoutB = Except<Foo, 'b', {requireExactProps: true}>;
//=> {a: number} & Partial<Record<'b', never>>

// @ts-expect-error
const fooWithoutB: FooWithoutB = {a: 1, b: '2'};
// errors at 'b': Type 'string' is not assignable to type 'undefined'.

// The `Omit` utility type doesn't work when omitting specific keys from objects containing index signatures.

// Consider the following example:

type UserData = {
	[metadata: string]: string;
	email: string;
	name: string;
	role: 'admin' | 'user';
};

// `Omit` clearly doesn't behave as expected in this case:
type PostPayload = Omit<UserData, 'email'>;
//=> {[x: string]: string; [x: number]: string}

// In situations like this, `Except` works better.
// It simply removes the `email` key while preserving all the other keys.
type PostPayloadFixed = Except<UserData, 'email'>;
//=> {[x: string]: string; name: string; role: 'admin' | 'user'}
```

@category Object
*/
type Except<ObjectType, KeysType extends keyof ObjectType, Options extends ExceptOptions = {}> = _Except<ObjectType, KeysType, ApplyDefaultOptions<ExceptOptions, DefaultExceptOptions, Options>>;
type _Except<ObjectType, KeysType extends keyof ObjectType, Options extends Required<ExceptOptions>> = { [KeyType in keyof ObjectType as Filter<KeyType, KeysType>]: ObjectType[KeyType] } & (Options['requireExactProps'] extends true ? Partial<Record<KeysType, never>> : {});
//#endregion
//#region ../../node_modules/.pnpm/type-fest@5.4.2/node_modules/type-fest/source/require-at-least-one.d.ts
/**
Create a type that requires at least one of the given keys. The remaining keys are kept as is.

@example
```
import type {RequireAtLeastOne} from 'type-fest';

type Responder = {
	text?: () => string;
	json?: () => string;
	secure?: boolean;
};

const responder: RequireAtLeastOne<Responder, 'text' | 'json'> = {
	json: () => '{"message": "ok"}',
	secure: true,
};
```

@category Object
*/
type RequireAtLeastOne<ObjectType, KeysType extends keyof ObjectType = keyof ObjectType> = IfNotAnyOrNever<ObjectType, If<IsNever<KeysType>, never, _RequireAtLeastOne<ObjectType, If<IsAny<KeysType>, keyof ObjectType, KeysType>>>>;
type _RequireAtLeastOne<ObjectType, KeysType extends keyof ObjectType> = { // For each `Key` in `KeysType` make a mapped type:
[Key in KeysType]-?: Required<Pick<ObjectType, Key>> & // 1. Make `Key`'s type required
// 2. Make all other keys in `KeysType` optional
Partial<Pick<ObjectType, Exclude<KeysType, Key>>> }[KeysType] & // 3. Add the remaining keys not in `KeysType`
Except<ObjectType, KeysType>;
//#endregion
//#region src-js/plugins/tokens.d.ts
/**
 * Options for various `SourceCode` methods e.g. `getFirstToken`.
 */
interface SkipOptions {
  /** Number of skipping tokens */
  skip?: number;
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
  /** Function to filter tokens */
  filter?: FilterFn | null;
}
/**
 * Options for various `SourceCode` methods e.g. `getFirstTokens`.
 */
interface CountOptions {
  /** Maximum number of tokens to return */
  count?: number;
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
  /** Function to filter tokens */
  filter?: FilterFn | null;
}
/**
 * Options for `getTokenByRangeStart`.
 */
interface RangeOptions {
  /** `true` to include comment tokens in the result */
  includeComments?: boolean;
}
/**
 * Filter function, passed as `filter` property of `SkipOptions` and `CountOptions`.
 */
type FilterFn = (token: TokenOrComment) => boolean;
/**
 * AST token type.
 */
type Token = BooleanToken | IdentifierToken | JSXIdentifierToken | JSXTextToken | KeywordToken | NullToken | NumericToken | PrivateIdentifierToken | PunctuatorToken | RegularExpressionToken | StringToken | TemplateToken;
interface BaseToken extends Span {
  value: string;
}
interface BooleanToken extends BaseToken {
  type: "Boolean";
}
interface IdentifierToken extends BaseToken {
  type: "Identifier";
}
interface JSXIdentifierToken extends BaseToken {
  type: "JSXIdentifier";
}
interface JSXTextToken extends BaseToken {
  type: "JSXText";
}
interface KeywordToken extends BaseToken {
  type: "Keyword";
}
interface NullToken extends BaseToken {
  type: "Null";
}
interface NumericToken extends BaseToken {
  type: "Numeric";
}
interface PrivateIdentifierToken extends BaseToken {
  type: "PrivateIdentifier";
}
interface PunctuatorToken extends BaseToken {
  type: "Punctuator";
}
interface RegularExpressionToken extends BaseToken {
  type: "RegularExpression";
  regex: {
    flags: string;
    pattern: string;
  };
}
interface StringToken extends BaseToken {
  type: "String";
}
interface TemplateToken extends BaseToken {
  type: "Template";
}
type TokenOrComment = Token | Comment;
/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s.
 */
/**
 * Get all tokens that are related to the given node.
 * @param node - The AST node.
 * @param beforeCount? - The number of tokens before the node to retrieve.
 * @param afterCount? - The number of tokens after the node to retrieve.
 * @returns Array of `Token`s.
 */
declare function getTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null, afterCount?: number | null): TokenOrComment[];
/**
 * Get the first token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getFirstToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the first tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s.
 */
declare function getFirstTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the last token of the given node.
 * @param node - The AST node.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getLastToken(node: Node, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the last tokens of the given node.
 * @param node - The AST node.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s.
 */
declare function getLastTokens(node: Node, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the token that precedes a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getTokenBefore(nodeOrToken: NodeOrToken, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the token that precedes a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenBefore` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getTokenOrCommentBefore(nodeOrToken: NodeOrToken, skip?: number): TokenOrComment | null;
/**
 * Get the tokens that precede a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s.
 */
declare function getTokensBefore(nodeOrToken: NodeOrToken, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the token that follows a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getTokenAfter(nodeOrToken: NodeOrToken, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the token that follows a given node or token.
 *
 * @deprecated Use `sourceCode.getTokenAfter` with `includeComments: true` instead.
 *
 * @param nodeOrToken The AST node or token.
 * @param skip - Number of tokens to skip.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getTokenOrCommentAfter(nodeOrToken: NodeOrToken, skip?: number): TokenOrComment | null;
/**
 * Get the tokens that follow a given node or token.
 * @param nodeOrToken - The AST node or token.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s.
 */
declare function getTokensAfter(nodeOrToken: NodeOrToken, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object. If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s between `left` and `right`.
 */
/**
 * Get all of the tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param padding - Number of extra tokens on either side of center.
 * @returns Array of `Token`s between `left` and `right`.
 */
declare function getTokensBetween(left: NodeOrToken, right: NodeOrToken, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the first token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getFirstTokenBetween(left: NodeOrToken, right: NodeOrToken, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the first tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s between `left` and `right`.
 */
declare function getFirstTokensBetween(left: NodeOrToken, right: NodeOrToken, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the last token between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param skipOptions? - Options object.
 *   If is a number, equivalent to `{ skip: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns `Token`, or `null` if all were skipped.
 */
declare function getLastTokenBetween(left: NodeOrToken, right: NodeOrToken, skipOptions?: SkipOptions | number | FilterFn | null): TokenOrComment | null;
/**
 * Get the last tokens between two non-overlapping nodes.
 * @param left - Node or token before the desired token range.
 * @param right - Node or token after the desired token range.
 * @param countOptions? - Options object.
 *   If is a number, equivalent to `{ count: n }`.
 *   If is a function, equivalent to `{ filter: fn }`.
 * @returns Array of `Token`s between `left` and `right`.
 */
declare function getLastTokensBetween(left: NodeOrToken, right: NodeOrToken, countOptions?: CountOptions | number | FilterFn | null): TokenOrComment[];
/**
 * Get the token starting at the specified index.
 * @param index - Index of the start of the token's range.
 * @param rangeOptions - Options object.
 * @returns The token starting at index, or `null` if no such token.
 */
declare function getTokenByRangeStart(index: number, rangeOptions?: RangeOptions | null): TokenOrComment | null;
/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter.
 *
 * Returns `false` if the given nodes or tokens overlap.
 *
 * Checks for whitespace *between tokens*, not including whitespace *inside tokens*.
 * e.g. Returns `false` for `isSpaceBetween(x, y)` in `x+" "+y`.
 *
 * @param first - The first node or token to check between.
 * @param second - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
declare function isSpaceBetween(first: NodeOrToken, second: NodeOrToken): boolean;
/**
 * Determine if two nodes or tokens have at least one whitespace character between them.
 * Order does not matter.
 *
 * Returns `false` if the given nodes or tokens overlap.
 *
 * Checks for whitespace *between tokens*, not including whitespace *inside tokens*.
 * e.g. Returns `false` for `isSpaceBetween(x, y)` in `x+" "+y`.
 *
 * Unlike `SourceCode#isSpaceBetween`, this function does return `true` if there is a `JSText` token between the two
 * input tokens, and it contains whitespace.
 * e.g. Returns `true` for `isSpaceBetweenTokens(x, slash)` in `<X>a b</X>`.
 *
 * @deprecated Use `sourceCode.isSpaceBetween` instead.
 *
 * @param first - The first node or token to check between.
 * @param second - The second node or token to check between.
 * @returns `true` if there is a whitespace character between
 *   any of the tokens found between the two given nodes or tokens.
 */
declare function isSpaceBetweenTokens(first: NodeOrToken, second: NodeOrToken): boolean;
declare namespace types_d_exports {
  export { AccessorProperty, AccessorPropertyType, Argument, ArrayAssignmentTarget, ArrayExpression, ArrayExpressionElement, ArrayPattern, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator, AssignmentPattern, AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetPattern, AssignmentTargetProperty, AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty, AssignmentTargetRest, AssignmentTargetWithDefault, AwaitExpression, BigIntLiteral, BinaryExpression, BinaryOperator, BindingIdentifier, BindingPattern, BindingProperty, BindingRestElement, BlockStatement, BooleanLiteral, BreakStatement, CallExpression, CatchClause, ChainElement, ChainExpression, Class, ClassBody, ClassElement, ClassType, Comment, ComputedMemberExpression, ConditionalExpression, ContinueStatement, DebuggerStatement, Declaration, Decorator, Directive, DoWhileStatement, EmptyStatement, ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration, ExportSpecifier, Expression, ExpressionStatement, ForInStatement, ForOfStatement, ForStatement, ForStatementInit, ForStatementLeft, FormalParameter, FormalParameterRest, Function$1 as Function, FunctionBody, FunctionType, Hashbang, IdentifierName, IdentifierReference, IfStatement, ImportAttribute, ImportAttributeKey, ImportDeclaration, ImportDeclarationSpecifier, ImportDefaultSpecifier, ImportExpression, ImportNamespaceSpecifier, ImportOrExportKind, ImportPhase, ImportSpecifier, JSDocNonNullableType, JSDocNullableType, JSDocUnknownType, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXClosingElement, JSXClosingFragment, JSXElement, JSXElementName, JSXEmptyExpression, JSXExpression, JSXExpressionContainer, JSXFragment, JSXIdentifier, JSXMemberExpression, JSXMemberExpressionObject, JSXNamespacedName, JSXOpeningElement, JSXOpeningFragment, JSXSpreadAttribute, JSXSpreadChild, JSXText, LabelIdentifier, LabeledStatement, LogicalExpression, LogicalOperator, MemberExpression, MetaProperty, MethodDefinition, MethodDefinitionKind, MethodDefinitionType, ModuleDeclaration, ModuleExportName, ModuleKind, NewExpression, Node$1 as Node, NullLiteral, NumericLiteral, ObjectAssignmentTarget, ObjectExpression, ObjectPattern, ObjectProperty, ObjectPropertyKind, ParamPattern, ParenthesizedExpression, PrivateFieldExpression, PrivateIdentifier, PrivateInExpression, Program, PropertyDefinition, PropertyDefinitionType, PropertyKey$1 as PropertyKey, PropertyKind, RegExpLiteral, ReturnStatement, SequenceExpression, SimpleAssignmentTarget, Span, SpreadElement, Statement, StaticBlock, StaticMemberExpression, StringLiteral, Super, SwitchCase, SwitchStatement, TSAccessibility, TSAnyKeyword, TSArrayType, TSAsExpression, TSBigIntKeyword, TSBooleanKeyword, TSCallSignatureDeclaration, TSClassImplements, TSConditionalType, TSConstructSignatureDeclaration, TSConstructorType, TSEnumBody, TSEnumDeclaration, TSEnumMember, TSEnumMemberName, TSExportAssignment, TSExternalModuleReference, TSFunctionType, TSGlobalDeclaration, TSImportEqualsDeclaration, TSImportType, TSImportTypeQualifiedName, TSImportTypeQualifier, TSIndexSignature, TSIndexSignatureName, TSIndexedAccessType, TSInferType, TSInstantiationExpression, TSInterfaceBody, TSInterfaceDeclaration, TSInterfaceHeritage, TSIntersectionType, TSIntrinsicKeyword, TSLiteral, TSLiteralType, TSMappedType, TSMappedTypeModifierOperator, TSMethodSignature, TSMethodSignatureKind, TSModuleBlock, TSModuleDeclaration, TSModuleDeclarationKind, TSModuleReference, TSNamedTupleMember, TSNamespaceExportDeclaration, TSNeverKeyword, TSNonNullExpression, TSNullKeyword, TSNumberKeyword, TSObjectKeyword, TSOptionalType, TSParameterProperty, TSParenthesizedType, TSPropertySignature, TSQualifiedName, TSRestType, TSSatisfiesExpression, TSSignature, TSStringKeyword, TSSymbolKeyword, TSTemplateLiteralType, TSThisParameter, TSThisType, TSTupleElement, TSTupleType, TSType, TSTypeAliasDeclaration, TSTypeAnnotation, TSTypeAssertion, TSTypeLiteral, TSTypeName, TSTypeOperator, TSTypeOperatorOperator, TSTypeParameter, TSTypeParameterDeclaration, TSTypeParameterInstantiation, TSTypePredicate, TSTypePredicateName, TSTypeQuery, TSTypeQueryExprName, TSTypeReference, TSUndefinedKeyword, TSUnionType, TSUnknownKeyword, TSVoidKeyword, TaggedTemplateExpression, TemplateElement, TemplateElementValue, TemplateLiteral, ThisExpression, ThrowStatement, Token, TryStatement, UnaryExpression, UnaryOperator, UpdateExpression, UpdateOperator, V8IntrinsicExpression, VariableDeclaration, VariableDeclarationKind, VariableDeclarator, WhileStatement, WithStatement, YieldExpression };
}
interface Program extends Span {
  type: "Program";
  body: Array<Directive | Statement>;
  sourceType: ModuleKind;
  hashbang: Hashbang | null;
  comments: Comment[];
  tokens: Token[];
  parent: null;
}
type Expression = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function$1 | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | V8IntrinsicExpression | MemberExpression;
interface IdentifierName extends Span {
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface IdentifierReference extends Span {
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface BindingIdentifier extends Span {
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface LabelIdentifier extends Span {
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface ThisExpression extends Span {
  type: "ThisExpression";
  parent: Node$1;
}
interface ArrayExpression extends Span {
  type: "ArrayExpression";
  elements: Array<ArrayExpressionElement>;
  parent: Node$1;
}
type ArrayExpressionElement = SpreadElement | null | Expression;
interface ObjectExpression extends Span {
  type: "ObjectExpression";
  properties: Array<ObjectPropertyKind>;
  parent: Node$1;
}
type ObjectPropertyKind = ObjectProperty | SpreadElement;
interface ObjectProperty extends Span {
  type: "Property";
  kind: PropertyKind;
  key: PropertyKey$1;
  value: Expression;
  method: boolean;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  parent: Node$1;
}
type PropertyKey$1 = IdentifierName | PrivateIdentifier | Expression;
type PropertyKind = "init" | "get" | "set";
interface TemplateLiteral extends Span {
  type: "TemplateLiteral";
  quasis: Array<TemplateElement>;
  expressions: Array<Expression>;
  parent: Node$1;
}
interface TaggedTemplateExpression extends Span {
  type: "TaggedTemplateExpression";
  tag: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  quasi: TemplateLiteral;
  parent: Node$1;
}
interface TemplateElement extends Span {
  type: "TemplateElement";
  value: TemplateElementValue;
  tail: boolean;
  parent: Node$1;
}
interface TemplateElementValue {
  raw: string;
  cooked: string | null;
}
type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;
interface ComputedMemberExpression extends Span {
  type: "MemberExpression";
  object: Expression;
  property: Expression;
  optional: boolean;
  computed: true;
  parent: Node$1;
}
interface StaticMemberExpression extends Span {
  type: "MemberExpression";
  object: Expression;
  property: IdentifierName;
  optional: boolean;
  computed: false;
  parent: Node$1;
}
interface PrivateFieldExpression extends Span {
  type: "MemberExpression";
  object: Expression;
  property: PrivateIdentifier;
  optional: boolean;
  computed: false;
  parent: Node$1;
}
interface CallExpression extends Span {
  type: "CallExpression";
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  optional: boolean;
  parent: Node$1;
}
interface NewExpression extends Span {
  type: "NewExpression";
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  parent: Node$1;
}
interface MetaProperty extends Span {
  type: "MetaProperty";
  meta: IdentifierName;
  property: IdentifierName;
  parent: Node$1;
}
interface SpreadElement extends Span {
  type: "SpreadElement";
  argument: Expression;
  parent: Node$1;
}
type Argument = SpreadElement | Expression;
interface UpdateExpression extends Span {
  type: "UpdateExpression";
  operator: UpdateOperator;
  prefix: boolean;
  argument: SimpleAssignmentTarget;
  parent: Node$1;
}
interface UnaryExpression extends Span {
  type: "UnaryExpression";
  operator: UnaryOperator;
  argument: Expression;
  prefix: true;
  parent: Node$1;
}
interface BinaryExpression extends Span {
  type: "BinaryExpression";
  left: Expression;
  operator: BinaryOperator;
  right: Expression;
  parent: Node$1;
}
interface PrivateInExpression extends Span {
  type: "BinaryExpression";
  left: PrivateIdentifier;
  operator: "in";
  right: Expression;
  parent: Node$1;
}
interface LogicalExpression extends Span {
  type: "LogicalExpression";
  left: Expression;
  operator: LogicalOperator;
  right: Expression;
  parent: Node$1;
}
interface ConditionalExpression extends Span {
  type: "ConditionalExpression";
  test: Expression;
  consequent: Expression;
  alternate: Expression;
  parent: Node$1;
}
interface AssignmentExpression extends Span {
  type: "AssignmentExpression";
  operator: AssignmentOperator;
  left: AssignmentTarget;
  right: Expression;
  parent: Node$1;
}
type AssignmentTarget = SimpleAssignmentTarget | AssignmentTargetPattern;
type SimpleAssignmentTarget = IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | MemberExpression;
type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget;
interface ArrayAssignmentTarget extends Span {
  type: "ArrayPattern";
  decorators?: [];
  elements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface ObjectAssignmentTarget extends Span {
  type: "ObjectPattern";
  decorators?: [];
  properties: Array<AssignmentTargetProperty | AssignmentTargetRest>;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface AssignmentTargetRest extends Span {
  type: "RestElement";
  decorators?: [];
  argument: AssignmentTarget;
  optional?: false;
  typeAnnotation?: null;
  value?: null;
  parent: Node$1;
}
type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTarget;
interface AssignmentTargetWithDefault extends Span {
  type: "AssignmentPattern";
  decorators?: [];
  left: AssignmentTarget;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty;
interface AssignmentTargetPropertyIdentifier extends Span {
  type: "Property";
  kind: "init";
  key: IdentifierReference;
  value: IdentifierReference | AssignmentTargetWithDefault;
  method: false;
  shorthand: true;
  computed: false;
  optional?: false;
  parent: Node$1;
}
interface AssignmentTargetPropertyProperty extends Span {
  type: "Property";
  kind: "init";
  key: PropertyKey$1;
  value: AssignmentTargetMaybeDefault;
  method: false;
  shorthand: false;
  computed: boolean;
  optional?: false;
  parent: Node$1;
}
interface SequenceExpression extends Span {
  type: "SequenceExpression";
  expressions: Array<Expression>;
  parent: Node$1;
}
interface Super extends Span {
  type: "Super";
  parent: Node$1;
}
interface AwaitExpression extends Span {
  type: "AwaitExpression";
  argument: Expression;
  parent: Node$1;
}
interface ChainExpression extends Span {
  type: "ChainExpression";
  expression: ChainElement;
  parent: Node$1;
}
type ChainElement = CallExpression | TSNonNullExpression | MemberExpression;
interface ParenthesizedExpression extends Span {
  type: "ParenthesizedExpression";
  expression: Expression;
  parent: Node$1;
}
type Statement = BlockStatement | BreakStatement | ContinueStatement | DebuggerStatement | DoWhileStatement | EmptyStatement | ExpressionStatement | ForInStatement | ForOfStatement | ForStatement | IfStatement | LabeledStatement | ReturnStatement | SwitchStatement | ThrowStatement | TryStatement | WhileStatement | WithStatement | Declaration | ModuleDeclaration;
interface Directive extends Span {
  type: "ExpressionStatement";
  expression: StringLiteral;
  directive: string;
  parent: Node$1;
}
interface Hashbang extends Span {
  type: "Hashbang";
  value: string;
  parent: Node$1;
}
interface BlockStatement extends Span {
  type: "BlockStatement";
  body: Array<Statement>;
  parent: Node$1;
}
type Declaration = VariableDeclaration | Function$1 | Class | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSGlobalDeclaration | TSImportEqualsDeclaration;
interface VariableDeclaration extends Span {
  type: "VariableDeclaration";
  kind: VariableDeclarationKind;
  declarations: Array<VariableDeclarator>;
  declare?: boolean;
  parent: Node$1;
}
type VariableDeclarationKind = "var" | "let" | "const" | "using" | "await using";
interface VariableDeclarator extends Span {
  type: "VariableDeclarator";
  id: BindingPattern;
  init: Expression | null;
  definite?: boolean;
  parent: Node$1;
}
interface EmptyStatement extends Span {
  type: "EmptyStatement";
  parent: Node$1;
}
interface ExpressionStatement extends Span {
  type: "ExpressionStatement";
  expression: Expression;
  directive?: string | null;
  parent: Node$1;
}
interface IfStatement extends Span {
  type: "IfStatement";
  test: Expression;
  consequent: Statement;
  alternate: Statement | null;
  parent: Node$1;
}
interface DoWhileStatement extends Span {
  type: "DoWhileStatement";
  body: Statement;
  test: Expression;
  parent: Node$1;
}
interface WhileStatement extends Span {
  type: "WhileStatement";
  test: Expression;
  body: Statement;
  parent: Node$1;
}
interface ForStatement extends Span {
  type: "ForStatement";
  init: ForStatementInit | null;
  test: Expression | null;
  update: Expression | null;
  body: Statement;
  parent: Node$1;
}
type ForStatementInit = VariableDeclaration | Expression;
interface ForInStatement extends Span {
  type: "ForInStatement";
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  parent: Node$1;
}
type ForStatementLeft = VariableDeclaration | AssignmentTarget;
interface ForOfStatement extends Span {
  type: "ForOfStatement";
  await: boolean;
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  parent: Node$1;
}
interface ContinueStatement extends Span {
  type: "ContinueStatement";
  label: LabelIdentifier | null;
  parent: Node$1;
}
interface BreakStatement extends Span {
  type: "BreakStatement";
  label: LabelIdentifier | null;
  parent: Node$1;
}
interface ReturnStatement extends Span {
  type: "ReturnStatement";
  argument: Expression | null;
  parent: Node$1;
}
interface WithStatement extends Span {
  type: "WithStatement";
  object: Expression;
  body: Statement;
  parent: Node$1;
}
interface SwitchStatement extends Span {
  type: "SwitchStatement";
  discriminant: Expression;
  cases: Array<SwitchCase>;
  parent: Node$1;
}
interface SwitchCase extends Span {
  type: "SwitchCase";
  test: Expression | null;
  consequent: Array<Statement>;
  parent: Node$1;
}
interface LabeledStatement extends Span {
  type: "LabeledStatement";
  label: LabelIdentifier;
  body: Statement;
  parent: Node$1;
}
interface ThrowStatement extends Span {
  type: "ThrowStatement";
  argument: Expression;
  parent: Node$1;
}
interface TryStatement extends Span {
  type: "TryStatement";
  block: BlockStatement;
  handler: CatchClause | null;
  finalizer: BlockStatement | null;
  parent: Node$1;
}
interface CatchClause extends Span {
  type: "CatchClause";
  param: BindingPattern | null;
  body: BlockStatement;
  parent: Node$1;
}
interface DebuggerStatement extends Span {
  type: "DebuggerStatement";
  parent: Node$1;
}
type BindingPattern = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;
interface AssignmentPattern extends Span {
  type: "AssignmentPattern";
  decorators?: [];
  left: BindingPattern;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface ObjectPattern extends Span {
  type: "ObjectPattern";
  decorators?: [];
  properties: Array<BindingProperty | BindingRestElement>;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface BindingProperty extends Span {
  type: "Property";
  kind: "init";
  key: PropertyKey$1;
  value: BindingPattern;
  method: false;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  parent: Node$1;
}
interface ArrayPattern extends Span {
  type: "ArrayPattern";
  decorators?: [];
  elements: Array<BindingPattern | BindingRestElement | null>;
  optional?: false;
  typeAnnotation?: null;
  parent: Node$1;
}
interface BindingRestElement extends Span {
  type: "RestElement";
  decorators?: [];
  argument: BindingPattern;
  optional?: false;
  typeAnnotation?: null;
  value?: null;
  parent: Node$1;
}
interface Function$1 extends Span {
  type: FunctionType;
  id: BindingIdentifier | null;
  generator: boolean;
  async: boolean;
  declare?: boolean;
  typeParameters?: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType?: TSTypeAnnotation | null;
  body: FunctionBody | null;
  expression: false;
  parent: Node$1;
}
type ParamPattern = FormalParameter | TSParameterProperty | FormalParameterRest;
type FunctionType = "FunctionDeclaration" | "FunctionExpression" | "TSDeclareFunction" | "TSEmptyBodyFunctionExpression";
interface FormalParameterRest extends Span {
  type: "RestElement";
  argument: BindingPattern;
  decorators?: [];
  optional?: boolean;
  typeAnnotation?: TSTypeAnnotation | null;
  value?: null;
  parent: Node$1;
}
type FormalParameter = {
  decorators?: Array<Decorator>;
} & BindingPattern;
interface TSParameterProperty extends Span {
  type: "TSParameterProperty";
  accessibility: TSAccessibility | null;
  decorators: Array<Decorator>;
  override: boolean;
  parameter: FormalParameter;
  readonly: boolean;
  static: boolean;
  parent: Node$1;
}
interface FunctionBody extends Span {
  type: "BlockStatement";
  body: Array<Directive | Statement>;
  parent: Node$1;
}
interface ArrowFunctionExpression extends Span {
  type: "ArrowFunctionExpression";
  expression: boolean;
  async: boolean;
  typeParameters?: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType?: TSTypeAnnotation | null;
  body: FunctionBody | Expression;
  id: null;
  generator: false;
  parent: Node$1;
}
interface YieldExpression extends Span {
  type: "YieldExpression";
  delegate: boolean;
  argument: Expression | null;
  parent: Node$1;
}
interface Class extends Span {
  type: ClassType;
  decorators: Array<Decorator>;
  id: BindingIdentifier | null;
  typeParameters?: TSTypeParameterDeclaration | null;
  superClass: Expression | null;
  superTypeArguments?: TSTypeParameterInstantiation | null;
  implements?: Array<TSClassImplements>;
  body: ClassBody;
  abstract?: boolean;
  declare?: boolean;
  parent: Node$1;
}
type ClassType = "ClassDeclaration" | "ClassExpression";
interface ClassBody extends Span {
  type: "ClassBody";
  body: Array<ClassElement>;
  parent: Node$1;
}
type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature;
interface MethodDefinition extends Span {
  type: MethodDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey$1;
  value: Function$1;
  kind: MethodDefinitionKind;
  computed: boolean;
  static: boolean;
  override?: boolean;
  optional?: boolean;
  accessibility?: TSAccessibility | null;
  parent: Node$1;
}
type MethodDefinitionType = "MethodDefinition" | "TSAbstractMethodDefinition";
interface PropertyDefinition extends Span {
  type: PropertyDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey$1;
  typeAnnotation?: TSTypeAnnotation | null;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  declare?: boolean;
  override?: boolean;
  optional?: boolean;
  definite?: boolean;
  readonly?: boolean;
  accessibility?: TSAccessibility | null;
  parent: Node$1;
}
type PropertyDefinitionType = "PropertyDefinition" | "TSAbstractPropertyDefinition";
type MethodDefinitionKind = "constructor" | "method" | "get" | "set";
interface PrivateIdentifier extends Span {
  type: "PrivateIdentifier";
  name: string;
  parent: Node$1;
}
interface StaticBlock extends Span {
  type: "StaticBlock";
  body: Array<Statement>;
  parent: Node$1;
}
type ModuleDeclaration = ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration;
type AccessorPropertyType = "AccessorProperty" | "TSAbstractAccessorProperty";
interface AccessorProperty extends Span {
  type: AccessorPropertyType;
  decorators: Array<Decorator>;
  key: PropertyKey$1;
  typeAnnotation?: TSTypeAnnotation | null;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  override?: boolean;
  definite?: boolean;
  accessibility?: TSAccessibility | null;
  declare?: false;
  optional?: false;
  readonly?: false;
  parent: Node$1;
}
interface ImportExpression extends Span {
  type: "ImportExpression";
  source: Expression;
  options: Expression | null;
  phase: ImportPhase | null;
  parent: Node$1;
}
interface ImportDeclaration extends Span {
  type: "ImportDeclaration";
  specifiers: Array<ImportDeclarationSpecifier>;
  source: StringLiteral;
  phase: ImportPhase | null;
  attributes: Array<ImportAttribute>;
  importKind?: ImportOrExportKind;
  parent: Node$1;
}
type ImportPhase = "source" | "defer";
type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier;
interface ImportSpecifier extends Span {
  type: "ImportSpecifier";
  imported: ModuleExportName;
  local: BindingIdentifier;
  importKind?: ImportOrExportKind;
  parent: Node$1;
}
interface ImportDefaultSpecifier extends Span {
  type: "ImportDefaultSpecifier";
  local: BindingIdentifier;
  parent: Node$1;
}
interface ImportNamespaceSpecifier extends Span {
  type: "ImportNamespaceSpecifier";
  local: BindingIdentifier;
  parent: Node$1;
}
interface ImportAttribute extends Span {
  type: "ImportAttribute";
  key: ImportAttributeKey;
  value: StringLiteral;
  parent: Node$1;
}
type ImportAttributeKey = IdentifierName | StringLiteral;
interface ExportNamedDeclaration extends Span {
  type: "ExportNamedDeclaration";
  declaration: Declaration | null;
  specifiers: Array<ExportSpecifier>;
  source: StringLiteral | null;
  exportKind?: ImportOrExportKind;
  attributes: Array<ImportAttribute>;
  parent: Node$1;
}
interface ExportDefaultDeclaration extends Span {
  type: "ExportDefaultDeclaration";
  declaration: ExportDefaultDeclarationKind;
  exportKind?: "value";
  parent: Node$1;
}
interface ExportAllDeclaration extends Span {
  type: "ExportAllDeclaration";
  exported: ModuleExportName | null;
  source: StringLiteral;
  attributes: Array<ImportAttribute>;
  exportKind?: ImportOrExportKind;
  parent: Node$1;
}
interface ExportSpecifier extends Span {
  type: "ExportSpecifier";
  local: ModuleExportName;
  exported: ModuleExportName;
  exportKind?: ImportOrExportKind;
  parent: Node$1;
}
type ExportDefaultDeclarationKind = Function$1 | Class | TSInterfaceDeclaration | Expression;
type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;
interface V8IntrinsicExpression extends Span {
  type: "V8IntrinsicExpression";
  name: IdentifierName;
  arguments: Array<Argument>;
  parent: Node$1;
}
interface BooleanLiteral extends Span {
  type: "Literal";
  value: boolean;
  raw: string | null;
  parent: Node$1;
}
interface NullLiteral extends Span {
  type: "Literal";
  value: null;
  raw: "null" | null;
  parent: Node$1;
}
interface NumericLiteral extends Span {
  type: "Literal";
  value: number;
  raw: string | null;
  parent: Node$1;
}
interface StringLiteral extends Span {
  type: "Literal";
  value: string;
  raw: string | null;
  parent: Node$1;
}
interface BigIntLiteral extends Span {
  type: "Literal";
  value: bigint;
  raw: string | null;
  bigint: string;
  parent: Node$1;
}
interface RegExpLiteral extends Span {
  type: "Literal";
  value: RegExp | null;
  raw: string | null;
  regex: {
    pattern: string;
    flags: string;
  };
  parent: Node$1;
}
interface JSXElement extends Span {
  type: "JSXElement";
  openingElement: JSXOpeningElement;
  children: Array<JSXChild>;
  closingElement: JSXClosingElement | null;
  parent: Node$1;
}
interface JSXOpeningElement extends Span {
  type: "JSXOpeningElement";
  name: JSXElementName;
  typeArguments?: TSTypeParameterInstantiation | null;
  attributes: Array<JSXAttributeItem>;
  selfClosing: boolean;
  parent: Node$1;
}
interface JSXClosingElement extends Span {
  type: "JSXClosingElement";
  name: JSXElementName;
  parent: Node$1;
}
interface JSXFragment extends Span {
  type: "JSXFragment";
  openingFragment: JSXOpeningFragment;
  children: Array<JSXChild>;
  closingFragment: JSXClosingFragment;
  parent: Node$1;
}
interface JSXOpeningFragment extends Span {
  type: "JSXOpeningFragment";
  attributes?: [];
  selfClosing?: false;
  parent: Node$1;
}
interface JSXClosingFragment extends Span {
  type: "JSXClosingFragment";
  parent: Node$1;
}
type JSXElementName = JSXIdentifier | JSXNamespacedName | JSXMemberExpression;
interface JSXNamespacedName extends Span {
  type: "JSXNamespacedName";
  namespace: JSXIdentifier;
  name: JSXIdentifier;
  parent: Node$1;
}
interface JSXMemberExpression extends Span {
  type: "JSXMemberExpression";
  object: JSXMemberExpressionObject;
  property: JSXIdentifier;
  parent: Node$1;
}
type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;
interface JSXExpressionContainer extends Span {
  type: "JSXExpressionContainer";
  expression: JSXExpression;
  parent: Node$1;
}
type JSXExpression = JSXEmptyExpression | Expression;
interface JSXEmptyExpression extends Span {
  type: "JSXEmptyExpression";
  parent: Node$1;
}
type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;
interface JSXAttribute extends Span {
  type: "JSXAttribute";
  name: JSXAttributeName;
  value: JSXAttributeValue | null;
  parent: Node$1;
}
interface JSXSpreadAttribute extends Span {
  type: "JSXSpreadAttribute";
  argument: Expression;
  parent: Node$1;
}
type JSXAttributeName = JSXIdentifier | JSXNamespacedName;
type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;
interface JSXIdentifier extends Span {
  type: "JSXIdentifier";
  name: string;
  parent: Node$1;
}
type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;
interface JSXSpreadChild extends Span {
  type: "JSXSpreadChild";
  expression: Expression;
  parent: Node$1;
}
interface JSXText extends Span {
  type: "JSXText";
  value: string;
  raw: string | null;
  parent: Node$1;
}
interface TSThisParameter extends Span {
  type: "Identifier";
  decorators: [];
  name: "this";
  optional: false;
  typeAnnotation: TSTypeAnnotation | null;
  parent: Node$1;
}
interface TSEnumDeclaration extends Span {
  type: "TSEnumDeclaration";
  id: BindingIdentifier;
  body: TSEnumBody;
  const: boolean;
  declare: boolean;
  parent: Node$1;
}
interface TSEnumBody extends Span {
  type: "TSEnumBody";
  members: Array<TSEnumMember>;
  parent: Node$1;
}
interface TSEnumMember extends Span {
  type: "TSEnumMember";
  id: TSEnumMemberName;
  initializer: Expression | null;
  computed: boolean;
  parent: Node$1;
}
type TSEnumMemberName = IdentifierName | StringLiteral | TemplateLiteral;
interface TSTypeAnnotation extends Span {
  type: "TSTypeAnnotation";
  typeAnnotation: TSType;
  parent: Node$1;
}
interface TSLiteralType extends Span {
  type: "TSLiteralType";
  literal: TSLiteral;
  parent: Node$1;
}
type TSLiteral = BooleanLiteral | NumericLiteral | BigIntLiteral | StringLiteral | TemplateLiteral | UnaryExpression;
type TSType = TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperator | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType;
interface TSConditionalType extends Span {
  type: "TSConditionalType";
  checkType: TSType;
  extendsType: TSType;
  trueType: TSType;
  falseType: TSType;
  parent: Node$1;
}
interface TSUnionType extends Span {
  type: "TSUnionType";
  types: Array<TSType>;
  parent: Node$1;
}
interface TSIntersectionType extends Span {
  type: "TSIntersectionType";
  types: Array<TSType>;
  parent: Node$1;
}
interface TSParenthesizedType extends Span {
  type: "TSParenthesizedType";
  typeAnnotation: TSType;
  parent: Node$1;
}
interface TSTypeOperator extends Span {
  type: "TSTypeOperator";
  operator: TSTypeOperatorOperator;
  typeAnnotation: TSType;
  parent: Node$1;
}
type TSTypeOperatorOperator = "keyof" | "unique" | "readonly";
interface TSArrayType extends Span {
  type: "TSArrayType";
  elementType: TSType;
  parent: Node$1;
}
interface TSIndexedAccessType extends Span {
  type: "TSIndexedAccessType";
  objectType: TSType;
  indexType: TSType;
  parent: Node$1;
}
interface TSTupleType extends Span {
  type: "TSTupleType";
  elementTypes: Array<TSTupleElement>;
  parent: Node$1;
}
interface TSNamedTupleMember extends Span {
  type: "TSNamedTupleMember";
  label: IdentifierName;
  elementType: TSTupleElement;
  optional: boolean;
  parent: Node$1;
}
interface TSOptionalType extends Span {
  type: "TSOptionalType";
  typeAnnotation: TSType;
  parent: Node$1;
}
interface TSRestType extends Span {
  type: "TSRestType";
  typeAnnotation: TSType;
  parent: Node$1;
}
type TSTupleElement = TSOptionalType | TSRestType | TSType;
interface TSAnyKeyword extends Span {
  type: "TSAnyKeyword";
  parent: Node$1;
}
interface TSStringKeyword extends Span {
  type: "TSStringKeyword";
  parent: Node$1;
}
interface TSBooleanKeyword extends Span {
  type: "TSBooleanKeyword";
  parent: Node$1;
}
interface TSNumberKeyword extends Span {
  type: "TSNumberKeyword";
  parent: Node$1;
}
interface TSNeverKeyword extends Span {
  type: "TSNeverKeyword";
  parent: Node$1;
}
interface TSIntrinsicKeyword extends Span {
  type: "TSIntrinsicKeyword";
  parent: Node$1;
}
interface TSUnknownKeyword extends Span {
  type: "TSUnknownKeyword";
  parent: Node$1;
}
interface TSNullKeyword extends Span {
  type: "TSNullKeyword";
  parent: Node$1;
}
interface TSUndefinedKeyword extends Span {
  type: "TSUndefinedKeyword";
  parent: Node$1;
}
interface TSVoidKeyword extends Span {
  type: "TSVoidKeyword";
  parent: Node$1;
}
interface TSSymbolKeyword extends Span {
  type: "TSSymbolKeyword";
  parent: Node$1;
}
interface TSThisType extends Span {
  type: "TSThisType";
  parent: Node$1;
}
interface TSObjectKeyword extends Span {
  type: "TSObjectKeyword";
  parent: Node$1;
}
interface TSBigIntKeyword extends Span {
  type: "TSBigIntKeyword";
  parent: Node$1;
}
interface TSTypeReference extends Span {
  type: "TSTypeReference";
  typeName: TSTypeName;
  typeArguments: TSTypeParameterInstantiation | null;
  parent: Node$1;
}
type TSTypeName = IdentifierReference | TSQualifiedName | ThisExpression;
interface TSQualifiedName extends Span {
  type: "TSQualifiedName";
  left: TSTypeName;
  right: IdentifierName;
  parent: Node$1;
}
interface TSTypeParameterInstantiation extends Span {
  type: "TSTypeParameterInstantiation";
  params: Array<TSType>;
  parent: Node$1;
}
interface TSTypeParameter extends Span {
  type: "TSTypeParameter";
  name: BindingIdentifier;
  constraint: TSType | null;
  default: TSType | null;
  in: boolean;
  out: boolean;
  const: boolean;
  parent: Node$1;
}
interface TSTypeParameterDeclaration extends Span {
  type: "TSTypeParameterDeclaration";
  params: Array<TSTypeParameter>;
  parent: Node$1;
}
interface TSTypeAliasDeclaration extends Span {
  type: "TSTypeAliasDeclaration";
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  typeAnnotation: TSType;
  declare: boolean;
  parent: Node$1;
}
type TSAccessibility = "private" | "protected" | "public";
interface TSClassImplements extends Span {
  type: "TSClassImplements";
  expression: IdentifierReference | ThisExpression | MemberExpression;
  typeArguments: TSTypeParameterInstantiation | null;
  parent: Node$1;
}
interface TSInterfaceDeclaration extends Span {
  type: "TSInterfaceDeclaration";
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  extends: Array<TSInterfaceHeritage>;
  body: TSInterfaceBody;
  declare: boolean;
  parent: Node$1;
}
interface TSInterfaceBody extends Span {
  type: "TSInterfaceBody";
  body: Array<TSSignature>;
  parent: Node$1;
}
interface TSPropertySignature extends Span {
  type: "TSPropertySignature";
  computed: boolean;
  optional: boolean;
  readonly: boolean;
  key: PropertyKey$1;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: null;
  static: false;
  parent: Node$1;
}
type TSSignature = TSIndexSignature | TSPropertySignature | TSCallSignatureDeclaration | TSConstructSignatureDeclaration | TSMethodSignature;
interface TSIndexSignature extends Span {
  type: "TSIndexSignature";
  parameters: Array<TSIndexSignatureName>;
  typeAnnotation: TSTypeAnnotation;
  readonly: boolean;
  static: boolean;
  accessibility: null;
  parent: Node$1;
}
interface TSCallSignatureDeclaration extends Span {
  type: "TSCallSignatureDeclaration";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  parent: Node$1;
}
type TSMethodSignatureKind = "method" | "get" | "set";
interface TSMethodSignature extends Span {
  type: "TSMethodSignature";
  key: PropertyKey$1;
  computed: boolean;
  optional: boolean;
  kind: TSMethodSignatureKind;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  accessibility: null;
  readonly: false;
  static: false;
  parent: Node$1;
}
interface TSConstructSignatureDeclaration extends Span {
  type: "TSConstructSignatureDeclaration";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  parent: Node$1;
}
interface TSIndexSignatureName extends Span {
  type: "Identifier";
  decorators: [];
  name: string;
  optional: false;
  typeAnnotation: TSTypeAnnotation;
  parent: Node$1;
}
interface TSInterfaceHeritage extends Span {
  type: "TSInterfaceHeritage";
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation | null;
  parent: Node$1;
}
interface TSTypePredicate extends Span {
  type: "TSTypePredicate";
  parameterName: TSTypePredicateName;
  asserts: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  parent: Node$1;
}
type TSTypePredicateName = IdentifierName | TSThisType;
interface TSModuleDeclaration extends Span {
  type: "TSModuleDeclaration";
  id: BindingIdentifier | StringLiteral | TSQualifiedName;
  body: TSModuleBlock | null;
  kind: TSModuleDeclarationKind;
  declare: boolean;
  global: false;
  parent: Node$1;
}
type TSModuleDeclarationKind = "module" | "namespace";
interface TSGlobalDeclaration extends Span {
  type: "TSModuleDeclaration";
  id: IdentifierName;
  body: TSModuleBlock;
  kind: "global";
  declare: boolean;
  global: true;
  parent: Node$1;
}
interface TSModuleBlock extends Span {
  type: "TSModuleBlock";
  body: Array<Directive | Statement>;
  parent: Node$1;
}
interface TSTypeLiteral extends Span {
  type: "TSTypeLiteral";
  members: Array<TSSignature>;
  parent: Node$1;
}
interface TSInferType extends Span {
  type: "TSInferType";
  typeParameter: TSTypeParameter;
  parent: Node$1;
}
interface TSTypeQuery extends Span {
  type: "TSTypeQuery";
  exprName: TSTypeQueryExprName;
  typeArguments: TSTypeParameterInstantiation | null;
  parent: Node$1;
}
type TSTypeQueryExprName = TSImportType | TSTypeName;
interface TSImportType extends Span {
  type: "TSImportType";
  source: StringLiteral;
  options: ObjectExpression | null;
  qualifier: TSImportTypeQualifier | null;
  typeArguments: TSTypeParameterInstantiation | null;
  parent: Node$1;
}
type TSImportTypeQualifier = IdentifierName | TSImportTypeQualifiedName;
interface TSImportTypeQualifiedName extends Span {
  type: "TSQualifiedName";
  left: TSImportTypeQualifier;
  right: IdentifierName;
  parent: Node$1;
}
interface TSFunctionType extends Span {
  type: "TSFunctionType";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  parent: Node$1;
}
interface TSConstructorType extends Span {
  type: "TSConstructorType";
  abstract: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  parent: Node$1;
}
interface TSMappedType extends Span {
  type: "TSMappedType";
  key: BindingIdentifier;
  constraint: TSType;
  nameType: TSType | null;
  typeAnnotation: TSType | null;
  optional: TSMappedTypeModifierOperator | false;
  readonly: TSMappedTypeModifierOperator | null;
  parent: Node$1;
}
type TSMappedTypeModifierOperator = true | "+" | "-";
interface TSTemplateLiteralType extends Span {
  type: "TSTemplateLiteralType";
  quasis: Array<TemplateElement>;
  types: Array<TSType>;
  parent: Node$1;
}
interface TSAsExpression extends Span {
  type: "TSAsExpression";
  expression: Expression;
  typeAnnotation: TSType;
  parent: Node$1;
}
interface TSSatisfiesExpression extends Span {
  type: "TSSatisfiesExpression";
  expression: Expression;
  typeAnnotation: TSType;
  parent: Node$1;
}
interface TSTypeAssertion extends Span {
  type: "TSTypeAssertion";
  typeAnnotation: TSType;
  expression: Expression;
  parent: Node$1;
}
interface TSImportEqualsDeclaration extends Span {
  type: "TSImportEqualsDeclaration";
  id: BindingIdentifier;
  moduleReference: TSModuleReference;
  importKind: ImportOrExportKind;
  parent: Node$1;
}
type TSModuleReference = TSExternalModuleReference | IdentifierReference | TSQualifiedName;
interface TSExternalModuleReference extends Span {
  type: "TSExternalModuleReference";
  expression: StringLiteral;
  parent: Node$1;
}
interface TSNonNullExpression extends Span {
  type: "TSNonNullExpression";
  expression: Expression;
  parent: Node$1;
}
interface Decorator extends Span {
  type: "Decorator";
  expression: Expression;
  parent: Node$1;
}
interface TSExportAssignment extends Span {
  type: "TSExportAssignment";
  expression: Expression;
  parent: Node$1;
}
interface TSNamespaceExportDeclaration extends Span {
  type: "TSNamespaceExportDeclaration";
  id: IdentifierName;
  parent: Node$1;
}
interface TSInstantiationExpression extends Span {
  type: "TSInstantiationExpression";
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation;
  parent: Node$1;
}
type ImportOrExportKind = "value" | "type";
interface JSDocNullableType extends Span {
  type: "TSJSDocNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
  parent: Node$1;
}
interface JSDocNonNullableType extends Span {
  type: "TSJSDocNonNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
  parent: Node$1;
}
interface JSDocUnknownType extends Span {
  type: "TSJSDocUnknownType";
  parent: Node$1;
}
type AssignmentOperator = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "<<=" | ">>=" | ">>>=" | "|=" | "^=" | "&=" | "||=" | "&&=" | "??=";
type BinaryOperator = "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | ">>>" | "|" | "^" | "&" | "in" | "instanceof";
type LogicalOperator = "||" | "&&" | "??";
type UnaryOperator = "+" | "-" | "!" | "~" | "typeof" | "void" | "delete";
type UpdateOperator = "++" | "--";
type ModuleKind = "script" | "module" | "commonjs";
type Node$1 = Program | IdentifierName | IdentifierReference | BindingIdentifier | LabelIdentifier | ThisExpression | ArrayExpression | ObjectExpression | ObjectProperty | TemplateLiteral | TaggedTemplateExpression | TemplateElement | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | CallExpression | NewExpression | MetaProperty | SpreadElement | UpdateExpression | UnaryExpression | BinaryExpression | PrivateInExpression | LogicalExpression | ConditionalExpression | AssignmentExpression | ArrayAssignmentTarget | ObjectAssignmentTarget | AssignmentTargetRest | AssignmentTargetWithDefault | AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty | SequenceExpression | Super | AwaitExpression | ChainExpression | ParenthesizedExpression | Directive | Hashbang | BlockStatement | VariableDeclaration | VariableDeclarator | EmptyStatement | ExpressionStatement | IfStatement | DoWhileStatement | WhileStatement | ForStatement | ForInStatement | ForOfStatement | ContinueStatement | BreakStatement | ReturnStatement | WithStatement | SwitchStatement | SwitchCase | LabeledStatement | ThrowStatement | TryStatement | CatchClause | DebuggerStatement | AssignmentPattern | ObjectPattern | BindingProperty | ArrayPattern | BindingRestElement | Function$1 | FunctionBody | ArrowFunctionExpression | YieldExpression | Class | ClassBody | MethodDefinition | PropertyDefinition | PrivateIdentifier | StaticBlock | AccessorProperty | ImportExpression | ImportDeclaration | ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier | ImportAttribute | ExportNamedDeclaration | ExportDefaultDeclaration | ExportAllDeclaration | ExportSpecifier | V8IntrinsicExpression | BooleanLiteral | NullLiteral | NumericLiteral | StringLiteral | BigIntLiteral | RegExpLiteral | JSXElement | JSXOpeningElement | JSXClosingElement | JSXFragment | JSXOpeningFragment | JSXClosingFragment | JSXNamespacedName | JSXMemberExpression | JSXExpressionContainer | JSXEmptyExpression | JSXAttribute | JSXSpreadAttribute | JSXIdentifier | JSXSpreadChild | JSXText | TSThisParameter | TSEnumDeclaration | TSEnumBody | TSEnumMember | TSTypeAnnotation | TSLiteralType | TSConditionalType | TSUnionType | TSIntersectionType | TSParenthesizedType | TSTypeOperator | TSArrayType | TSIndexedAccessType | TSTupleType | TSNamedTupleMember | TSOptionalType | TSRestType | TSAnyKeyword | TSStringKeyword | TSBooleanKeyword | TSNumberKeyword | TSNeverKeyword | TSIntrinsicKeyword | TSUnknownKeyword | TSNullKeyword | TSUndefinedKeyword | TSVoidKeyword | TSSymbolKeyword | TSThisType | TSObjectKeyword | TSBigIntKeyword | TSTypeReference | TSQualifiedName | TSTypeParameterInstantiation | TSTypeParameter | TSTypeParameterDeclaration | TSTypeAliasDeclaration | TSClassImplements | TSInterfaceDeclaration | TSInterfaceBody | TSPropertySignature | TSIndexSignature | TSCallSignatureDeclaration | TSMethodSignature | TSConstructSignatureDeclaration | TSIndexSignatureName | TSInterfaceHeritage | TSTypePredicate | TSModuleDeclaration | TSGlobalDeclaration | TSModuleBlock | TSTypeLiteral | TSInferType | TSTypeQuery | TSImportType | TSImportTypeQualifiedName | TSFunctionType | TSConstructorType | TSMappedType | TSTemplateLiteralType | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSImportEqualsDeclaration | TSExternalModuleReference | TSNonNullExpression | Decorator | TSExportAssignment | TSNamespaceExportDeclaration | TSInstantiationExpression | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType | ParamPattern;
//#endregion
//#region src-js/generated/visitor.d.ts
interface VisitorObject {
  DebuggerStatement?: (node: DebuggerStatement) => void;
  "DebuggerStatement:exit"?: (node: DebuggerStatement) => void;
  EmptyStatement?: (node: EmptyStatement) => void;
  "EmptyStatement:exit"?: (node: EmptyStatement) => void;
  Literal?: (node: BooleanLiteral | NullLiteral | NumericLiteral | StringLiteral | BigIntLiteral | RegExpLiteral) => void;
  "Literal:exit"?: (node: BooleanLiteral | NullLiteral | NumericLiteral | StringLiteral | BigIntLiteral | RegExpLiteral) => void;
  PrivateIdentifier?: (node: PrivateIdentifier) => void;
  "PrivateIdentifier:exit"?: (node: PrivateIdentifier) => void;
  Super?: (node: Super) => void;
  "Super:exit"?: (node: Super) => void;
  TemplateElement?: (node: TemplateElement) => void;
  "TemplateElement:exit"?: (node: TemplateElement) => void;
  ThisExpression?: (node: ThisExpression) => void;
  "ThisExpression:exit"?: (node: ThisExpression) => void;
  JSXClosingFragment?: (node: JSXClosingFragment) => void;
  "JSXClosingFragment:exit"?: (node: JSXClosingFragment) => void;
  JSXEmptyExpression?: (node: JSXEmptyExpression) => void;
  "JSXEmptyExpression:exit"?: (node: JSXEmptyExpression) => void;
  JSXIdentifier?: (node: JSXIdentifier) => void;
  "JSXIdentifier:exit"?: (node: JSXIdentifier) => void;
  JSXOpeningFragment?: (node: JSXOpeningFragment) => void;
  "JSXOpeningFragment:exit"?: (node: JSXOpeningFragment) => void;
  JSXText?: (node: JSXText) => void;
  "JSXText:exit"?: (node: JSXText) => void;
  TSAnyKeyword?: (node: TSAnyKeyword) => void;
  "TSAnyKeyword:exit"?: (node: TSAnyKeyword) => void;
  TSBigIntKeyword?: (node: TSBigIntKeyword) => void;
  "TSBigIntKeyword:exit"?: (node: TSBigIntKeyword) => void;
  TSBooleanKeyword?: (node: TSBooleanKeyword) => void;
  "TSBooleanKeyword:exit"?: (node: TSBooleanKeyword) => void;
  TSIntrinsicKeyword?: (node: TSIntrinsicKeyword) => void;
  "TSIntrinsicKeyword:exit"?: (node: TSIntrinsicKeyword) => void;
  TSJSDocUnknownType?: (node: JSDocUnknownType) => void;
  "TSJSDocUnknownType:exit"?: (node: JSDocUnknownType) => void;
  TSNeverKeyword?: (node: TSNeverKeyword) => void;
  "TSNeverKeyword:exit"?: (node: TSNeverKeyword) => void;
  TSNullKeyword?: (node: TSNullKeyword) => void;
  "TSNullKeyword:exit"?: (node: TSNullKeyword) => void;
  TSNumberKeyword?: (node: TSNumberKeyword) => void;
  "TSNumberKeyword:exit"?: (node: TSNumberKeyword) => void;
  TSObjectKeyword?: (node: TSObjectKeyword) => void;
  "TSObjectKeyword:exit"?: (node: TSObjectKeyword) => void;
  TSStringKeyword?: (node: TSStringKeyword) => void;
  "TSStringKeyword:exit"?: (node: TSStringKeyword) => void;
  TSSymbolKeyword?: (node: TSSymbolKeyword) => void;
  "TSSymbolKeyword:exit"?: (node: TSSymbolKeyword) => void;
  TSThisType?: (node: TSThisType) => void;
  "TSThisType:exit"?: (node: TSThisType) => void;
  TSUndefinedKeyword?: (node: TSUndefinedKeyword) => void;
  "TSUndefinedKeyword:exit"?: (node: TSUndefinedKeyword) => void;
  TSUnknownKeyword?: (node: TSUnknownKeyword) => void;
  "TSUnknownKeyword:exit"?: (node: TSUnknownKeyword) => void;
  TSVoidKeyword?: (node: TSVoidKeyword) => void;
  "TSVoidKeyword:exit"?: (node: TSVoidKeyword) => void;
  AccessorProperty?: (node: AccessorProperty) => void;
  "AccessorProperty:exit"?: (node: AccessorProperty) => void;
  ArrayExpression?: (node: ArrayExpression) => void;
  "ArrayExpression:exit"?: (node: ArrayExpression) => void;
  ArrayPattern?: (node: ArrayPattern) => void;
  "ArrayPattern:exit"?: (node: ArrayPattern) => void;
  ArrowFunctionExpression?: (node: ArrowFunctionExpression) => void;
  "ArrowFunctionExpression:exit"?: (node: ArrowFunctionExpression) => void;
  AssignmentExpression?: (node: AssignmentExpression) => void;
  "AssignmentExpression:exit"?: (node: AssignmentExpression) => void;
  AssignmentPattern?: (node: AssignmentPattern) => void;
  "AssignmentPattern:exit"?: (node: AssignmentPattern) => void;
  AwaitExpression?: (node: AwaitExpression) => void;
  "AwaitExpression:exit"?: (node: AwaitExpression) => void;
  BinaryExpression?: (node: BinaryExpression) => void;
  "BinaryExpression:exit"?: (node: BinaryExpression) => void;
  BlockStatement?: (node: BlockStatement) => void;
  "BlockStatement:exit"?: (node: BlockStatement) => void;
  BreakStatement?: (node: BreakStatement) => void;
  "BreakStatement:exit"?: (node: BreakStatement) => void;
  CallExpression?: (node: CallExpression) => void;
  "CallExpression:exit"?: (node: CallExpression) => void;
  CatchClause?: (node: CatchClause) => void;
  "CatchClause:exit"?: (node: CatchClause) => void;
  ChainExpression?: (node: ChainExpression) => void;
  "ChainExpression:exit"?: (node: ChainExpression) => void;
  ClassBody?: (node: ClassBody) => void;
  "ClassBody:exit"?: (node: ClassBody) => void;
  ClassDeclaration?: (node: Class) => void;
  "ClassDeclaration:exit"?: (node: Class) => void;
  ClassExpression?: (node: Class) => void;
  "ClassExpression:exit"?: (node: Class) => void;
  ConditionalExpression?: (node: ConditionalExpression) => void;
  "ConditionalExpression:exit"?: (node: ConditionalExpression) => void;
  ContinueStatement?: (node: ContinueStatement) => void;
  "ContinueStatement:exit"?: (node: ContinueStatement) => void;
  Decorator?: (node: Decorator) => void;
  "Decorator:exit"?: (node: Decorator) => void;
  DoWhileStatement?: (node: DoWhileStatement) => void;
  "DoWhileStatement:exit"?: (node: DoWhileStatement) => void;
  ExportAllDeclaration?: (node: ExportAllDeclaration) => void;
  "ExportAllDeclaration:exit"?: (node: ExportAllDeclaration) => void;
  ExportDefaultDeclaration?: (node: ExportDefaultDeclaration) => void;
  "ExportDefaultDeclaration:exit"?: (node: ExportDefaultDeclaration) => void;
  ExportNamedDeclaration?: (node: ExportNamedDeclaration) => void;
  "ExportNamedDeclaration:exit"?: (node: ExportNamedDeclaration) => void;
  ExportSpecifier?: (node: ExportSpecifier) => void;
  "ExportSpecifier:exit"?: (node: ExportSpecifier) => void;
  ExpressionStatement?: (node: ExpressionStatement) => void;
  "ExpressionStatement:exit"?: (node: ExpressionStatement) => void;
  ForInStatement?: (node: ForInStatement) => void;
  "ForInStatement:exit"?: (node: ForInStatement) => void;
  ForOfStatement?: (node: ForOfStatement) => void;
  "ForOfStatement:exit"?: (node: ForOfStatement) => void;
  ForStatement?: (node: ForStatement) => void;
  "ForStatement:exit"?: (node: ForStatement) => void;
  FunctionDeclaration?: (node: Function$1) => void;
  "FunctionDeclaration:exit"?: (node: Function$1) => void;
  FunctionExpression?: (node: Function$1) => void;
  "FunctionExpression:exit"?: (node: Function$1) => void;
  Identifier?: (node: IdentifierName | IdentifierReference | BindingIdentifier | LabelIdentifier | TSThisParameter | TSIndexSignatureName) => void;
  "Identifier:exit"?: (node: IdentifierName | IdentifierReference | BindingIdentifier | LabelIdentifier | TSThisParameter | TSIndexSignatureName) => void;
  IfStatement?: (node: IfStatement) => void;
  "IfStatement:exit"?: (node: IfStatement) => void;
  ImportAttribute?: (node: ImportAttribute) => void;
  "ImportAttribute:exit"?: (node: ImportAttribute) => void;
  ImportDeclaration?: (node: ImportDeclaration) => void;
  "ImportDeclaration:exit"?: (node: ImportDeclaration) => void;
  ImportDefaultSpecifier?: (node: ImportDefaultSpecifier) => void;
  "ImportDefaultSpecifier:exit"?: (node: ImportDefaultSpecifier) => void;
  ImportExpression?: (node: ImportExpression) => void;
  "ImportExpression:exit"?: (node: ImportExpression) => void;
  ImportNamespaceSpecifier?: (node: ImportNamespaceSpecifier) => void;
  "ImportNamespaceSpecifier:exit"?: (node: ImportNamespaceSpecifier) => void;
  ImportSpecifier?: (node: ImportSpecifier) => void;
  "ImportSpecifier:exit"?: (node: ImportSpecifier) => void;
  LabeledStatement?: (node: LabeledStatement) => void;
  "LabeledStatement:exit"?: (node: LabeledStatement) => void;
  LogicalExpression?: (node: LogicalExpression) => void;
  "LogicalExpression:exit"?: (node: LogicalExpression) => void;
  MemberExpression?: (node: MemberExpression) => void;
  "MemberExpression:exit"?: (node: MemberExpression) => void;
  MetaProperty?: (node: MetaProperty) => void;
  "MetaProperty:exit"?: (node: MetaProperty) => void;
  MethodDefinition?: (node: MethodDefinition) => void;
  "MethodDefinition:exit"?: (node: MethodDefinition) => void;
  NewExpression?: (node: NewExpression) => void;
  "NewExpression:exit"?: (node: NewExpression) => void;
  ObjectExpression?: (node: ObjectExpression) => void;
  "ObjectExpression:exit"?: (node: ObjectExpression) => void;
  ObjectPattern?: (node: ObjectPattern) => void;
  "ObjectPattern:exit"?: (node: ObjectPattern) => void;
  ParenthesizedExpression?: (node: ParenthesizedExpression) => void;
  "ParenthesizedExpression:exit"?: (node: ParenthesizedExpression) => void;
  Program?: (node: Program) => void;
  "Program:exit"?: (node: Program) => void;
  Property?: (node: ObjectProperty | AssignmentTargetProperty | AssignmentTargetPropertyProperty | BindingProperty) => void;
  "Property:exit"?: (node: ObjectProperty | AssignmentTargetProperty | AssignmentTargetPropertyProperty | BindingProperty) => void;
  PropertyDefinition?: (node: PropertyDefinition) => void;
  "PropertyDefinition:exit"?: (node: PropertyDefinition) => void;
  RestElement?: (node: AssignmentTargetRest | BindingRestElement | FormalParameterRest) => void;
  "RestElement:exit"?: (node: AssignmentTargetRest | BindingRestElement | FormalParameterRest) => void;
  ReturnStatement?: (node: ReturnStatement) => void;
  "ReturnStatement:exit"?: (node: ReturnStatement) => void;
  SequenceExpression?: (node: SequenceExpression) => void;
  "SequenceExpression:exit"?: (node: SequenceExpression) => void;
  SpreadElement?: (node: SpreadElement) => void;
  "SpreadElement:exit"?: (node: SpreadElement) => void;
  StaticBlock?: (node: StaticBlock) => void;
  "StaticBlock:exit"?: (node: StaticBlock) => void;
  SwitchCase?: (node: SwitchCase) => void;
  "SwitchCase:exit"?: (node: SwitchCase) => void;
  SwitchStatement?: (node: SwitchStatement) => void;
  "SwitchStatement:exit"?: (node: SwitchStatement) => void;
  TaggedTemplateExpression?: (node: TaggedTemplateExpression) => void;
  "TaggedTemplateExpression:exit"?: (node: TaggedTemplateExpression) => void;
  TemplateLiteral?: (node: TemplateLiteral) => void;
  "TemplateLiteral:exit"?: (node: TemplateLiteral) => void;
  ThrowStatement?: (node: ThrowStatement) => void;
  "ThrowStatement:exit"?: (node: ThrowStatement) => void;
  TryStatement?: (node: TryStatement) => void;
  "TryStatement:exit"?: (node: TryStatement) => void;
  UnaryExpression?: (node: UnaryExpression) => void;
  "UnaryExpression:exit"?: (node: UnaryExpression) => void;
  UpdateExpression?: (node: UpdateExpression) => void;
  "UpdateExpression:exit"?: (node: UpdateExpression) => void;
  V8IntrinsicExpression?: (node: V8IntrinsicExpression) => void;
  "V8IntrinsicExpression:exit"?: (node: V8IntrinsicExpression) => void;
  VariableDeclaration?: (node: VariableDeclaration) => void;
  "VariableDeclaration:exit"?: (node: VariableDeclaration) => void;
  VariableDeclarator?: (node: VariableDeclarator) => void;
  "VariableDeclarator:exit"?: (node: VariableDeclarator) => void;
  WhileStatement?: (node: WhileStatement) => void;
  "WhileStatement:exit"?: (node: WhileStatement) => void;
  WithStatement?: (node: WithStatement) => void;
  "WithStatement:exit"?: (node: WithStatement) => void;
  YieldExpression?: (node: YieldExpression) => void;
  "YieldExpression:exit"?: (node: YieldExpression) => void;
  JSXAttribute?: (node: JSXAttribute) => void;
  "JSXAttribute:exit"?: (node: JSXAttribute) => void;
  JSXClosingElement?: (node: JSXClosingElement) => void;
  "JSXClosingElement:exit"?: (node: JSXClosingElement) => void;
  JSXElement?: (node: JSXElement) => void;
  "JSXElement:exit"?: (node: JSXElement) => void;
  JSXExpressionContainer?: (node: JSXExpressionContainer) => void;
  "JSXExpressionContainer:exit"?: (node: JSXExpressionContainer) => void;
  JSXFragment?: (node: JSXFragment) => void;
  "JSXFragment:exit"?: (node: JSXFragment) => void;
  JSXMemberExpression?: (node: JSXMemberExpression) => void;
  "JSXMemberExpression:exit"?: (node: JSXMemberExpression) => void;
  JSXNamespacedName?: (node: JSXNamespacedName) => void;
  "JSXNamespacedName:exit"?: (node: JSXNamespacedName) => void;
  JSXOpeningElement?: (node: JSXOpeningElement) => void;
  "JSXOpeningElement:exit"?: (node: JSXOpeningElement) => void;
  JSXSpreadAttribute?: (node: JSXSpreadAttribute) => void;
  "JSXSpreadAttribute:exit"?: (node: JSXSpreadAttribute) => void;
  JSXSpreadChild?: (node: JSXSpreadChild) => void;
  "JSXSpreadChild:exit"?: (node: JSXSpreadChild) => void;
  TSAbstractAccessorProperty?: (node: AccessorProperty) => void;
  "TSAbstractAccessorProperty:exit"?: (node: AccessorProperty) => void;
  TSAbstractMethodDefinition?: (node: MethodDefinition) => void;
  "TSAbstractMethodDefinition:exit"?: (node: MethodDefinition) => void;
  TSAbstractPropertyDefinition?: (node: PropertyDefinition) => void;
  "TSAbstractPropertyDefinition:exit"?: (node: PropertyDefinition) => void;
  TSArrayType?: (node: TSArrayType) => void;
  "TSArrayType:exit"?: (node: TSArrayType) => void;
  TSAsExpression?: (node: TSAsExpression) => void;
  "TSAsExpression:exit"?: (node: TSAsExpression) => void;
  TSCallSignatureDeclaration?: (node: TSCallSignatureDeclaration) => void;
  "TSCallSignatureDeclaration:exit"?: (node: TSCallSignatureDeclaration) => void;
  TSClassImplements?: (node: TSClassImplements) => void;
  "TSClassImplements:exit"?: (node: TSClassImplements) => void;
  TSConditionalType?: (node: TSConditionalType) => void;
  "TSConditionalType:exit"?: (node: TSConditionalType) => void;
  TSConstructSignatureDeclaration?: (node: TSConstructSignatureDeclaration) => void;
  "TSConstructSignatureDeclaration:exit"?: (node: TSConstructSignatureDeclaration) => void;
  TSConstructorType?: (node: TSConstructorType) => void;
  "TSConstructorType:exit"?: (node: TSConstructorType) => void;
  TSDeclareFunction?: (node: Function$1) => void;
  "TSDeclareFunction:exit"?: (node: Function$1) => void;
  TSEmptyBodyFunctionExpression?: (node: Function$1) => void;
  "TSEmptyBodyFunctionExpression:exit"?: (node: Function$1) => void;
  TSEnumBody?: (node: TSEnumBody) => void;
  "TSEnumBody:exit"?: (node: TSEnumBody) => void;
  TSEnumDeclaration?: (node: TSEnumDeclaration) => void;
  "TSEnumDeclaration:exit"?: (node: TSEnumDeclaration) => void;
  TSEnumMember?: (node: TSEnumMember) => void;
  "TSEnumMember:exit"?: (node: TSEnumMember) => void;
  TSExportAssignment?: (node: TSExportAssignment) => void;
  "TSExportAssignment:exit"?: (node: TSExportAssignment) => void;
  TSExternalModuleReference?: (node: TSExternalModuleReference) => void;
  "TSExternalModuleReference:exit"?: (node: TSExternalModuleReference) => void;
  TSFunctionType?: (node: TSFunctionType) => void;
  "TSFunctionType:exit"?: (node: TSFunctionType) => void;
  TSImportEqualsDeclaration?: (node: TSImportEqualsDeclaration) => void;
  "TSImportEqualsDeclaration:exit"?: (node: TSImportEqualsDeclaration) => void;
  TSImportType?: (node: TSImportType) => void;
  "TSImportType:exit"?: (node: TSImportType) => void;
  TSIndexSignature?: (node: TSIndexSignature) => void;
  "TSIndexSignature:exit"?: (node: TSIndexSignature) => void;
  TSIndexedAccessType?: (node: TSIndexedAccessType) => void;
  "TSIndexedAccessType:exit"?: (node: TSIndexedAccessType) => void;
  TSInferType?: (node: TSInferType) => void;
  "TSInferType:exit"?: (node: TSInferType) => void;
  TSInstantiationExpression?: (node: TSInstantiationExpression) => void;
  "TSInstantiationExpression:exit"?: (node: TSInstantiationExpression) => void;
  TSInterfaceBody?: (node: TSInterfaceBody) => void;
  "TSInterfaceBody:exit"?: (node: TSInterfaceBody) => void;
  TSInterfaceDeclaration?: (node: TSInterfaceDeclaration) => void;
  "TSInterfaceDeclaration:exit"?: (node: TSInterfaceDeclaration) => void;
  TSInterfaceHeritage?: (node: TSInterfaceHeritage) => void;
  "TSInterfaceHeritage:exit"?: (node: TSInterfaceHeritage) => void;
  TSIntersectionType?: (node: TSIntersectionType) => void;
  "TSIntersectionType:exit"?: (node: TSIntersectionType) => void;
  TSJSDocNonNullableType?: (node: JSDocNonNullableType) => void;
  "TSJSDocNonNullableType:exit"?: (node: JSDocNonNullableType) => void;
  TSJSDocNullableType?: (node: JSDocNullableType) => void;
  "TSJSDocNullableType:exit"?: (node: JSDocNullableType) => void;
  TSLiteralType?: (node: TSLiteralType) => void;
  "TSLiteralType:exit"?: (node: TSLiteralType) => void;
  TSMappedType?: (node: TSMappedType) => void;
  "TSMappedType:exit"?: (node: TSMappedType) => void;
  TSMethodSignature?: (node: TSMethodSignature) => void;
  "TSMethodSignature:exit"?: (node: TSMethodSignature) => void;
  TSModuleBlock?: (node: TSModuleBlock) => void;
  "TSModuleBlock:exit"?: (node: TSModuleBlock) => void;
  TSModuleDeclaration?: (node: TSModuleDeclaration | TSGlobalDeclaration) => void;
  "TSModuleDeclaration:exit"?: (node: TSModuleDeclaration | TSGlobalDeclaration) => void;
  TSNamedTupleMember?: (node: TSNamedTupleMember) => void;
  "TSNamedTupleMember:exit"?: (node: TSNamedTupleMember) => void;
  TSNamespaceExportDeclaration?: (node: TSNamespaceExportDeclaration) => void;
  "TSNamespaceExportDeclaration:exit"?: (node: TSNamespaceExportDeclaration) => void;
  TSNonNullExpression?: (node: TSNonNullExpression) => void;
  "TSNonNullExpression:exit"?: (node: TSNonNullExpression) => void;
  TSOptionalType?: (node: TSOptionalType) => void;
  "TSOptionalType:exit"?: (node: TSOptionalType) => void;
  TSParameterProperty?: (node: TSParameterProperty) => void;
  "TSParameterProperty:exit"?: (node: TSParameterProperty) => void;
  TSParenthesizedType?: (node: TSParenthesizedType) => void;
  "TSParenthesizedType:exit"?: (node: TSParenthesizedType) => void;
  TSPropertySignature?: (node: TSPropertySignature) => void;
  "TSPropertySignature:exit"?: (node: TSPropertySignature) => void;
  TSQualifiedName?: (node: TSQualifiedName) => void;
  "TSQualifiedName:exit"?: (node: TSQualifiedName) => void;
  TSRestType?: (node: TSRestType) => void;
  "TSRestType:exit"?: (node: TSRestType) => void;
  TSSatisfiesExpression?: (node: TSSatisfiesExpression) => void;
  "TSSatisfiesExpression:exit"?: (node: TSSatisfiesExpression) => void;
  TSTemplateLiteralType?: (node: TSTemplateLiteralType) => void;
  "TSTemplateLiteralType:exit"?: (node: TSTemplateLiteralType) => void;
  TSTupleType?: (node: TSTupleType) => void;
  "TSTupleType:exit"?: (node: TSTupleType) => void;
  TSTypeAliasDeclaration?: (node: TSTypeAliasDeclaration) => void;
  "TSTypeAliasDeclaration:exit"?: (node: TSTypeAliasDeclaration) => void;
  TSTypeAnnotation?: (node: TSTypeAnnotation) => void;
  "TSTypeAnnotation:exit"?: (node: TSTypeAnnotation) => void;
  TSTypeAssertion?: (node: TSTypeAssertion) => void;
  "TSTypeAssertion:exit"?: (node: TSTypeAssertion) => void;
  TSTypeLiteral?: (node: TSTypeLiteral) => void;
  "TSTypeLiteral:exit"?: (node: TSTypeLiteral) => void;
  TSTypeOperator?: (node: TSTypeOperator) => void;
  "TSTypeOperator:exit"?: (node: TSTypeOperator) => void;
  TSTypeParameter?: (node: TSTypeParameter) => void;
  "TSTypeParameter:exit"?: (node: TSTypeParameter) => void;
  TSTypeParameterDeclaration?: (node: TSTypeParameterDeclaration) => void;
  "TSTypeParameterDeclaration:exit"?: (node: TSTypeParameterDeclaration) => void;
  TSTypeParameterInstantiation?: (node: TSTypeParameterInstantiation) => void;
  "TSTypeParameterInstantiation:exit"?: (node: TSTypeParameterInstantiation) => void;
  TSTypePredicate?: (node: TSTypePredicate) => void;
  "TSTypePredicate:exit"?: (node: TSTypePredicate) => void;
  TSTypeQuery?: (node: TSTypeQuery) => void;
  "TSTypeQuery:exit"?: (node: TSTypeQuery) => void;
  TSTypeReference?: (node: TSTypeReference) => void;
  "TSTypeReference:exit"?: (node: TSTypeReference) => void;
  TSUnionType?: (node: TSUnionType) => void;
  "TSUnionType:exit"?: (node: TSUnionType) => void;
  [key: string]: (node: Node$1) => void;
}
//#endregion
//#region src-js/plugins/types.d.ts
type BeforeHook = () => boolean | void;
type AfterHook = () => void;
type VisitorWithHooks = VisitorObject & {
  before?: BeforeHook;
  after?: AfterHook;
};
interface Node extends Span {}
type NodeOrToken = Node | Token | Comment;
interface Comment extends Span {
  type: "Line" | "Block" | "Shebang";
  value: string;
}
//#endregion
//#region src-js/plugins/location.d.ts
/**
 * Range of source offsets.
 */
type Range = [number, number];
/**
 * Interface for any type which has `range` field.
 */
interface Ranged {
  range: Range;
}
/**
 * Interface for any type which has location properties.
 */
interface Span extends Ranged {
  start: number;
  end: number;
  loc: Location;
}
/**
 * Source code location.
 */
interface Location {
  start: LineColumn;
  end: LineColumn;
}
/**
 * Line number + column number pair.
 * `line` is 1-indexed, `column` is 0-indexed.
 */
interface LineColumn {
  line: number;
  column: number;
}
/**
 * Convert a source text index into a (line, column) pair.
 * @param offset - The index of a character in a file.
 * @returns `{line, column}` location object with 1-indexed line and 0-indexed column.
 * @throws {TypeError|RangeError} If non-numeric `offset`, or `offset` out of range.
 */
declare function getLineColumnFromOffset(offset: number): LineColumn;
/**
 * Convert a `{ line, column }` pair into a range index.
 * @param loc - A line/column location.
 * @returns The character index of the location in the file.
 * @throws {TypeError|RangeError} If `loc` is not an object with a numeric `line` and `column`,
 *   or if the `line` is less than or equal to zero, or the line or column is out of the expected range.
 */
declare function getOffsetFromLineColumn(loc: LineColumn): number;
/**
 * Get the range of the given node or token.
 * @param nodeOrToken - Node or token to get the range of
 * @returns Range of the node or token
 */
declare function getRange(nodeOrToken: NodeOrToken): Range;
/**
 * Get the location of the given node or token.
 * @param nodeOrToken - Node or token to get the location of
 * @returns Location of the node or token
 */
declare function getLoc(nodeOrToken: NodeOrToken): Location;
/**
 * Get the deepest node containing a range index.
 * @param offset - Range index of the desired node
 * @returns The node if found, or `null` if not found
 */
declare function getNodeByRangeIndex(offset: number): Node$1 | null;
//#endregion
//#region src-js/plugins/fix.d.ts
type FixFn = (fixer: Fixer) => Fix | Array<Fix | null | undefined> | IterableIterator<Fix | null | undefined> | null | undefined;
type Fix = {
  range: Range;
  text: string;
};
declare const FIXER: Readonly<{
  insertTextBefore(nodeOrToken: Ranged, text: string): Fix;
  insertTextBeforeRange(range: Range, text: string): Fix;
  insertTextAfter(nodeOrToken: Ranged, text: string): Fix;
  insertTextAfterRange(range: Range, text: string): Fix;
  remove(nodeOrToken: Ranged): Fix;
  removeRange(range: Range): Fix;
  replaceText(nodeOrToken: Ranged, text: string): Fix;
  replaceTextRange(range: Range, text: string): Fix;
}>;
type Fixer = typeof FIXER;
//#endregion
//#region src-js/plugins/report.d.ts
/**
 * Diagnostic object.
 * Passed to `Context#report()`.
 *
 * - Either `message` or `messageId` property must be provided.
 * - Either `node` or `loc` property must be provided.
 */
type Diagnostic = RequireAtLeastOne<RequireAtLeastOne<DiagnosticBase, "node" | "loc">, "message" | "messageId">;
interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  node?: Ranged;
  loc?: LocationWithOptionalEnd | LineColumn;
  data?: DiagnosticData | null | undefined;
  fix?: FixFn;
  suggest?: Suggestion[];
}
/**
 * Location with `end` property optional.
 */
interface LocationWithOptionalEnd {
  start: LineColumn;
  end?: LineColumn | null | undefined;
}
/**
 * Data to interpolate into a diagnostic message.
 */
type DiagnosticData = Record<string, string | number | boolean | bigint | null | undefined>;
/**
 * Suggested fix.
 * NOT IMPLEMENTED YET.
 */
type Suggestion = RequireAtLeastOne<SuggestionBase, "desc" | "messageId">;
interface SuggestionBase {
  desc?: string;
  messageId?: string;
  fix: FixFn;
  data?: DiagnosticData | null | undefined;
}
//#endregion
//#region src-js/plugins/settings.d.ts
/**
 * Settings for the file being linted.
 *
 * Settings are deserialized from JSON, so can only contain JSON-compatible values.
 */
type Settings = JsonObject;
//#endregion
//#region src-js/plugins/comments.d.ts
/**
 * Retrieve an array containing all comments in the source code.
 * @returns Array of `Comment`s in order they appear in source.
 */
declare function getAllComments(): Comment[];
/**
 * Get all comments directly before the given node or token.
 *
 * "Directly before" means only comments before this node, and after the preceding token.
 *
 * ```js
 * // Define `x`
 * const x = 1;
 * // Define `y`
 * const y = 2;
 * ```
 *
 * `sourceCode.getCommentsBefore(varDeclY)` will only return "Define `y`" comment, not also "Define `x`".
 *
 * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
 * @returns Array of `Comment`s in occurrence order.
 */
declare function getCommentsBefore(nodeOrToken: NodeOrToken): Comment[];
/**
 * Get all comment tokens directly after the given node or token.
 *
 * "Directly after" means only comments between end of this node, and the next token following it.
 *
 * ```js
 * const x = 1;
 * // Define `y`
 * const y = 2;
 * // Define `z`
 * const z = 3;
 * ```
 *
 * `sourceCode.getCommentsAfter(varDeclX)` will only return "Define `y`" comment, not also "Define `z`".
 *
 * @param nodeOrToken - The AST node or token to check for adjacent comment tokens.
 * @returns Array of `Comment`s in occurrence order.
 */
declare function getCommentsAfter(nodeOrToken: NodeOrToken): Comment[];
/**
 * Get all comment tokens inside the given node.
 * @param node - The AST node to get the comments for.
 * @returns Array of `Comment`s in occurrence order.
 */
declare function getCommentsInside(node: Node): Comment[];
/**
 * Check whether any comments exist or not between the given 2 nodes.
 * @param nodeOrToken1 - Start node/token.
 * @param nodeOrToken2 - End node/token.
 * @returns `true` if one or more comments exist between the two.
 */
declare function commentsExistBetween(nodeOrToken1: NodeOrToken, nodeOrToken2: NodeOrToken): boolean;
/**
 * Retrieve the JSDoc comment for a given node.
 *
 * @deprecated
 *
 * @param node - The AST node to get the comment for.
 * @returns The JSDoc comment for the given node, or `null` if not found.
 */
declare function getJSDocComment(node: Node): Comment | null;
//#endregion
//#region src-js/plugins/scope.d.ts
interface Scope {
  type: ScopeType;
  isStrict: boolean;
  upper: Scope | null;
  childScopes: Scope[];
  variableScope: Scope;
  block: Node$1;
  variables: Variable[];
  set: Map<string, Variable>;
  references: Reference[];
  through: Reference[];
  functionExpressionScope: boolean;
  implicit?: {
    variables: Variable[];
    set: Map<string, Variable>;
  };
}
type ScopeType = "block" | "catch" | "class" | "class-field-initializer" | "class-static-block" | "for" | "function" | "function-expression-name" | "global" | "module" | "switch" | "with";
interface Variable {
  name: string;
  scope: Scope;
  identifiers: Identifier[];
  references: Reference[];
  defs: Definition[];
}
interface Reference {
  identifier: Identifier;
  from: Scope;
  resolved: Variable | null;
  writeExpr: Expression | null;
  init: boolean;
  isWrite(): boolean;
  isRead(): boolean;
  isReadOnly(): boolean;
  isWriteOnly(): boolean;
  isReadWrite(): boolean;
}
interface Definition {
  type: DefinitionType;
  name: Identifier;
  node: Node$1;
  parent: Node$1 | null;
}
type DefinitionType = "CatchClause" | "ClassName" | "FunctionName" | "ImplicitGlobalVariable" | "ImportBinding" | "Parameter" | "Variable";
type Identifier = IdentifierName | IdentifierReference | BindingIdentifier | LabelIdentifier | TSThisParameter | TSIndexSignatureName;
/**
 * Discard TS-ESLint `ScopeManager`, to free memory.
 */
/**
 * @see https://eslint.org/docs/latest/developer-guide/scope-manager-interface#scopemanager-interface
 */
declare const SCOPE_MANAGER: Readonly<{
  /**
   * All scopes.
   */
  readonly scopes: Scope[];
  /**
   * The root scope.
   */
  readonly globalScope: Scope | null;
  /**
   * Get the variables that a given AST node defines.
   * The returned variables' `def[].node` / `def[].parent` property is the node.
   * If the node does not define any variable, this returns an empty array.
   * @param node AST node to get variables of.
   */
  getDeclaredVariables(node: Node$1): Variable[];
  /**
   * Get the scope of a given AST node. The returned scope's `block` property is the node.
   * This method never returns `function-expression-name` scope.
   * If the node does not have a scope, returns `null`.
   *
   * @param node An AST node to get their scope.
   * @param inner If the node has multiple scopes, this returns the outermost scope normally.
   *   If `inner` is `true` then this returns the innermost scope.
   */
  acquire(node: Node$1, inner?: boolean): Scope | null;
}>;
type ScopeManager = typeof SCOPE_MANAGER;
/**
 * Determine whether the given identifier node is a reference to a global variable.
 * @param node - `Identifier` node to check.
 * @returns `true` if the identifier is a reference to a global variable.
 */
declare function isGlobalReference(node: Node$1): boolean;
/**
 * Get the variables that `node` defines.
 * This is a convenience method that passes through to the same method on the `ScopeManager`.
 * @param node - The node for which the variables are obtained.
 * @returns An array of variable nodes representing the variables that `node` defines.
 */
declare function getDeclaredVariables(node: Node$1): Variable[];
/**
 * Get the scope for the given node.
 * @param node - The node to get the scope of.
 * @returns The scope information for this node.
 */
declare function getScope(node: Node$1): Scope;
/**
 * Marks as used a variable with the given name in a scope indicated by the given reference node.
 * This affects the `no-unused-vars` rule.
 * @param name - Variable name
 * @param refNode - Reference node
 * @returns `true` if a variable with the given name was found and marked as used, otherwise `false`
 */
declare function markVariableAsUsed(name: string, refNode: Node$1): boolean;
//#endregion
//#region src-js/plugins/source_code.d.ts
declare const SOURCE_CODE: Readonly<{
  /**
   * Source text.
   */
  readonly text: string;
  /**
   * `true` if file has Unicode BOM.
   */
  readonly hasBOM: boolean;
  /**
   * AST of the file.
   */
  readonly ast: Program;
  /**
   * `true` if the AST is in ESTree format.
   */
  isESTree: true;
  /**
   * `ScopeManager` for the file.
   */
  readonly scopeManager: ScopeManager;
  /**
   * Visitor keys to traverse this AST.
   */
  readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
  /**
   * Parser services for the file.
   *
   * Oxlint does not offer any parser services.
   */
  parserServices: Readonly<Record<string, unknown>>;
  /**
   * Source text as array of lines, split according to specification's definition of line breaks.
   */
  readonly lines: string[];
  /**
   * Character offset of the first character of each line in source text,
   * split according to specification's definition of line breaks.
   */
  readonly lineStartIndices: number[];
  /**
   * Array of all tokens and comments in the file, in source order.
   */
  readonly tokensAndComments: (Token | Comment)[];
  /**
   * Get the source code for the given node.
   * @param node? - The AST node to get the text for.
   * @param beforeCount? - The number of characters before the node to retrieve.
   * @param afterCount? - The number of characters after the node to retrieve.
   * @returns Source text representing the AST node.
   */
  getText(node?: Ranged | null, beforeCount?: number | null, afterCount?: number | null): string;
  /**
   * Get all the ancestors of a given node.
   * @param node - AST node
   * @returns All the ancestor nodes in the AST, not including the provided node,
   *   starting from the root node at index 0 and going inwards to the parent node.
   */
  getAncestors(node: Node): Node[];
  /**
   * Get source text as array of lines, split according to specification's definition of line breaks.
   */
  getLines(): string[];
  getRange: typeof getRange;
  getLoc: typeof getLoc;
  getNodeByRangeIndex: typeof getNodeByRangeIndex;
  getLocFromIndex: typeof getLineColumnFromOffset;
  getIndexFromLoc: typeof getOffsetFromLineColumn;
  getAllComments: typeof getAllComments;
  getCommentsBefore: typeof getCommentsBefore;
  getCommentsAfter: typeof getCommentsAfter;
  getCommentsInside: typeof getCommentsInside;
  commentsExistBetween: typeof commentsExistBetween;
  getJSDocComment: typeof getJSDocComment;
  isGlobalReference: typeof isGlobalReference;
  getDeclaredVariables: typeof getDeclaredVariables;
  getScope: typeof getScope;
  markVariableAsUsed: typeof markVariableAsUsed;
  getTokens: typeof getTokens;
  getFirstToken: typeof getFirstToken;
  getFirstTokens: typeof getFirstTokens;
  getLastToken: typeof getLastToken;
  getLastTokens: typeof getLastTokens;
  getTokenBefore: typeof getTokenBefore;
  getTokenOrCommentBefore: typeof getTokenOrCommentBefore;
  getTokensBefore: typeof getTokensBefore;
  getTokenAfter: typeof getTokenAfter;
  getTokenOrCommentAfter: typeof getTokenOrCommentAfter;
  getTokensAfter: typeof getTokensAfter;
  getTokensBetween: typeof getTokensBetween;
  getFirstTokenBetween: typeof getFirstTokenBetween;
  getFirstTokensBetween: typeof getFirstTokensBetween;
  getLastTokenBetween: typeof getLastTokenBetween;
  getLastTokensBetween: typeof getLastTokensBetween;
  getTokenByRangeStart: typeof getTokenByRangeStart;
  isSpaceBetween: typeof isSpaceBetween;
  isSpaceBetweenTokens: typeof isSpaceBetweenTokens;
}>;
type SourceCode = typeof SOURCE_CODE;
//#endregion
//#region src-js/plugins/context.d.ts
declare const LANGUAGE_OPTIONS: {
  /**
   * Source type of the file being linted.
   */
  readonly sourceType: ModuleKind;
  /**
   * ECMAScript version of the file being linted.
   */
  ecmaVersion: number;
  /**
   * Parser used to parse the file being linted.
   */
  parser: Readonly<{
    /**
     * Parser name.
     */
    name: "oxc";
    /**
     * Parser version.
     */
    version: "0.0.0";
    /**
     * Parse code into an AST.
     * @param code - Code to parse
     * @param options? - Parser options
     * @returns AST
     */
    parse(code: string, options?: Record<string, unknown>): Program;
    /**
     * Visitor keys for AST nodes.
     */
    VisitorKeys: Readonly<{
      DebuggerStatement: readonly never[];
      EmptyStatement: readonly never[];
      Literal: readonly never[];
      PrivateIdentifier: readonly never[];
      Super: readonly never[];
      TemplateElement: readonly never[];
      ThisExpression: readonly never[];
      JSXClosingFragment: readonly never[];
      JSXEmptyExpression: readonly never[];
      JSXIdentifier: readonly never[];
      JSXOpeningFragment: readonly never[];
      JSXText: readonly never[];
      TSAnyKeyword: readonly never[];
      TSBigIntKeyword: readonly never[];
      TSBooleanKeyword: readonly never[];
      TSIntrinsicKeyword: readonly never[];
      TSJSDocUnknownType: readonly never[];
      TSNeverKeyword: readonly never[];
      TSNullKeyword: readonly never[];
      TSNumberKeyword: readonly never[];
      TSObjectKeyword: readonly never[];
      TSStringKeyword: readonly never[];
      TSSymbolKeyword: readonly never[];
      TSThisType: readonly never[];
      TSUndefinedKeyword: readonly never[];
      TSUnknownKeyword: readonly never[];
      TSVoidKeyword: readonly never[];
      AccessorProperty: readonly string[];
      ArrayExpression: readonly string[];
      ArrayPattern: readonly string[];
      ArrowFunctionExpression: readonly string[];
      AssignmentExpression: readonly string[];
      AssignmentPattern: readonly string[];
      AwaitExpression: readonly string[];
      BinaryExpression: readonly string[];
      BlockStatement: readonly string[];
      BreakStatement: readonly string[];
      CallExpression: readonly string[];
      CatchClause: readonly string[];
      ChainExpression: readonly string[];
      ClassBody: readonly string[];
      ClassDeclaration: readonly string[];
      ClassExpression: readonly string[];
      ConditionalExpression: readonly string[];
      ContinueStatement: readonly string[];
      Decorator: readonly string[];
      DoWhileStatement: readonly string[];
      ExportAllDeclaration: readonly string[];
      ExportDefaultDeclaration: readonly string[];
      ExportNamedDeclaration: readonly string[];
      ExportSpecifier: readonly string[];
      ExpressionStatement: readonly string[];
      ForInStatement: readonly string[];
      ForOfStatement: readonly string[];
      ForStatement: readonly string[];
      FunctionDeclaration: readonly string[];
      FunctionExpression: readonly string[];
      Identifier: readonly string[];
      IfStatement: readonly string[];
      ImportAttribute: readonly string[];
      ImportDeclaration: readonly string[];
      ImportDefaultSpecifier: readonly string[];
      ImportExpression: readonly string[];
      ImportNamespaceSpecifier: readonly string[];
      ImportSpecifier: readonly string[];
      LabeledStatement: readonly string[];
      LogicalExpression: readonly string[];
      MemberExpression: readonly string[];
      MetaProperty: readonly string[];
      MethodDefinition: readonly string[];
      NewExpression: readonly string[];
      ObjectExpression: readonly string[];
      ObjectPattern: readonly string[];
      ParenthesizedExpression: readonly string[];
      Program: readonly string[];
      Property: readonly string[];
      PropertyDefinition: readonly string[];
      RestElement: readonly string[];
      ReturnStatement: readonly string[];
      SequenceExpression: readonly string[];
      SpreadElement: readonly string[];
      StaticBlock: readonly string[];
      SwitchCase: readonly string[];
      SwitchStatement: readonly string[];
      TaggedTemplateExpression: readonly string[];
      TemplateLiteral: readonly string[];
      ThrowStatement: readonly string[];
      TryStatement: readonly string[];
      UnaryExpression: readonly string[];
      UpdateExpression: readonly string[];
      V8IntrinsicExpression: readonly string[];
      VariableDeclaration: readonly string[];
      VariableDeclarator: readonly string[];
      WhileStatement: readonly string[];
      WithStatement: readonly string[];
      YieldExpression: readonly string[];
      JSXAttribute: readonly string[];
      JSXClosingElement: readonly string[];
      JSXElement: readonly string[];
      JSXExpressionContainer: readonly string[];
      JSXFragment: readonly string[];
      JSXMemberExpression: readonly string[];
      JSXNamespacedName: readonly string[];
      JSXOpeningElement: readonly string[];
      JSXSpreadAttribute: readonly string[];
      JSXSpreadChild: readonly string[];
      TSAbstractAccessorProperty: readonly string[];
      TSAbstractMethodDefinition: readonly string[];
      TSAbstractPropertyDefinition: readonly string[];
      TSArrayType: readonly string[];
      TSAsExpression: readonly string[];
      TSCallSignatureDeclaration: readonly string[];
      TSClassImplements: readonly string[];
      TSConditionalType: readonly string[];
      TSConstructSignatureDeclaration: readonly string[];
      TSConstructorType: readonly string[];
      TSDeclareFunction: readonly string[];
      TSEmptyBodyFunctionExpression: readonly string[];
      TSEnumBody: readonly string[];
      TSEnumDeclaration: readonly string[];
      TSEnumMember: readonly string[];
      TSExportAssignment: readonly string[];
      TSExternalModuleReference: readonly string[];
      TSFunctionType: readonly string[];
      TSImportEqualsDeclaration: readonly string[];
      TSImportType: readonly string[];
      TSIndexSignature: readonly string[];
      TSIndexedAccessType: readonly string[];
      TSInferType: readonly string[];
      TSInstantiationExpression: readonly string[];
      TSInterfaceBody: readonly string[];
      TSInterfaceDeclaration: readonly string[];
      TSInterfaceHeritage: readonly string[];
      TSIntersectionType: readonly string[];
      TSJSDocNonNullableType: readonly string[];
      TSJSDocNullableType: readonly string[];
      TSLiteralType: readonly string[];
      TSMappedType: readonly string[];
      TSMethodSignature: readonly string[];
      TSModuleBlock: readonly string[];
      TSModuleDeclaration: readonly string[];
      TSNamedTupleMember: readonly string[];
      TSNamespaceExportDeclaration: readonly string[];
      TSNonNullExpression: readonly string[];
      TSOptionalType: readonly string[];
      TSParameterProperty: readonly string[];
      TSParenthesizedType: readonly string[];
      TSPropertySignature: readonly string[];
      TSQualifiedName: readonly string[];
      TSRestType: readonly string[];
      TSSatisfiesExpression: readonly string[];
      TSTemplateLiteralType: readonly string[];
      TSTupleType: readonly string[];
      TSTypeAliasDeclaration: readonly string[];
      TSTypeAnnotation: readonly string[];
      TSTypeAssertion: readonly string[];
      TSTypeLiteral: readonly string[];
      TSTypeOperator: readonly string[];
      TSTypeParameter: readonly string[];
      TSTypeParameterDeclaration: readonly string[];
      TSTypeParameterInstantiation: readonly string[];
      TSTypePredicate: readonly string[];
      TSTypeQuery: readonly string[];
      TSTypeReference: readonly string[];
      TSUnionType: readonly string[];
    }>;
    /**
     * Ast node types.
     */
    readonly Syntax: Readonly<Record<string, string>>;
    /**
     * Latest ECMAScript version supported by parser.
     */
    latestEcmaVersion: 17;
    /**
     * ECMAScript versions supported by parser.
     */
    supportedEcmaVersions: readonly number[];
  }>;
  /**
   * Parser options used to parse the file being linted.
   */
  parserOptions: Readonly<{
    /**
     * Source type of the file being linted.
     */
    readonly sourceType: ModuleKind;
    /**
     * ECMA features.
     */
    ecmaFeatures: Readonly<{
      /**
       * `true` if file was parsed as JSX.
       */
      readonly jsx: boolean;
      /**
       * `true` if file was parsed with top-level `return` statements allowed.
       */
      readonly globalReturn: boolean;
      /**
       * `true` if file was parsed as strict mode code.
       */
      readonly impliedStrict: boolean;
    }>;
  }>;
  /**
   * Globals defined for the file being linted.
   */
  readonly globals: Readonly<Globals>;
  /**
   * Environments defined for the file being linted.
   */
  readonly env: Readonly<Envs>;
};
/**
 * Language options used when parsing a file.
 */
type LanguageOptions = Readonly<typeof LANGUAGE_OPTIONS>;
declare const FILE_CONTEXT: Readonly<{
  /**
   * Absolute path of the file being linted.
   */
  readonly filename: string;
  /**
   * Get absolute path of the file being linted.
   * @returns Absolute path of the file being linted.
   * @deprecated Use `context.filename` property instead.
   */
  getFilename(): string;
  /**
   * Physical absolute path of the file being linted.
   */
  readonly physicalFilename: string;
  /**
   * Get physical absolute path of the file being linted.
   * @returns Physical absolute path of the file being linted.
   * @deprecated Use `context.physicalFilename` property instead.
   */
  getPhysicalFilename(): string;
  /**
   * Current working directory.
   */
  readonly cwd: string;
  /**
   * Get current working directory.
   * @returns The current working directory.
   * @deprecated Use `context.cwd` property instead.
   */
  getCwd(): string;
  /**
   * Source code of the file being linted.
   */
  readonly sourceCode: SourceCode;
  /**
   * Get source code of the file being linted.
   * @returns Source code of the file being linted.
   * @deprecated Use `context.sourceCode` property instead.
   */
  getSourceCode(): SourceCode;
  /**
   * Language options used when parsing this file.
   */
  readonly languageOptions: LanguageOptions;
  /**
   * Settings for the file being linted.
   */
  readonly settings: Readonly<Settings>;
  /**
   * Create a new object with the current object as the prototype and
   * the specified properties as its own properties.
   * @param extension - The properties to add to the new object.
   * @returns A new object with the current object as the prototype
   *   and the specified properties as its own properties.
   */
  extend(this: FileContext, extension: Record<string | number | symbol, unknown>): FileContext;
  /**
   * Parser options used to parse the file being linted.
   * @deprecated Use `languageOptions.parserOptions` instead.
   */
  readonly parserOptions: Record<string, unknown>;
  /**
   * The path to the parser used to parse this file.
   * @deprecated No longer supported.
   */
  readonly parserPath: string | undefined;
}>;
/**
 * Context object for a file.
 * Is the prototype for `Context` objects for each rule.
 */
type FileContext = typeof FILE_CONTEXT;
/**
 * Context object for a rule.
 * Passed to `create` and `createOnce` functions.
 */
interface Context extends FileContext {
  /**
   * Rule ID, in form `<plugin>/<rule>`.
   */
  id: string;
  /**
   * Rule options for this rule on this file.
   */
  options: Readonly<Options>;
  /**
   * Report an error/warning.
   */
  report(this: void, diagnostic: Diagnostic): void;
}
//#endregion
//#region src-js/plugins/rule_meta.d.ts
/**
 * Rule metadata.
 * `meta` property of `Rule`.
 */
interface RuleMeta {
  /**
   * Type of rule.
   *
   * - `problem`: The rule is identifying code that either will cause an error or may cause a confusing behavior.
   *   Developers should consider this a high priority to resolve.
   * - `suggestion`: The rule is identifying something that could be done in a better way but no errors will occur
   *   if the code isn’t changed.
   * - `layout`: The rule cares primarily about whitespace, semicolons, commas, and parentheses, all the parts
   *   of the program that determine how the code looks rather than how it executes.
   *   These rules work on parts of the code that aren’t specified in the AST.
   */
  type?: "problem" | "suggestion" | "layout";
  /**
   * Rule documentation.
   */
  docs?: RuleDocs;
  /**
   * Templates for error/warning messages.
   */
  messages?: Record<string, string>;
  /**
   * Type of fixes that the rule provides.
   * Must be `'code'` or `'whitespace'` if the rule provides fixes.
   */
  fixable?: "code" | "whitespace";
  /**
   * Specifies whether rule can return suggestions.
   * Must be `true` if the rule provides suggestions.
   * @default false
   */
  hasSuggestions?: boolean;
  /**
   * Shape of options for the rule.
   * Mandatory if the rule has options.
   */
  schema?: RuleOptionsSchema;
  /**
   * Default options for the rule.
   * If present, any user-provided options in their config will be merged on top of them recursively.
   */
  defaultOptions?: Options;
  /**
   * Indicates whether the rule has been deprecated, and info about the deprecation and possible replacements.
   */
  deprecated?: boolean | RuleDeprecatedInfo;
  /**
   * Information about available replacements for the rule.
   * This may be an empty array to explicitly state there is no replacement.
   * @deprecated Use `deprecated.replacedBy` instead.
   */
  replacedBy?: RuleReplacedByInfo[];
}
/**
 * Rule documentation.
 * `docs` property of `RuleMeta`.
 *
 * Often used for documentation generation and tooling.
 */
interface RuleDocs {
  /**
   * Short description of the rule.
   */
  description?: string;
  /**
   * Typically a boolean, representing whether the rule is enabled by the recommended config.
   */
  recommended?: unknown;
  /**
   * URL for rule documentation.
   */
  url?: string;
  /**
   * Other arbitrary user-defined properties.
   */
  [key: string]: unknown;
}
/**
 * Info about deprecation of a rule, and possible replacements.
 * `deprecated` property of `RuleMeta`.
 */
interface RuleDeprecatedInfo {
  /**
   * General message presentable to the user. May contain why this rule is deprecated or how to replace the rule.
   */
  message?: string;
  /**
   * URL with more information about this rule deprecation.
   */
  url?: string;
  /**
   * Information about available replacements for the rule.
   * This may be an empty array to explicitly state there is no replacement.
   */
  replacedBy?: RuleReplacedByInfo[];
  /**
   * Version (as semver string) deprecating the rule.
   */
  deprecatedSince?: string;
  /**
   * Version (as semver string) likely to remove the rule.
   * e.g. the next major version.
   *
   * The special value `null` means the rule will no longer be changed, but will be kept available indefinitely.
   */
  availableUntil?: string | null;
}
/**
 * Info about a possible replacement for a rule.
 */
interface RuleReplacedByInfo {
  /**
   * A general message about this rule replacement.
   */
  message?: string;
  /**
   * A URL with more information about this rule replacement.
   */
  url?: string;
  /**
   * Which plugin has the replacement rule.
   *
   * The `name` property should be the package name, and should be:
   * - `"oxlint"` if the replacement is an Oxlint core rule.
   * - `"eslint"` if the replacement is an ESLint core rule.
   *
   * This property should be omitted if the replacement rule is in the same plugin.
   */
  plugin?: RuleReplacedByExternalSpecifier;
  /**
   * Name of replacement rule.
   * May be omitted if the plugin only contains a single rule, or has the same name as the rule.
   */
  rule?: RuleReplacedByExternalSpecifier;
}
/**
 * Details about a plugin or rule that replaces a deprecated rule.
 */
interface RuleReplacedByExternalSpecifier {
  /**
   * For a plugin, the package name.
   * For a rule, the rule name.
   */
  name?: string;
  /**
   * URL pointing to documentation for the plugin / rule.
   */
  url?: string;
}
//#endregion
//#region src-js/plugins/load.d.ts
/**
 * Linter plugin, comprising multiple rules
 */
interface Plugin {
  meta?: {
    name?: string;
  };
  rules: Record<string, Rule>;
}
/**
 * Linter rule.
 *
 * `Rule` can have either `create` method, or `createOnce` method.
 * If `createOnce` method is present, `create` is ignored.
 *
 * If defining the rule with `createOnce`, and you want the rule to work with ESLint too,
 * you need to wrap the plugin containing the rule with `eslintCompatPlugin`.
 */
type Rule = CreateRule | CreateOnceRule;
interface CreateRule {
  meta?: RuleMeta;
  create: (context: Context) => VisitorObject;
}
interface CreateOnceRule {
  meta?: RuleMeta;
  create?: (context: Context) => VisitorObject;
  createOnce: (context: Context) => VisitorWithHooks;
}
//#endregion
//#region src-js/package/define.d.ts
/**
 * Define a plugin.
 *
 * No-op function, just to provide type safety. Input is passed through unchanged.
 *
 * @param plugin - Plugin to define
 * @returns Same plugin as passed in
 */
declare function definePlugin(plugin: Plugin): Plugin;
/**
 * Define a rule.
 *
 * No-op function, just to provide type safety. Input is passed through unchanged.
 *
 * @param rule - Rule to define
 * @returns Same rule as passed in
 */
declare function defineRule(rule: Rule): Rule;
//#endregion
//#region src-js/package/compat.d.ts
/**
 * Convert a plugin which used Oxlint's `createOnce` API to also work with ESLint.
 *
 * If any of the plugin's rules use the Oxlint alternative `createOnce` API,
 * add ESLint-compatible `create` methods to those rules, which delegate to `createOnce`.
 * This makes the plugin compatible with ESLint.
 *
 * The `plugin` object passed in is mutated in-place.
 *
 * @param plugin - Plugin to convert
 * @returns Plugin with all rules having `create` method
 * @throws {Error} If `plugin` is not an object, or `plugin.rules` is not an object
 */
declare function eslintCompatPlugin(plugin: Plugin): Plugin;
//#endregion
export { type AfterHook, type BeforeHook, type BooleanToken, type Comment, type Context, type CountOptions, type CreateOnceRule, type CreateRule, type Definition, type DefinitionType, type Diagnostic, type DiagnosticData, type types_d_exports as ESTree, type Envs, type FilterFn, type Fix, type FixFn, type Fixer, type Globals, type IdentifierToken, type JSXIdentifierToken, type JSXTextToken, type KeywordToken, type LanguageOptions, type LineColumn, type Location, type Node, type NullToken, type NumericToken, type Options, type Plugin, type PrivateIdentifierToken, type PunctuatorToken, type Range, type RangeOptions, type Ranged, type Reference, type RegularExpressionToken, type Rule, type RuleDeprecatedInfo, type RuleDocs, type RuleMeta, type RuleOptionsSchema, type RuleReplacedByExternalSpecifier, type RuleReplacedByInfo, type Scope, type ScopeManager, type ScopeType, type Settings, type SkipOptions, type SourceCode, type Span, type StringToken, type Suggestion, type TemplateToken, type Token, type Variable, type VisitorObject as Visitor, type VisitorWithHooks, definePlugin, defineRule, eslintCompatPlugin };