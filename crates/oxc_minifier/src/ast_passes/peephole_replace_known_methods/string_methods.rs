use oxc_ecmascript::ToInt32;

pub(super) struct StringUtils;

impl StringUtils {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub(super) fn evaluate_string_index_of(
        string: &str,
        search_value: Option<&str>,
        from_index: Option<f64>,
    ) -> isize {
        let from_index = from_index.map_or(0, |x| x.to_int_32().max(0)) as usize;
        let Some(search_value) = search_value else {
            return -1;
        };
        let result = string.chars().skip(from_index).collect::<String>().find(search_value);
        result.map(|index| index + from_index).map_or(-1, |index| index as isize)
    }

    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub(super) fn evaluate_string_last_index_of(
        string: &str,
        search_value: Option<&str>,
        from_index: Option<f64>,
    ) -> isize {
        let Some(search_value) = search_value else { return -1 };

        let from_index =
            from_index.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize + search_value.len());

        string
            .chars()
            .take(from_index)
            .collect::<String>()
            .rfind(search_value)
            .map_or(-1, |index| index as isize)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_evaluate_string_index_of() {
        use super::StringUtils;

        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(0.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(1.0)),
            3
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(4.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(4.1)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(0.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(-1.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(-1.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("t"), Some(-1.1)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of(
                "test test test",
                Some("t"),
                Some(-1_073_741_825.0)
            ),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("e"), Some(0.0)),
            1
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("s"), Some(0.0)),
            2
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(4.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(5.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(6.0)),
            10
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(0.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(-1.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("not found"), Some(-1.0)),
            -1
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(-1.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of(
                "test test test",
                Some("test"),
                Some(-1_073_741_825.0)
            ),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("test"), Some(0.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_index_of("test test test", Some("notpresent"), Some(0.0)),
            -1
        );
        assert_eq!(StringUtils::evaluate_string_index_of("test test test", None, Some(0.0)), -1);
    }

    #[test]
    fn test_evaluate_string_last_index_of() {
        use super::StringUtils;
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(15.0)),
            10
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(14.0)),
            10
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(10.0)),
            10
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(9.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(6.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(5.0)),
            5
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(4.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", Some("test"), Some(0.0)),
            0
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of(
                "test test test",
                Some("notpresent"),
                Some(0.0)
            ),
            -1
        );
        assert_eq!(
            StringUtils::evaluate_string_last_index_of("test test test", None, Some(1.0)),
            -1
        );
        assert_eq!(StringUtils::evaluate_string_last_index_of("abcdef", Some("b"), None), 1);
    }
}
