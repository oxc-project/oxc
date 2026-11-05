declare const x: unique symbol;

interface I {
    [x](x: number): void;
}

type Method = {
    [x]<x>(value: x): void;
};

type Accessors = {
    get [x](): number;
    set [x](x: number);
};
