// should work
enum A { ['baz'] }             // ❌ currently fails
enum B { [`baz`] }             // ❌ currently fails
enum C { ['baz'] = 2 }         // ❌ currently fails
enum D { [`baz`] = 2 }         // ❌ currently fails
enum E { 'baz' }               // 👍 work fine
enum F { baz }                 // 👍 work fine
enum G { 'baz' = 2 }           // 👍 work fine
enum H { baz = 2 }             // 👍 work fine
