/// Returns true if the given array of strings represents a valid Vitest function call
/// with optional modifiers, such as `test.todo.only`.
pub fn is_valid_vitest_call<T: AsRef<str>>(members: &[T]) -> bool {
    match members[0].as_ref() {
        "afterAll" | "afterEach" | "beforeAll" | "beforeEach" => members.len() == 1,
        "bench" => is_valid_bench_chain(&members[1..]),
        "describe" | "suite" => is_valid_describe_chain(&members[1..]),
        "it" | "test" => is_valid_it_chain(&members[1..]),
        _ => false,
    }
}

/// Check for duplicate modifiers. This has quadratic complexity, but since we only
/// have a very small number of modifiers, this is fine.
fn has_duplicates<T: AsRef<str>>(modifiers: &[T]) -> bool {
    let len = modifiers.len();
    for i in 0..len {
        for j in (i + 1)..len {
            if modifiers[i].as_ref() == modifiers[j].as_ref() {
                return true;
            }
        }
    }
    false
}

fn is_valid_bench_chain<T: AsRef<str>>(modifiers: &[T]) -> bool {
    for modifier in modifiers {
        if !matches!(modifier.as_ref(), "only" | "runIf" | "skip" | "skipIf" | "todo") {
            return false;
        }
    }

    !has_duplicates(modifiers)
}

fn is_valid_describe_chain<T: AsRef<str>>(modifiers: &[T]) -> bool {
    for (i, modifier) in modifiers.iter().enumerate() {
        match modifier.as_ref() {
            "each" | "for" => {
                // each must be at the end
                if i != modifiers.len() - 1 {
                    return false;
                }
            }
            "concurrent" | "only" | "runIf" | "sequential" | "shuffle" | "skip" | "skipIf"
            | "todo" => {}
            _ => return false,
        }
    }

    !has_duplicates(modifiers)
}

fn is_valid_it_chain<T: AsRef<str>>(modifiers: &[T]) -> bool {
    for (i, modifier) in modifiers.iter().enumerate() {
        match modifier.as_ref() {
            "each" => {
                // each must be at the end
                if i != modifiers.len() - 1 {
                    return false;
                }
            }
            "extend" => {
                // extend must be at the beginning (first modifier after it/test)
                if i != 0 {
                    return false;
                }
            }
            "concurrent" | "fails" | "only" | "runIf" | "sequential" | "skip" | "skipIf"
            | "todo" => {}
            _ => return false,
        }
    }

    !has_duplicates(modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_base_functions() {
        assert!(is_valid_vitest_call(&["afterAll"]));
        assert!(is_valid_vitest_call(&["afterEach"]));
        assert!(is_valid_vitest_call(&["beforeAll"]));
        assert!(is_valid_vitest_call(&["beforeEach"]));
        assert!(!is_valid_vitest_call(&["fdescribe"]));
        assert!(!is_valid_vitest_call(&["fit"]));

        // These should not accept modifiers
        assert!(!is_valid_vitest_call(&["afterAll", "only"]));
        assert!(!is_valid_vitest_call(&["beforeEach", "skip"]));
    }

    // These are not supported in Vitest, as far as I can tell.
    #[test]
    fn test_x_functions() {
        assert!(!is_valid_vitest_call(&["xdescribe"]));
        assert!(!is_valid_vitest_call(&["xdescribe", "each"]));
        assert!(!is_valid_vitest_call(&["xit"]));
        assert!(!is_valid_vitest_call(&["xit", "each"]));
        assert!(!is_valid_vitest_call(&["xtest"]));
        assert!(!is_valid_vitest_call(&["xtest", "each"]));
        assert!(!is_valid_vitest_call(&["xdescribe", "only"]));
        assert!(!is_valid_vitest_call(&["xit", "skip"]));
    }

    #[test]
    fn test_bench() {
        assert!(is_valid_vitest_call(&["bench"]));
        assert!(is_valid_vitest_call(&["bench", "only"]));
        assert!(is_valid_vitest_call(&["bench", "skip"]));
        assert!(is_valid_vitest_call(&["bench", "todo"]));
        assert!(is_valid_vitest_call(&["bench", "runIf"]));
        assert!(is_valid_vitest_call(&["bench", "skipIf"]));
        assert!(is_valid_vitest_call(&["bench", "only", "skip"]));
        assert!(is_valid_vitest_call(&["bench", "only", "skip", "todo"]));

        // Invalid: concurrent is not a bench modifier
        assert!(!is_valid_vitest_call(&["bench", "concurrent"]));
        // Invalid: duplicates
        assert!(!is_valid_vitest_call(&["bench", "only", "only"]));
    }

    #[test]
    fn test_describe_and_suite() {
        assert!(is_valid_vitest_call(&["describe"]));
        assert!(is_valid_vitest_call(&["suite"]));
        assert!(is_valid_vitest_call(&["describe", "concurrent"]));
        assert!(is_valid_vitest_call(&["describe", "each"]));
        assert!(is_valid_vitest_call(&["describe", "only"]));
        assert!(is_valid_vitest_call(&["describe", "for"]));
        assert!(is_valid_vitest_call(&["describe", "concurrent", "each"]));
        assert!(is_valid_vitest_call(&["describe", "only", "concurrent", "each"]));

        // each must be at the end
        assert!(!is_valid_vitest_call(&["describe", "each", "only"]));

        // Invalid: fails is not a describe modifier
        assert!(!is_valid_vitest_call(&["describe", "fails"]));
        // Invalid: duplicates
        assert!(!is_valid_vitest_call(&["describe", "only", "only"]));
    }

    #[test]
    fn test_it_and_test() {
        assert!(is_valid_vitest_call(&["it"]));
        assert!(is_valid_vitest_call(&["test"]));
        assert!(is_valid_vitest_call(&["it", "concurrent"]));
        assert!(is_valid_vitest_call(&["it", "each"]));
        assert!(is_valid_vitest_call(&["it", "extend"]));
        assert!(is_valid_vitest_call(&["it", "fails"]));
        assert!(is_valid_vitest_call(&["it", "only"]));
        assert!(is_valid_vitest_call(&["it", "extend", "concurrent"]));
        assert!(is_valid_vitest_call(&["it", "extend", "fails", "each"]));

        // extend must be first
        assert!(!is_valid_vitest_call(&["it", "concurrent", "extend"]));
        // each must be last
        assert!(!is_valid_vitest_call(&["it", "each", "only"]));
        // Invalid: duplicates
        assert!(!is_valid_vitest_call(&["it", "only", "only"]));
        // very long
        assert!(is_valid_vitest_call(&[
            "test",
            "extend",
            "concurrent",
            "fails",
            "only",
            "sequential",
            "todo",
            "each"
        ]));
    }

    #[test]
    fn test_invalid_base_functions() {
        assert!(!is_valid_vitest_call(&["unknown"]));
        assert!(!is_valid_vitest_call(&["foo"]));
        assert!(!is_valid_vitest_call(&["console"]));
    }
}
