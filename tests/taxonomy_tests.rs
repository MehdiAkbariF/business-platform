use domain::taxonomy::{normalize_taxonomy_term, TaxonomySlug};

#[test]
fn test_taxonomy_slug_parsing() {
    assert!(TaxonomySlug::parse("car-repair-services").is_ok());
    assert!(TaxonomySlug::parse("").is_err());
    assert!(TaxonomySlug::parse("invalid slug with spaces").is_err());
}

#[test]
fn test_persian_taxonomy_term_normalization() {
    let raw = "  رستوران و کافی‌شاپ  ";
    let normalized = normalize_taxonomy_term(raw);
    assert_eq!(normalized, "رستوران و کافی‌شاپ");

    // Arabic Yeh and Kaf normalization
    let arabic = "تعميرات اليكترونيك";
    let persian = normalize_taxonomy_term(arabic);
    assert_eq!(persian, "تعمیرات الیکترونیک");
}