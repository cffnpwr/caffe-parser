use unicode_categories::UnicodeCategories;

pub(super) fn is_unicode_punctuation(c: char) -> bool {
    c.is_ascii_punctuation()
        || c.is_punctuation()
        || c.is_punctuation_close()
        || c.is_punctuation_open()
        || c.is_punctuation_final_quote()
        || c.is_punctuation_initial_quote()
        || c.is_punctuation_other()
        || c.is_punctuation_connector()
        || c.is_punctuation_dash()
}
