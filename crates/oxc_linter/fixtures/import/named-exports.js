var a = 1
var b = 2

export {a, b}

var c = 3
export {c as d}

export class ExportedClass {

}


// destructuring exports

export var { destructuredProp, ...restProps } = {}
         , { destructingAssign = null } = {}
         , { destructingAssign: destructingRenamedAssign = null } = {}
         , [ arrayKeyProp, ...arrayRestKeyProps ] = []
         , [ { deepProp } ] = []
         , { arr: [ ,, deepSparseElement ] } = {}
