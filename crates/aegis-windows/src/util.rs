//! Small shared helpers.

/// Minimal RFC-4180-ish CSV parser: handles quoted fields, escaped `""`,
/// commas/newlines inside quotes, and CRLF. Returns rows of fields.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => in_quotes = false,
                _ => field.push(c),
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => {
                    row.push(std::mem::take(&mut field));
                }
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Map header name → column index (case-insensitive).
pub fn header_index(header: &[String], name: &str) -> Option<usize> {
    header
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_csv() {
        let text = "\"a\",\"b,c\",\"d\"\"e\"\n1,2,3\n";
        let rows = parse_csv(text);
        assert_eq!(rows[0], vec!["a", "b,c", "d\"e"]);
        assert_eq!(rows[1], vec!["1", "2", "3"]);
    }
}
