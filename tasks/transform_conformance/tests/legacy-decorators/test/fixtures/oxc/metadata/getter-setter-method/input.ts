declare function dec(target: any, propertyKey: string, descriptor: PropertyDescriptor): void;

class C {
    @dec
    get address(): string {
        return "test";
    }

    @dec
    set address(value: string) {
    }

    @dec
    regularMethod(): string {
        return "test";
    }
}
