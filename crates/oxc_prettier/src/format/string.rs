fn get_preferred_quote(raw: &str, prefer_single_quote: bool) -> char {
    let (preferred_quote_char, alternate_quote_char) =
        if prefer_single_quote { ('\'', '"') } else { ('"', '\'') };

    let mut preferred_quote_count = 0;
    let mut alternate_quote_count = 0;

    for character in raw.chars() {
        if character == preferred_quote_char {
            preferred_quote_count += 1;
        } else if character == alternate_quote_char {
            alternate_quote_count += 1;
        }
    }

    if preferred_quote_count > alternate_quote_count {
        alternate_quote_char
    } else {
        preferred_quote_char
    }
}

fn make_string(raw_text: &str, enclosing_quote: char) -> String {
    let other_quote = if enclosing_quote == '"' { '\'' } else { '"' };
    let mut result = String::new();
    result.push(enclosing_quote);

    let mut chars = raw_text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next_char) = chars.peek() {
                    if next_char != other_quote {
                        result.push('\\');
                    }
                    result.push(next_char);
                    chars.next();
                } else {
                    result.push('\\');
                }
            }
            _ if c == enclosing_quote => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result.push(enclosing_quote);
    result
}

pub(super) fn print_string(raw_text: &str, prefer_single_quote: bool) -> String {
    let enclosing_quote = get_preferred_quote(raw_text, prefer_single_quote);
    make_string(raw_text, enclosing_quote)
}
