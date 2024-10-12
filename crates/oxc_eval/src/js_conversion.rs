use std::convert::Infallible;

/// Like [`Into`], but conversion logic follows the ECMAScript specification.
pub trait JsInto<T> {
    fn into_js(self) -> T;
}

/// Like [`From`], but conversion logic follows the ECMAScript specification.
pub trait JsFrom<T> {
    #[must_use]
    fn from_js(value: T) -> Self;
}

/// Like [`TryInto`], but conversion logic follows the ECMAScript specification.
pub trait TryJsInto<T> {
    /// The error type returned when the conversion fails.
    type Error;
    fn try_into_js(self) -> Result<T, Self::Error>;
}

/// Like [`TryFrom`], but conversion logic follows the ECMAScript specification.
pub trait TryJsFrom<T>: Sized {
    /// The error type returned when the conversion fails.
    type Error;
    fn try_from_js(value: T) -> Result<Self, Self::Error>;
}

// identity

impl<T> JsFrom<T> for T {
    #[inline]
    fn from_js(value: T) -> Self {
        value
    }
}

// from/into reciprocal impls

impl<T, U> JsInto<T> for U
where
    T: JsFrom<U>,
{
    #[must_use]
    fn into_js(self) -> T {
        T::from_js(self)
    }
}

impl<T, U> TryJsInto<T> for U
where
    T: TryJsFrom<U>,
{
    type Error = T::Error;
    fn try_into_js(self) -> Result<T, Self::Error> {
        T::try_from_js(self)
    }
}

impl<T, U> TryJsFrom<U> for T
where
    T: JsFrom<U>,
{
    type Error = Infallible;

    #[inline]
    fn try_from_js(value: U) -> Result<Self, Self::Error> {
        Ok(Self::from_js(value))
    }
}
