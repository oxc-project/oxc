#[macro_export]
macro_rules! dummy {
    () => {
        Default::default()
    };

    (panic) => {
        dummy!(@ panic)
    };

    (unreachable) => {
        dummy!(@ unreachable)
    };

    (@ $macro:ident) => {
        $macro!("Unexpected `Dummy` ast node.")
    };
}
