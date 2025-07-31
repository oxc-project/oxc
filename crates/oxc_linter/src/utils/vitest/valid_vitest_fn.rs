use std::borrow::Cow;

pub fn is_valid_test_fn_call(members: &[Cow<str>]) -> bool {
    let mut members = members.iter().map(AsRef::as_ref);

    let Some(first) = members.next() else {
        return false;
    };

    let second = members.next();

    match first {
        // Simple root functions that should not have any modifiers
        "beforeAll" | "afterAll" | "beforeEach" | "afterEach" => second.is_none(),

        // bench function with comprehensive modifier support (Vitest-specific)
        "bench" => {
            validate_modifiers(second, &mut members, &[
                "only", "skip", "todo", "runIf", "skipIf"
            ])
        }

        // describe variants that can have modifiers (Jest + Vitest)
        "describe" | "fdescribe" | "xdescribe" | "suite" => {
            validate_modifiers(second, &mut members, &[
                "only", "skip", "each", "todo", "concurrent", "sequential", "shuffle", "runIf", "skipIf"
            ])
        }

        // it/test variants that can have modifiers (Jest + Vitest)
        "it" | "test" | "fit" | "xit" | "xtest" => {
            validate_modifiers(second, &mut members, &[
                "only", "skip", "each", "concurrent", "failing", "todo",
                "sequential", "fails", "extend", "runIf", "skipIf"
            ])
        }

        _ => false,
    }
}

fn validate_modifiers<'a>(
    second: Option<&'a str>,
    members: &mut impl Iterator<Item = &'a str>,
    valid_modifiers: &[&str],
) -> bool {
    let Some(second) = second else { return true };

    std::iter::once(second)
        .chain(members)
        .all(|member| valid_modifiers.contains(&member))
}
