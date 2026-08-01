// TS rejects computed enum member names, but oxc parses them;
// `['baz']` must format the same as `'baz'` so a second pass is stable (oxc-4449)
enum A { ['baz'] }
enum B { [`baz`] }
enum C { ['baz'] = 2 }
enum D { [`baz`] = 2 }
enum E { 'baz' }
enum F { baz }
enum G { 'b-az' = 2 }
enum H { 'baz' = 2, 'b-az' = 3 }
