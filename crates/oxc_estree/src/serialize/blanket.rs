use std::borrow::Cow;

use super::{ESTree, Serializer};

/// [`ESTree`] implementation for a `&` reference to any type that implements `ESTree`.
impl<T> ESTree for &T
where
    T: ESTree + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

/// [`ESTree`] implementation for a `&mut` reference to any type that implements `ESTree`.
impl<T> ESTree for &mut T
where
    T: ESTree + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

/// [`ESTree`] implementation for `Option<T>`.
impl<T: ESTree> ESTree for Option<T> {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        if let Some(value) = self {
            value.serialize(serializer);
        } else {
            serializer.buffer_mut().print_str("null");
        }
    }
}

/// [`ESTree`] implementation for a `Cow` wrapping any type that implements `ESTree`.
impl<T> ESTree for Cow<'_, T>
where
    T: ESTree + ToOwned + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

#[cfg(test)]
mod tests {
    use super::super::CompactTSSerializer;
    use super::*;

    #[expect(clippy::needless_borrow)]
    #[test]
    fn serialize_ref() {
        let cases = [(&"foo", r#""foo""#), (&&"bar", r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_mut_ref() {
        let cases = [(&mut "foo", r#""foo""#), (&mut &mut "bar", r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_option() {
        let cases = [(None, "null"), (Some(123.0f64), "123")];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_cow() {
        let cases =
            [(Cow::Borrowed("foo"), r#""foo""#), (Cow::Owned("bar".to_string()), r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
}
