pub fn dedent(s: &str) -> String {
    // find the min whitespace count
    let min_indent = s
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or_default();

    s.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| if line.len() >= min_indent { &line[min_indent..] } else { line })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedent_without_interpolation() {
        let input = "\tfirst\n\t\t second\n\t\t third";
        let expected = "first\n\t second\n\t third";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_interpolation() {
        let line = "line";
        let second = "second";
        let input = format!("\t\tfirst {line}\n\t\t {second}\n\t\t third");
        let expected = "first line\n second\n third";
        let result = dedent(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_blank_first_line() {
        let input =
            "\n\tSome text that I might want to indent:\n\t\t* reasons\n\t\t* fun\n\tThat's all.";
        let expected = "Some text that I might want to indent:\n\t* reasons\n\t* fun\nThat's all.";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_multiple_blank_first_lines() {
        let input = "\n\n\t first\n\t second\n\t third";
        let expected = "first\nsecond\nthird";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_same_number_of_spaces() {
        let input = "\n\t first\n\t\t second\n\t\t\t third\n";
        let expected = "first\n second\n\t third";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_single_line_input() {
        let input = "A single line of input.";
        let expected = "A single line of input.";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_single_line_with_newline() {
        let input = "\n\tA single line of input.\n";
        let expected = "A single line of input.";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_escaped_characters() {
        let input = "\\n\\t";
        let expected = "\\n\\t";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_spaces_for_indentation() {
        let input = "  first\n    second\n      third\n";
        let expected = "first\n  second\n    third";
        let result = dedent(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dedent_with_tabs_for_indentation() {
        let input = "\tfirst\n\t\tsecond\n\t\t\tthird\n";
        let expected = "first\n\tsecond\n\t\tthird";
        let result = dedent(input);
        assert_eq!(result, expected);
    }
}
