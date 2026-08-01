import type importedKey from "mod";

const valueKey = Symbol();

interface Foo {
    get [valueKey](): Set<string>;
    set [valueKey](value: Set<string>);
    [importedKey](): string;
}

type T = typeof valueKey;
