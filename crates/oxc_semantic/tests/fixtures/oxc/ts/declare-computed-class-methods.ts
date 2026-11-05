import type importedKey from "mod";

const declaredMethodKey = Symbol();
const declaredGetterKey = Symbol();
const abstractMethodKey = Symbol();
const abstractSetterKey = Symbol();
const runtimeMethodKey = Symbol();

declare class AmbientFoo {
    [declaredMethodKey](): string;
    get [declaredGetterKey](): string;
    [importedKey](): string;
}

abstract class AbstractFoo {
    abstract [abstractMethodKey](): string;
    abstract set [abstractSetterKey](value: string);
    abstract [importedKey](): string;
    [runtimeMethodKey](): string;
}
