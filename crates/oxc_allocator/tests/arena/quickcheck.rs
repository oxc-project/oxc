/// A redefinition/wrapper macro of `quickcheck::quickcheck!` that supports
/// limiting the number of test iterations to one when we are running under
/// MIRI.
#[macro_export]
macro_rules! quickcheck {
    (
        $(
            $(#[$m:meta])*
            fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) -> $ret:ty {
                $($code:tt)*
            }
        )*
    ) => {
        $(
            #[test]
            $(#[$m])*
            fn $fn_name() {
                fn prop($($arg_name: $arg_ty),*) -> $ret {
                    $($code)*
                }

                let mut qc = ::quickcheck::QuickCheck::new();

                // Use the `QUICKCHECK_TESTS` environment variable from
                // compiletime to avoid violating MIRI's isolation by looking at
                // the runtime environment variable.
                let tests = option_env!("QUICKCHECK_TESTS").and_then(|s| s.parse().ok());

                // Limit quickcheck tests to a single iteration under MIRI,
                // since they are otherwise super slow.
                #[cfg(miri)]
                let tests = tests.or(Some(1));

                if let Some(tests) = tests {
                    eprintln!("Executing at most {} quickchecks", tests);
                    qc = qc.tests(tests);
                }

                qc.quickcheck(prop as fn($($arg_ty),*) -> $ret);
            }
        )*
    };
}
