use crate::validation::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    pub raw_query: String,
    pub normalized_query: String,
    pub tokens: Vec<String>,
}

pub fn normalize_persian_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let trimmed = input.trim();

    for ch in trimmed.chars() {
        match ch {
            // Arabic to Persian Yeh & Kaf
            'ي' | 'ى' | 'ئ' => normalized.push('ی'),
            'ك' => normalized.push('ک'),
            'ة' => normalized.push('ه'),
            'ؤ' => normalized.push('و'),
            // Persian/Arabic numbers to ASCII
            '۰' | '٠' => normalized.push('0'),
            '۱' | '١' => normalized.push('1'),
            '۲' | '٢' => normalized.push('2'),
            '۳' | '٣' => normalized.push('3'),
            '۴' | '٤' => normalized.push('4'),
            '۵' | '٥' => normalized.push('5'),
            '۶' | '٦' => normalized.push('6'),
            '۷' | '٧' => normalized.push('7'),
            '۸' | '٨' => normalized.push('8'),
            '۹' | '٩' => normalized.push('9'),
            // Zero-width non-joiner (half-space) -> space for indexing
            '\u{200c}' | '\u{200f}' | '\u{200e}' => normalized.push(' '),
            // Arabic diacritics (Harakat) removal
            '\u{064B}'..='\u{065F}' | '\u{0670}' => {},
            // Punctuation to space
            '!' | '?' | '؟' | '،' | ',' | '.' | '-' | '_' | ':' | ';' | '؛' | '/' | '\\' | '(' | ')' | '[' | ']' => {
                normalized.push(' ');
            }
            c => normalized.push(c.to_ascii_lowercase()),
        }
    }

    // Collapse multiple whitespaces
    normalized
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn parse_search_query(raw_query: &str) -> Result<ParsedQuery, ValidationError> {
    if raw_query.len() > 255 {
        return Err(ValidationError::TooLong("q".to_string(), 255));
    }

    let normalized = normalize_persian_text(raw_query);
    let tokens: Vec<String> = normalized
        .split_whitespace()
        .filter(|t| t.len() >= 2)
        .map(|s| s.to_string())
        .collect();

    Ok(ParsedQuery {
        raw_query: raw_query.to_string(),
        normalized_query: normalized,
        tokens,
    })
}

pub fn validate_coordinates(lat: f64, lon: f64) -> Result<(), ValidationError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(ValidationError::InvalidFormat("lat".to_string(), "Latitude must be between -90 and 90".to_string()));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(ValidationError::InvalidFormat("lon".to_string(), "Longitude must be between -180 and 180".to_string()));
    }
    Ok(())
}