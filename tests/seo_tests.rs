use domain::seo::{PageMetadata, StructuredAddress, StructuredDataJsonLd, StructuredGeo};

#[test]
fn test_structured_data_json_ld_generation() {
    let json_ld = StructuredDataJsonLd {
        context: "https://schema.org".to_string(),
        schema_type: "LocalBusiness".to_string(),
        name: "کافه نادری".to_string(),
        description: Some("کافه تاریخی در تهران".to_string()),
        url: "https://platform.com/businesses/naderi-cafe".to_string(),
        telephone: Some("02166701030".to_string()),
        address: Some(StructuredAddress {
            street_address: "خیابان جمهوری، پایین‌تر از چهارراه استانبول".to_string(),
            address_locality: "تهران".to_string(),
            address_region: "تهران".to_string(),
            address_country: "IR".to_string(),
        }),
        geo: Some(StructuredGeo {
            latitude: 35.6944,
            longitude: 51.4172,
        }),
    };

    assert_eq!(json_ld.schema_type, "LocalBusiness");
    assert!(json_ld.address.is_some());
    assert!(json_ld.geo.is_some());
}

#[test]
fn test_metadata_canonical_structure() {
    let meta = PageMetadata {
        title: "کافه نادری | تهران".to_string(),
        description: "اطلاعات کافه نادری".to_string(),
        canonical_url: "https://platform.com/businesses/naderi-cafe".to_string(),
        robots: "index, follow".to_string(),
        og_title: "کافه نادری | تهران".to_string(),
        og_description: "اطلاعات کافه نادری".to_string(),
        og_image: None,
        og_type: "business.business".to_string(),
    };

    assert_eq!(meta.robots, "index, follow");
    assert!(meta.canonical_url.starts_with("https://"));
}