use serde_json::json;

use super::NoUnusedVars;
use crate::{tester::Tester, RuleMeta as _};

// TODO: port these over. I (@DonIsaac) would love some help with this...

#[test]
fn test() {
    let pass = vec![
        (
            "
        import { ClassDecoratorFactory } from 'decorators';
        @ClassDecoratorFactory()
        export class Foo {}
            ",
            None,
        ),
        (
            "
        import { ClassDecorator } from 'decorators';
        @ClassDecorator
        export class Foo {}
            ",
            None,
        ),
        (
            "
        import { AccessorDecoratorFactory } from 'decorators';
        export class Foo {
          @AccessorDecoratorFactory(true)
          get bar() {}
        }
            ",
            None,
        ),
        (
            "
        import { AccessorDecorator } from 'decorators';
        export class Foo {
          @AccessorDecorator
          set bar() {}
        }
            ",
            None,
        ),
        (
            "
        import { MethodDecoratorFactory } from 'decorators';
        export class Foo {
          @MethodDecoratorFactory(false)
          bar() {}
        }
            ",
            None,
        ),
        (
            "
        import { MethodDecorator } from 'decorators';
        export class Foo {
          @MethodDecorator
          static bar() {}
        }
            ",
            None,
        ),
        (
            "
        import { ConstructorParameterDecoratorFactory } from 'decorators';
        export class Service {
          constructor(
            @ConstructorParameterDecoratorFactory(APP_CONFIG) config: AppConfig,
          ) {
            this.title = config.title;
          }
        }
            ",
            None,
        ),
        (
            "
        import { ConstructorParameterDecorator } from 'decorators';
        export class Foo {
          constructor(@ConstructorParameterDecorator bar) {
            this.bar = bar;
          }
        }
            ",
            None,
        ),
        (
            "
        import { ParameterDecoratorFactory } from 'decorators';
        export class Qux {
          bar(@ParameterDecoratorFactory(true) baz: number) {
            console.log(baz);
          }
        }
            ",
            None,
        ),
        (
            "
        import { ParameterDecorator } from 'decorators';
        export class Foo {
          static greet(@ParameterDecorator name: string) {
            return name;
          }
        }
            ",
            None,
        ),
        (
            "
        import { Input, Output, EventEmitter } from 'decorators';
        export class SomeComponent {
          @Input() data;
          @Output()
          click = new EventEmitter();
        }
            ",
            None,
        ),
        (
            "
        import { configurable } from 'decorators';
        export class A {
          @configurable(true) static prop1;

          @configurable(false)
          static prop2;
        }
            ",
            None,
        ),
        (
            "
        import { foo, bar } from 'decorators';
        export class B {
          @foo x;

          @bar
          y;
        }
            ",
            None,
        ),
        (
            "
        interface Base {}
        class Thing implements Base {}
        new Thing();
            ",
            None,
        ),
        (
            "
        interface Base {}
        const a: Base = {};
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Foo } from 'foo';
        function bar<T>(): T {}
        bar<Foo>();
            ",
            None,
        ),
        (
            "
        import { Foo } from 'foo';
        const bar = function <T>(): T {};
        bar<Foo>();
            ",
            None,
        ),
        (
            "
        import { Foo } from 'foo';
        const bar = <T,>(): T => {};
        bar<Foo>();
            ",
            None,
        ),
        (
            "
        import { Foo } from 'foo';
        <Foo>(<T,>(): T => {})();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Nullable<string> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        const a: Nullable<SomeOther> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Nullable | undefined = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Nullable & undefined = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        const a: Nullable<SomeOther[]> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        const a: Nullable<Array<SomeOther>> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Array<Nullable> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Nullable[] = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Array<Nullable[]> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        const a: Array<Array<Nullable>> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        const a: Array<Nullable<SomeOther>> = 'hello';
        console.log(a);
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Component } from 'react';
        class Foo implements Component<Nullable> {}

        new Foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Component } from 'react';
        class Foo extends Component<Nullable, {}> {}
        new Foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Component } from 'react';
        class Foo extends Component<Nullable<SomeOther>, {}> {}
        new Foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Component } from 'react';
        class Foo implements Component<Nullable<SomeOther>, {}> {}
        new Foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Component, Component2 } from 'react';
        class Foo implements Component<Nullable<SomeOther>, {}>, Component2 {}
        new Foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Another } from 'some';
        class A {
          do = (a: Nullable<Another>) => {
            console.log(a);
          };
        }
        new A();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Another } from 'some';
        class A {
          do(a: Nullable<Another>) {
            console.log(a);
          }
        }
        new A();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Another } from 'some';
        class A {
          do(): Nullable<Another> {
            return null;
          }
        }
        new A();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Another } from 'some';
        export interface A {
          do(a: Nullable<Another>);
        }
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { Another } from 'some';
        export interface A {
          other: Nullable<Another>;
        }
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        function foo(a: Nullable) {
          console.log(a);
        }
        foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        function foo(): Nullable {
          return null;
        }
        foo();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        class A extends Nullable<SomeOther> {
          other: Nullable<Another>;
        }
        new A();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Another } from 'some';
        class A extends Nullable<SomeOther> {
          do(a: Nullable<Another>) {
            console.log(a);
          }
        }
        new A();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Another } from 'some';
        export interface A extends Nullable<SomeOther> {
          other: Nullable<Another>;
        }
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'some';
        import { Another } from 'some';
        export interface A extends Nullable<SomeOther> {
          do(a: Nullable<Another>);
        }
            ",
            None,
        ),
        (
            "
        import { Foo } from './types';

        class Bar<T extends Foo> {
          prop: T;
        }

        new Bar<number>();
            ",
            None,
        ),
        (
            "
        import { Foo, Bar } from './types';

        class Baz<T extends Foo & Bar> {
          prop: T;
        }

        new Baz<any>();
            ",
            None,
        ),
        (
            "
        import { Foo } from './types';

        class Bar<T = Foo> {
          prop: T;
        }

        new Bar<number>();
            ",
            None,
        ),
        (
            "
        import { Foo } from './types';

        class Foo<T = any> {
          prop: T;
        }

        new Foo();
            ",
            None,
        ),
        (
            "
        import { Foo } from './types';

        class Foo<T = {}> {
          prop: T;
        }

        new Foo();
            ",
            None,
        ),
        (
            "
        import { Foo } from './types';

        class Foo<T extends {} = {}> {
          prop: T;
        }

        new Foo();
            ",
            None,
        ),
        // FIXME
        (
            "
        type Foo = 'a' | 'b' | 'c';
        type Bar = number;

        export const map: { [name in Foo]: Bar } = {
          a: 1,
          b: 2,
          c: 3,
        };
            ",
            None,
        ),
        // 4.1 remapped mapped type
        (
            "
        type Foo = 'a' | 'b' | 'c';
        type Bar = number;

        export const map: { [name in Foo as string]: Bar } = {
          a: 1,
          b: 2,
          c: 3,
        };
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        class A<T> {
          bar: T;
        }
        new A<Nullable>();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        function foo<T extends Nullable>(): T {}
        foo<SomeOther>();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        class A<T extends Nullable> {
          bar: T;
        }
        new A<SomeOther>();
            ",
            None,
        ),
        (
            "
        import { Nullable } from 'nullable';
        import { SomeOther } from 'other';
        interface A<T extends Nullable> {
          bar: T;
        }
        export const a: A<SomeOther> = {
          foo: 'bar',
        };
            ",
            None,
        ),
        // https://github.com/bradzacher/eslint-plugin-typescript/issues/150
        (
            "
        export class App {
          constructor(private logger: Logger) {
            console.log(this.logger);
          }
        }
            ",
            None,
        ),
        (
            "
        export class App {
          constructor(bar: string);
          constructor(private logger: Logger) {
            console.log(this.logger);
          }
        }
            ",
            None,
        ),
        (
            "
        export class App {
          constructor(
            baz: string,
            private logger: Logger,
          ) {
            console.log(baz);
            console.log(this.logger);
          }
        }
            ",
            None,
        ),
        (
            "
        export class App {
          constructor(
            baz: string,
            private logger: Logger,
            private bar: () => void,
          ) {
            console.log(this.logger);
            this.bar();
          }
        }
            ",
            None,
        ),
        (
            "
        export class App {
          constructor(private logger: Logger) {}
          meth() {
            console.log(this.logger);
          }
        }
            ",
            None,
        ),
        // https://github.com/bradzacher/eslint-plugin-typescript/issues/126
        (
            "
        import { Component, Vue } from 'vue-property-decorator';
        import HelloWorld from './components/HelloWorld.vue';

        @Component({
          components: {
            HelloWorld,
          },
        })
        export default class App extends Vue {}
            ",
            None,
        ),
        // https://github.com/bradzacher/eslint-plugin-typescript/issues/189
        (
            "
        import firebase, { User } from 'firebase/app';
        // initialize firebase project
        firebase.initializeApp({});
        export function authenticated(cb: (user: User | null) => void): void {
          firebase.auth().onAuthStateChanged(user => cb(user));
        }
            ",
            None,
        ),
        // https://github.com/bradzacher/eslint-plugin-typescript/issues/33
        (
            "
        import { Foo } from './types';
        export class Bar<T extends Foo> {
          prop: T;
        }
            ",
            None,
        ),
        (
            "
        import webpack from 'webpack';
        export default function webpackLoader(this: webpack.loader.LoaderContext) {}
            ",
            None,
        ),
        (
            "
        import execa, { Options as ExecaOptions } from 'execa';
        export function foo(options: ExecaOptions): execa {
          options();
        }
            ",
            None,
        ),
        (
            "
        import { Foo, Bar } from './types';
        export class Baz<F = Foo & Bar> {
          prop: F;
        }
            ",
            None,
        ),
        (
            "
        // warning 'B' is defined but never used
        export const a: Array<{ b: B }> = [];
            ",
            None,
        ),
        (
            "
        export enum FormFieldIds {
          PHONE = 'phone',
          EMAIL = 'email',
        }
            ",
            None,
        ),
        (
            "
        enum FormFieldIds {
          PHONE = 'phone',
          EMAIL = 'email',
        }
        export interface IFoo {
          fieldName: FormFieldIds;
        }
            ",
            None,
        ),
        (
            "
        enum FormFieldIds {
          PHONE = 'phone',
          EMAIL = 'email',
        }
        export interface IFoo {
          fieldName: FormFieldIds.EMAIL;
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/25
        (
            "
        import * as fastify from 'fastify';
        import { Server, IncomingMessage, ServerResponse } from 'http';
        const server: fastify.FastifyInstance<Server, IncomingMessage, ServerResponse> =
          fastify({});
        server.get('/ping');
            ",
            None,
        ),
        // FIXME
        // https://github.com/typescript-eslint/typescript-eslint/issues/61
        // (
        //     "
        // declare namespace Foo {
        //   function bar(line: string, index: number | null, tabSize: number): number;
        //   var baz: string;
        // }
        // console.log(Foo);
        //     ",
        //     None,
        // ),
        (
            "
        import foo from 'foo';
        export interface Bar extends foo.i18n {}
            ",
            None,
        ),
        (
            "
        import foo from 'foo';
        import bar from 'foo';
        export interface Bar extends foo.i18n<bar> {}
            ",
            None,
        ),
        // https://github.com/eslint/typescript-eslint-parser/issues/535
        (
            "
        import { observable } from 'mobx';
        export default class ListModalStore {
          @observable
          orderList: IObservableArray<BizPurchaseOrderTO> = observable([]);
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/122#issuecomment-462008078
        (
            "
        import { Dec, TypeA, Class } from 'test';
        export default class Foo {
          constructor(
            @Dec(Class)
            private readonly prop: TypeA<Class>,
          ) {}
        }
            ",
            None,
        ),
        // FIXME - parse error
        // (
        //     "
        // import { Dec, TypeA, Class } from 'test';
        // export default class Foo {
        //   constructor(
        //     @Dec(Class)
        //     ...prop: TypeA<Class>
        //   ) {
        //     prop();
        //   }
        // }
        //     ",
        //     None,
        // ),
        (
            "
        export function foo(): void;
        export function foo(): void;
        export function foo(): void {}
            ",
            None,
        ),
        (
            "
        export function foo(a: number): number;
        export function foo(a: string): string;
        export function foo(a: number | string): number | string {
          return a;
        }
            ",
            None,
        ),
        (
            "
        export function foo<T>(a: number): T;
        export function foo<T>(a: string): T;
        export function foo<T>(a: number | string): T {
          return a;
        }
            ",
            None,
        ),
        (
            "
        export type T = {
          new (): T;
          new (arg: number): T;
          new <T>(arg: number): T;
        };
            ",
            None,
        ),
        // NOTE: Parse error
        // (
        //     "
        // export type T = new () => T;
        // export type T = new (arg: number) => T;
        // export type T = new <T>(arg: number) => T;
        //     ",
        //     None,
        // ),
        (
            "
        enum Foo {
          a,
        }
        export type T = {
          [Foo.a]: 1;
        };
            ",
            None,
        ),
        (
            "
        type Foo = string;
        export class Bar {
          [x: Foo]: any;
        }
            ",
            None,
        ),
        (
            "
        type Foo = string;
        export class Bar {
          [x: Foo]: Foo;
        }
            ",
            None,
        ),
        (
            "
        namespace Foo {
          export const Foo = 1;
        }

        export { Foo };
            ",
            None,
        ),
        (
            "
        export namespace Foo {
          export const item: Foo = 1;
        }
            ",
            None,
        ),
        // (
        //     "
        // namespace foo.bar {
        //   export interface User {
        //     name: string;
        //   }
        // }
        //     ",
        //     None,
        // ),
        // exported self-referencing types
        (
            "
        export interface Foo {
          bar: string;
          baz: Foo['bar'];
        }
            ",
            None,
        ),
        (
            "
        export type Bar = Array<Bar>;
            ",
            None,
        ),
        // declaration merging
        (
            "
        function Foo() {}

        namespace Foo {
          export const x = 1;
        }

        export { Foo };
            ",
            None,
        ),
        (
            "
        class Foo {}

        namespace Foo {
          export const x = 1;
        }

        export { Foo };
            ",
            None,
        ),
        (
            "
        namespace Foo {}

        const Foo = 1;

        export { Foo };
            ",
            None,
        ),
        (
            "
        type Foo = {
          error: Error | null;
        };

        export function foo() {
          return new Promise<Foo>();
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/5152
        (
            "
        function foo<T>(value: T): T {
          return { value };
        }
        export type Foo<T> = typeof foo<T>;
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2331
        (
            "
        export interface Event<T> {
          (
            listener: (e: T) => any,
            thisArgs?: any,
            disposables?: Disposable[],
          ): Disposable;
        }
        ",
            Some(
                json!( [{ "args": "after-used", "argsIgnorePattern": "^_", "ignoreRestSiblings": true, "varsIgnorePattern": "^_$"}] ),
            ),
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2369
        (
            "
        export class Test {
          constructor(@Optional() value: number[] = []) {
            console.log(value);
          }
        }

        function Optional() {
          return () => {};
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2417
        (
            "
        import { FooType } from './fileA';

        export abstract class Foo {
          protected abstract readonly type: FooType;
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2449
        (
            "
        export type F<A extends unknown[]> = (...a: A) => unknown;
            ",
            None,
        ),
        (
            "
        import { Foo } from './bar';
        export type F<A extends unknown[]> = (...a: Foo<A>) => unknown;
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2452
        (
            r"
        type StyledPaymentProps = {
          isValid: boolean;
        };

        export const StyledPayment = styled.div<StyledPaymentProps>``;
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2453
        (
            "
        import type { foo } from './a';
        export type Bar = typeof foo;
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2459
        (
            "
        export type Test<U> = U extends (k: infer I) => void ? I : never;
            ",
            None,
        ),
        (
            "
        export type Test<U> = U extends { [k: string]: infer I } ? I : never;
            ",
            None,
        ),
        (
            "
        export type Test<U> = U extends (arg: {
          [k: string]: (arg2: infer I) => void;
        }) => void
          ? I
          : never;
            ",
            None,
        ),
        // (
        //     "
        // declare module 'foo' {
        //   type Test = 1;
        // }
        // ",
        //     None,
        // ),
        (
            "
        declare module 'foo' {
          type Test = 1;
          const x: Test = 1;
          export = x;
        }
            ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2523
        (
            "
        declare global {
          interface Foo {}
        }
            ",
            None,
        ),
        (
            "
        declare global {
          namespace jest {
            interface Matchers<R> {
              toBeSeven: () => R;
            }
          }
        }
        ",
            None,
        ),
        // (
        //     "
        // export declare namespace Foo {
        //   namespace Bar {
        //     namespace Baz {
        //       namespace Bam {
        //         const x = 1;
        //       }
        //     }
        //   }
        // }
        // ",
        //     None,
        // ),
        (
            "
            class Foo<T> {
                value: T;
            }
            class Bar<T> {
                foo = Foo<T>;
            }
            new Bar();
            ",
            None,
        ),
        //     // 4.1 template literal types
        (
            r"
        type Color = 'red' | 'blue';
        type Quantity = 'one' | 'two';
        export type SeussFish = `${Quantity | Color} fish`;
              ",
            None,
        ),
        (
            r#"
        type VerticalAlignment = "top" | "middle" | "bottom";
        type HorizontalAlignment = "left" | "center" | "right";

        export declare function setAlignment(value: `${VerticalAlignment}-${HorizontalAlignment}`): void;
        "#,
            None,
        ),
        (
            r#"
        type EnthusiasticGreeting<T extends string> = `${Uppercase<T>} - ${Lowercase<T>} - ${Capitalize<T>} - ${Uncapitalize<T>}`;
        export type HELLO = EnthusiasticGreeting<"heLLo">;"#,
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2648
        // ignored by pattern, even though it's only self-referenced
        (
            "
            namespace _Foo {
                export const bar = 1;
                export const baz = Foo.bar;
            }",
            Some(json!([{ "varsIgnorePattern": "^_" }])),
        ),
        (
            "
            interface _Foo {
                a: string;
                b: _Foo;
            }
            ",
            Some(json!([{ "varsIgnorePattern": "^_" }])),
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2844
        (
            r#"
        /* eslint collect-unused-vars: "error" */
        declare module 'next-auth' {
          interface User {
            id: string;
            givenName: string;
            familyName: string;
          }
        }
            "#,
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2972
        // (
        //     "

        // import { TestGeneric, Test } from 'fake-module';

        // declare function deco(..._param: any): any;
        // export class TestClass {
        //   @deco
        //   public test(): TestGeneric<Test> {}
        // }
        //       ",
        //     None,
        // ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/5577
        (
            "
        function foo() {}

        export class Foo {
          constructor() {
            foo();
          }
        }
            ",
            None,
        ),
        (
            "
        function foo() {}

        export class Foo {
            static {}

            constructor() {
                foo();
            }
        }
        ",
            None,
        ),
        (
            "
        interface Foo {
          bar: string;
        }
        export const Foo = 'bar';
            ",
            None,
        ),
        (
            "
        export const Foo = 'bar';
        interface Foo {
          bar: string;
        }
            ",
            None,
        ),
        // NOTE: intentional behavior change
        // (
        //     "
        // let foo = 1;
        // foo ??= 2;
        //     ",
        //     None,
        // ),
        // (
        //     "
        // let foo = 1;
        // foo &&= 2;
        //     ",
        //     None,
        // ),
        // (
        //     "
        // let foo = 1;
        // foo ||= 2;
        //     ",
        //     None,
        // ),
        (
            "
        const foo = 1;
        export = foo;
            ",
            None,
        ),
        (
            "
        const Foo = 1;
        interface Foo {
          bar: string;
        }
        export = Foo;
            ",
            None,
        ),
        (
            "
        interface Foo {
          bar: string;
        }
        export = Foo;
            ",
            None,
        ),
        (
            "
        type Foo = 1;
        export = Foo;
            ",
            None,
        ),
        (
            "
        type Foo = 1;
        export = {} as Foo;
            ",
            None,
        ),
        (
            "
        declare module 'foo' {
          type Foo = 1;
          export = Foo;
        }
            ",
            None,
        ),
        (
            "
        namespace Foo {
          export const foo = 1;
        }
        export namespace Bar {
          export import TheFoo = Foo;
        }
            ",
            None,
        ),
    ];

    let fail = vec![
        ("import { ClassDecoratorFactory } from 'decorators'; export class Foo {}", None),
        (
            "import { Foo, Bar } from 'foo';
        function baz<Foo>(): Foo {}
        baz<Bar>();
        ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          const a: string = 'hello';
          console.log(a);
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { SomeOther } from 'other';
          const a: Nullable<string> = 'hello';
          console.log(a);
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { Another } from 'some';
          class A {
            do = (a: Nullable) => {
              console.log(a);
            };
          }
          new A();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { Another } from 'some';
          class A {
            do(a: Nullable) {
              console.log(a);
            }
          }
          new A();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { Another } from 'some';
          class A {
            do(): Nullable {
              return null;
            }
          }
          new A();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { Another } from 'some';
          export interface A {
            do(a: Nullable);
          }
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { Another } from 'some';
          export interface A {
            other: Nullable;
          }
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          function foo(a: string) {
            console.log(a);
          }
          foo();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          function foo(): string | null {
            return null;
          }
          foo();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { SomeOther } from 'some';
          import { Another } from 'some';
          class A extends Nullable {
            other: Nullable<Another>;
          }
          new A();
                ",
            None,
        ),
        (
            "
          import { Nullable } from 'nullable';
          import { SomeOther } from 'some';
          import { Another } from 'some';
          abstract class A extends Nullable {
            other: Nullable<Another>;
          }
          new A();
                ",
            None,
        ),
        (
            "
        enum FormFieldIds {
          PHONE = 'phone',
          EMAIL = 'email',
        }
            ",
            None,
        ),
        (
            "
          import test from 'test';
          import baz from 'baz';
          export interface Bar extends baz.test {}
                ",
            None,
        ),
        (
            "
          import test from 'test';
          import baz from 'baz';
          export interface Bar extends baz.test {}
                ",
            None,
        ),
        (
            "
          import test from 'test';
          import baz from 'baz';
          export interface Bar extends baz().test {}
          ",
            None,
        ),
        (
            "
          import test from 'test';
          import baz from 'baz';
          export class Bar implements baz.test {}
          ",
            None,
        ),
        (
            "
          import test from 'test';
          import baz from 'baz';
          export class Bar implements baz.test {}
                ",
            None,
        ),
        // NOTE: parse error
        // (
        //     "
        //   import test from 'test';
        //   import baz from 'baz';
        //   export class Bar implements baz().test {}
        //   ",
        //     None,
        // ),
        ("namespace Foo {}", None),
        ("namespace Foo { export const Foo = 1; } ", None),
        (
            "
            namespace Foo {
                const Foo = 1;
                console.log(Foo);
            }
            ",
            None,
        ),
        // (
        //     "
        //     namespace Foo {
        //         export const Bar = 1;
        //         console.log(Foo.Bar);
        //     }
        //     ",
        //     None,
        // ),
        (
            "
        namespace Foo {
          namespace Foo {
            export const Bar = 1;
            console.log(Foo.Bar);
          }
        }
        ",
            None,
        ),
        // self-referencing types
        // (
        //     "
        //     interface Foo {
        //         bar: string;
        //         baz: Foo['bar'];
        //     }
        //     ",
        //     None,
        // ),
        // FIXME
        ("type Foo = Array<Foo>", None),
        (
            "
        declare module 'foo' {
            type Test = any;
            const x = 1;
            export = x;
        }
        ",
            None,
        ),
        (
            "
        // not declared
        export namespace Foo {
            namespace Bar {
                namespace Baz {
                    namespace Bam {
                        const x = 1;
                    }
                }
            }
        }
        ",
            None,
        ),
        // (
        //     "
        //   interface Foo {
        //     a: string;
        //   }
        //   interface Foo {
        //     b: Foo;
        //   }
        //         ",
        //     None,
        // ),
        // ("let x = null; x = foo(x);", None),
        (
            "
            interface Foo {
                bar: string;
            }
            const Foo = 'bar';
                ",
            None,
        ),
        (
            "
          let foo = 1;
          foo += 1;
                ",
            None,
        ),
        (
            "
          interface Foo {
            bar: string;
          }
          type Bar = 1;
          export = Bar;
                ",
            None,
        ),
        (
            "
          interface Foo {
            bar: string;
          }
          type Bar = 1;
          export = Foo;
        ",
            None,
        ),
        (
            "
          namespace Foo {
            export const foo = 1;
          }
          export namespace Bar {
            import TheFoo = Foo;
          },
        ",
            None,
        ),
        ("const foo: number = 1;", None),
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript-eslint")
        .test_and_snapshot();
}

#[test]
fn test_tsx() {
    let pass = vec![
        // https://github.com/typescript-eslint/typescript-eslint/issues/141
        (
            "
        import { TypeA } from './interface';
        export const a = <GenericComponent<TypeA> />;
        ",
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/160
        (
            r#"
        const text = 'text';
        export function Foo() {
          return (
            <div>
              <input type="search" size={30} placeholder={text} />
            </div>
          );
        }"#,
            None,
        ),
        // https://github.com/typescript-eslint/typescript-eslint/issues/2455
        (
            "
                import React from 'react';

                export const ComponentFoo: React.FC = () => {
                  return <div>Foo Foo</div>;
                };
              ",
            None,
        ),
        // FIXME: Support JSX pragmas
        //       parserOptions: {
        //         ecmaFeatures: {
        //           jsx: true,
        //         },
        //         jsxPragma: 'h',
        //       },
        (
            "
                import { h } from 'some-other-jsx-lib';

                export const ComponentFoo: h.FC = () => {
                  return <div>Foo Foo</div>;
                };
              ",
            None,
        ),
        // NOTE: I'm not sure why this passes, but it does. I don't see any
        // implicit references to `Fragment` in semantic, but I won't look a
        // gift horse in the mouth.
        //       parserOptions: {
        //         ecmaFeatures: {
        //           jsx: true,
        //         },
        //         jsxFragmentName: 'Fragment',
        //       },
        (
            "
                import { Fragment } from 'react';

                export const ComponentFoo: Fragment = () => {
                  return <>Foo Foo</>;
                };
              ",
            None,
        ),
    ];

    let fail = vec![
        // https://github.com/typescript-eslint/typescript-eslint/issues/2455
        (
            "
        import React from 'react';
        import { Fragment } from 'react';

        export const ComponentFoo = () => {
          return <div>Foo Foo</div>;
        };
              ",
            None,
        ),
        //       {
        //         code: `
        //   import React from 'react';
        //   import { Fragment } from 'react';

        //   export const ComponentFoo = () => {
        //     return <div>Foo Foo</div>;
        //   };
        //         `,
        //         parserOptions: {
        //           ecmaFeatures: {
        //             jsx: true,
        //           },
        //         },
        //         errors: [
        //           {
        //             messageId: 'unusedVar',
        //             line: 3,
        //             column: 10,
        //             data: {
        //               varName: 'Fragment',
        //               action: 'defined',
        //               additional: '',
        //             },
        //           },
        //         ],
        //       },
        //       {
        //         code: `
        //   import React from 'react';
        //   import { h } from 'some-other-jsx-lib';

        //   export const ComponentFoo = () => {
        //     return <div>Foo Foo</div>;
        //   };
        //         `,
        //         parserOptions: {
        //           ecmaFeatures: {
        //             jsx: true,
        //           },
        //           jsxPragma: 'h',
        //         },
        //         errors: [
        //           {
        //             messageId: 'unusedVar',
        //             line: 2,
        //             column: 8,
        //             data: {
        //               varName: 'React',
        //               action: 'defined',
        //               additional: '',
        //             },
        //           },
        //         ],
        //       },

        // https://github.com/typescript-eslint/typescript-eslint/issues/3303
        // (
        //     "
        //   import React from 'react';

        //   export const ComponentFoo = () => {
        //     return <div>Foo Foo</div>;
        //   };
        //         ",
        //     None,
        // ),
        //         parserOptions: {
        //           ecmaFeatures: {
        //             jsx: true,
        //           },
        //           jsxPragma: null,
        //         },
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("typescript-eslint-tsx")
        .test_and_snapshot();
}

#[test]
fn test_d_ts() {
    let pass = vec![
        // https://github.com/typescript-eslint/typescript-eslint/issues/2456
        "
        interface Foo {}
        type Bar = {};
        declare class Clazz {}
        declare function func();
        declare enum Enum {}
        declare namespace Name {}
        declare const v1 = 1;
        declare var v2 = 1;
        declare let v3 = 1;
        declare const { v4 };
        declare const { v4: v5 };
        declare const [v6];
        ",
        "
        declare namespace A {
          export interface A {}
        }
        ",
        "declare function A(A: string): string;",
        // https://github.com/typescript-eslint/typescript-eslint/issues/2714
        "
        interface IItem {
          title: string;
          url: string;
          children?: IItem[];
        }
        ",
    ];
    let fail = vec![];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .change_rule_path_extension("d.ts")
        .test();
}
