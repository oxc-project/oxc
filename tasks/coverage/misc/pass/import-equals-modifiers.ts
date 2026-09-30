namespace A { export class B {} }
import Local = A.B;
export import Exported = A.B;
import External = require("external");
export import ExportedExternal = require("external");
export import type ExportedType = require("external");
declare namespace Ambient {
    import Local = A.B;
    export import Exported = A.B;
}
