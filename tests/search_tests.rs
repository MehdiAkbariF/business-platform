use domain::search::{normalize_persian_text, parse_search_query, validate_coordinates};

#[test]
fn test_persian_search_normalization_and_diacritics() {
    let raw = "  رِستورانِ  ايتاليايي   شیک‌ كافه  ";
    let normalized = normalize_persian_text(raw);
    assert_eq!(normalized, "رستوران ایتالیایی شیک کافه");
}

#[test]
fn test_persian_numbers_and_half_spaces() {
    let raw = "پیتزا ۲۴ ساعته\u{200c}تهران";
    let normalized = normalize_persian_text(raw);
    assert_eq!(normalized, "پیتزا 24 ساعته تهران");
}

#[test]
fn test_search_query_tokenization() {
    let parsed = parse_search_query("  تعمیرگاه تخصصی بنز و بی ام دبلیو در ونک  ").unwrap();
    assert!(parsed.tokens.contains(&"تعمیرگاه".to_string()));
    assert!(parsed.tokens.contains(&"تخصصی".to_string()));
    assert!(parsed.tokens.contains(&"بنز".to_string()));
    assert!(parsed.tokens.contains(&"ونک".to_string()));
}

#[test]
fn test_geo_coordinate_boundaries() {
    assert!(validate_coordinates(35.6892, 51.3890).is_ok()); // Tehran Coordinates
    assert!(validate_coordinates(-90.0, 180.0).is_ok());
    assert!(validate_coordinates(91.0, 50.0).is_err());
    assert!(validate_coordinates(35.0, -181.0).is_err());
}