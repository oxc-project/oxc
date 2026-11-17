#![cfg(doctest)]

//! Tests to ensure the compiler rejects code which misuses brands.
//!
//! These have to be doctests, as that's the only kind of test which can check
//! that code fails to compile.

/**
Doctests to ensure the compiler rejects code which mixes 2 different brands.

The 1st test is a control. It's the same as the others, except it doesn't mix brands.
It compiles, so the others fail to compile because they mix brands, not for some other reason.

```
use oxc_data_structures::brands::{Brand, BrandOrigin};

fn origin_and_brand<'id>(_origin: &BrandOrigin<'id>, _brand: Brand<'id>) {}
fn two_brands<'id>(_brand1: Brand<'id>, _brand2: Brand<'id>) {}

BrandOrigin::scope(|origin1| {
    BrandOrigin::scope(|origin2| {
        origin_and_brand(&origin1, origin1.brand());
        origin_and_brand(&origin2, origin2.brand());
        two_brands(origin1.brand(), origin1.brand());
        two_brands(origin2.brand(), origin2.brand());
    });
});
```

A `Brand` can't be used with the `BrandOrigin` of a different brand.

```compile_fail
use oxc_data_structures::brands::{Brand, BrandOrigin};

fn origin_and_brand<'id>(_origin: &BrandOrigin<'id>, _brand: Brand<'id>) {}

BrandOrigin::scope(|origin1| {
    BrandOrigin::scope(|origin2| {
        origin_and_brand(&origin1, origin2.brand());
    });
});
```

2 `Brand`s from different `BrandOrigin`s can't be used together.
This would compile if `Brand` was covariant over `'id`.

```compile_fail
use oxc_data_structures::brands::{Brand, BrandOrigin};

fn two_brands<'id>(_brand1: Brand<'id>, _brand2: Brand<'id>) {}

BrandOrigin::scope(|origin1| {
    BrandOrigin::scope(|origin2| {
        two_brands(origin1.brand(), origin2.brand());
    });
});
```
*/
const _BRANDS_CANNOT_BE_MIXED: () = ();

/**
Doctests to ensure the compiler rejects code which lets a brand leave the closure it was created in.

The 1st test is a control. It's the same as the others, except it returns something
with no brand. It compiles, so the others fail to compile because of what they return.

```
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

let _: usize = BrandOrigin::scope(|origin| size_of_val(&origin));
let _: usize = BrandOrigin::scope(|origin| size_of_val(&origin.brand()));
let _: usize = BrandOrigin::scope_within(|origin, _: BrandScope<'_, 'static>| size_of_val(&origin));
```

A `BrandOrigin` can't be returned from `BrandOrigin::scope`.

```compile_fail
use oxc_data_structures::brands::BrandOrigin;

let _ = BrandOrigin::scope(|origin| origin);
```

A `Brand` can't be returned from `BrandOrigin::scope`.

```compile_fail
use oxc_data_structures::brands::BrandOrigin;

let _ = BrandOrigin::scope(|origin| origin.brand());
```

A `BrandOrigin` can't be returned from `BrandOrigin::scope_within`.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

let _ = BrandOrigin::scope_within(|origin, _: BrandScope<'_, 'static>| origin);
```

Data which doesn't outlive `'data` can't be shortened to `'id`.
The 1st block is a control, which compiles.

```
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn shorten<'id>(_origin: &BrandOrigin<'id>, _text: &'id str) {}

fn test<'data>(text: &'data str) {
    BrandOrigin::scope_within(|origin, _: BrandScope<'_, 'data>| shorten(&origin, text));
}
```

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn shorten<'id>(_origin: &BrandOrigin<'id>, _text: &'id str) {}

fn test<'data>(_text: &'data str) {
    BrandOrigin::scope_within(|origin, _: BrandScope<'_, 'data>| {
        let local = String::new();
        shorten(&origin, &local);
    });
}
```

A `BrandOrigin` can't be stored in a variable outside the closure.
The 1st block is a control, which compiles.

```
use oxc_data_structures::brands::BrandOrigin;

let mut slot = None;
BrandOrigin::scope(|origin| slot = Some(size_of_val(&origin)));
assert_eq!(slot, Some(0));
```

```compile_fail
use oxc_data_structures::brands::BrandOrigin;

let mut slot = None;
BrandOrigin::scope(|origin| slot = Some(origin));
```
*/
const _BRANDS_CANNOT_ESCAPE: () = ();

/**
Doctests to ensure the compiler rejects misuse of `BrandScope::extend_lifetime`.

The 1st test is a control. It compiles, so the others fail to compile because of what they extend.

```
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(text: &'data str) -> &'data str {
    BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<&str>(text)
    })
}
```

The lifetime of a reference to a local variable can't be extended.
This would compile if `BrandScope` was covariant over `'id`.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(_text: &'data str) -> &'data str {
    BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        let local = String::new();
        brand_scope.extend_lifetime::<&str>(&local)
    })
}
```

A lifetime can't be extended to longer than `'data`.
This would compile if `BrandScope` was contravariant over `'data`.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(text: &'data str) -> &'static str {
    BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<&str>(text)
    })
}
```

The lifetime of a `BrandOrigin` or `Brand` can't be extended.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(_text: &'data str) {
    let _ = BrandOrigin::scope_within(|origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<BrandOrigin>(origin)
    });
}
```

```compile_fail
use oxc_data_structures::brands::{Brand, BrandOrigin, BrandScope};

fn test<'data>(_text: &'data str) {
    let _ = BrandOrigin::scope_within(|origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<Brand>(origin.brand())
    });
}
```

Nor can the lifetime of a reference to one, because that reference isn't `Unbranded`.
The 1st block is a control, which compiles.

```
use oxc_data_structures::brands::{BrandOrigin, Unbranded};

fn assert_unbranded<T: Unbranded>(_value: &T) {}

BrandOrigin::scope(|_origin| {
    let text = String::new();
    assert_unbranded::<&String>(&&text);
});
```

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, Unbranded};

fn assert_unbranded<T: Unbranded>(_value: &T) {}

BrandOrigin::scope(|origin| {
    assert_unbranded::<&BrandOrigin>(&&origin);
});
```

A `&mut` reference can have its lifetime extended, but the original can't be used
while the extended one exists. The 1st block is a control, which compiles.

```
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(num: &'data mut u32) -> &'data mut u32 {
    BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<&mut u32>(&mut *num)
    })
}

let mut num = 1;
*test(&mut num) += 1;
assert_eq!(num, 2);
```

The original can't be used in the closure.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(num: &'data mut u32) -> &'data mut u32 {
    BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        let extended = brand_scope.extend_lifetime::<&mut u32>(&mut *num);
        *num += 1;
        extended
    })
}
```

Nor after the closure.

```compile_fail
use oxc_data_structures::brands::{BrandOrigin, BrandScope};

fn test<'data>(num: &'data mut u32) {
    let extended = BrandOrigin::scope_within(|_origin, brand_scope: BrandScope<'_, 'data>| {
        brand_scope.extend_lifetime::<&mut u32>(&mut *num)
    });
    *num += 1;
    *extended += 1;
}
```
*/
const _EXTEND_LIFETIME_CANNOT_BE_MISUSED: () = ();
