import type * as TypeOnlyNamespace from "type-only";
import * as RuntimeNamespace from "runtime";

const LocalBase = class {};

declare class Declared extends TypeOnlyNamespace.Base {}
declare class NamespaceDeclared extends RuntimeNamespace.Base {}
declare class LocalDeclared extends LocalBase {}
class Runtime extends RuntimeNamespace.Base {}
