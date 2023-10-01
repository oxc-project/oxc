pub trait Expect<P, R> {
    fn expect<F>(self, expectation: F) -> Self
    where
        F: FnOnce(P) -> R;
}
