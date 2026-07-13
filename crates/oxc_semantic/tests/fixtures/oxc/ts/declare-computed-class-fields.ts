import type importedKey from "mod";

const declaredKey = "";
const ambientKey = "";
const abstractKey = "";
const runtimeKey = "";

class Foo {
    declare [declaredKey]: string;
    declare [importedKey]: string;
    [runtimeKey]: string;
}

declare class AmbientFoo {
    [ambientKey]: string;
    [importedKey]: string;
}

abstract class AbstractFoo {
    abstract [abstractKey]: string;
    abstract [importedKey]: string;
}
