use crate::unicode::{
    CONNECTOR_PUNCTUATION, DECIMAL_NUMBER, LETTER_NUMBER, LOWERCASE_LETTER, MODIFIER_LETTER,
    NONSPACING_MARK, OTHER_LETTER, SPACE_SEPARATOR, SPACING_MARK, TITLECASE_LETTER,
    UPPERCASE_LETTER,
};

/// This is NOT the same as [`char::is_whitespace`].
///
/// <https://spec.json5.org/#white-space>
pub fn is_json5_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000D}' | '\u{0020}' | '\u{00A0}' | '\u{2028}' | '\u{2029}' | '\u{FEFF}'
    ) || SPACE_SEPARATOR.contains_char(c)
}

/// <https://262.ecma-international.org/5.1/#sec-7.3>
pub fn is_json5_line_terminator(c: char) -> bool {
    matches!(c, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
}

/// <https://262.ecma-international.org/5.1/#sec-7.6>
pub fn is_json5_identifier_start(c: char) -> bool {
    matches!(c, '$' | '_')
        || UPPERCASE_LETTER.contains_char(c)
        || LOWERCASE_LETTER.contains_char(c)
        || TITLECASE_LETTER.contains_char(c)
        || MODIFIER_LETTER.contains_char(c)
        || OTHER_LETTER.contains_char(c)
        || LETTER_NUMBER.contains_char(c)
}

/// <https://262.ecma-international.org/5.1/#sec-7.6>
pub fn is_json5_identifier(c: char) -> bool {
    is_json5_identifier_start(c)
        || matches!(c, '\u{200C}' | '\u{200D}')
        || NONSPACING_MARK.contains_char(c)
        || SPACING_MARK.contains_char(c)
        || DECIMAL_NUMBER.contains_char(c)
        || CONNECTOR_PUNCTUATION.contains_char(c)
}

/// <https://spec.json5.org/#strings>
pub fn escape(delimeter: char, c: char) -> Option<&'static str> {
    match c {
        '"' if delimeter == '"' => Some(r#"\""#),
        '\'' if delimeter == '\'' => Some(r"\'"),
        '\\' => Some(r"\\"),
        '\n' => Some(r"\n"),
        '\r' => Some(r"\r"),
        // '\u{2028}' and '\u{2029}' don't strictly *need* to be escaped, but the spec recommends
        // that they *should* be. <https://spec.json5.org/#separators>
        '\u{2028}' => Some(r"\u2028"),
        '\u{2029}' => Some(r"\u2029"),
        _ => None,
    }
}
