declare function dec(target: any, propertyKey: string, desc: PropertyDescriptor): void;

class C {
    @dec accessor a: any;
    @dec static accessor b: any;
    @dec accessor c: string = "hello";
}
