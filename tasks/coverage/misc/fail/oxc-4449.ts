// should fail
enum A { [foo] }               // Computed property names are not allowed in enums
enum B { [1] }                 // An enum member cannot have a numeric name.
enum C { 1 }                   // An enum member cannot have a numeric name.
enum D { [`test${foo}`] }      // Computed property names are not allowed in enums.
enum E { `baz` = 2 }           // Enum member expected.
enum F { ['baz' + 'baz'] }     // Computed property names are not allowed in enums.
