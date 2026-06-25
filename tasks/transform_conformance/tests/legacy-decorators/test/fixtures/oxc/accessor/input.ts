declare function dec(target: any, propertyKey: string, desc: PropertyDescriptor): void;
declare var a: string;
declare var foo: { bar: string };

class C {
    @dec accessor a: any;
    @dec static accessor b: any;
    @dec accessor c: string = "hello";
    @dec accessor [a]: any;
    @dec accessor [foo.bar]: any;
    accessor d: any;
    accessor #e: any;
}
