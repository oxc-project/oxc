pub trait Expect<P, R> {
    #[allow(unused)]
    #[must_use]
    fn expect<F>(self, expectation: F) -> Self
    where
        F: FnOnce(P) -> R;
}
