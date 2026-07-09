const E = { B: 1 };

namespace N {
    declare enum E {
        A = E.B,
        B = 1,
    }
}
