export class Foo {
    public [Symbol.iterator](): Iterator<number> {
        return {
            next: () => ({ done: false, value: 1 }),
        };
    }
}

export class Bar {
    *[Symbol.iterator](): Generator<number> {
        yield 1;
        yield 2;
        yield 3;
    }
}

export class Bang {
    *[globalThis.Symbol.iterator](): Generator<number> {
        yield 1;
        yield 2;
        yield 3;
    }
}

export const bang = {
    [Symbol.iterator](): Iterator<number> {
        return {
            next: () => ({ done: false, value: 1 }),
        }
    },
};

export const boom = {
    *[Symbol.iterator](): Generator<number> {
        yield 1;
        yield 2;
        yield 3;
    }
};


// invalid cases

const x = Symbol("foo");
export class A {
    [x](): number {
        return 1;
    }
}
export const b = {
    [x]: 1,
}
