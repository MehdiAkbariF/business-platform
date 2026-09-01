use domain::business::{Business, BusinessSlug, BusinessStatus};
use domain::membership::{BusinessMembership, MembershipRole};
use domain::monetization::{validate_double_entry_balance, Currency, LedgerEntry, LedgerEntryType, Money};
use domain::ranking::{calculate_distance_score, calculate_freshness_score};
use domain::search::{normalize_persian_text, parse_search_query};
use domain::security::{sanitize_storage_key, validate_public_destination_url};
use domain::seo::{StructuredAddress, StructuredDataJsonLd, StructuredGeo};
use shared::{BusinessId, MembershipId, UserId};

#[test]
fn test_e2e_business_lifecycle_and_state_machine_invariants() {
    let user_id = UserId::new();
    let business_id = BusinessId::new();
    let slug = BusinessSlug::parse("cafe-shik").unwrap();

    let mut business = Business::new(
        business_id,
        slug,
        "کافه شیک".to_string(),
        Some("کافه و رستوران دنج".to_string()),
        Some("توضیحات کامل درباره کافه شیک".to_string()),
        user_id,
    );

    let owner_membership = BusinessMembership::new(
        MembershipId::new(),
        business_id,
        user_id,
        MembershipRole::Owner,
    );

    // Invariant: Initial status is Draft
    assert_eq!(business.status, BusinessStatus::Draft);
    assert_eq!(owner_membership.role, MembershipRole::Owner);
    assert!(owner_membership.can_edit_profile());
    assert!(owner_membership.can_submit());
    assert!(owner_membership.can_archive());

    // Invariant: Submission requires primary location and primary category
    assert!(business.submit_for_review(false, false).is_err());
    assert!(business.submit_for_review(true, false).is_err());
    assert!(business.submit_for_review(false, true).is_err());
    assert!(business.submit_for_review(true, true).is_ok());
    assert_eq!(business.status, BusinessStatus::PendingReview);

    // Invariant: Admin moderation allows PUBLISHED or REJECTED
    assert!(business.can_transition_to(BusinessStatus::Published));
    assert!(business.can_transition_to(BusinessStatus::Rejected));
    assert!(business.can_transition_to(BusinessStatus::Archived));
}

#[test]
fn test_e2e_monetization_double_entry_ledger_and_money_arithmetic() {
    let price1 = Money::new(500_000, Currency::Irr);
    let price2 = Money::new(250_000, Currency::Irr);

    let total = price1.add(&price2).unwrap();
    assert_eq!(total.amount, 750_000);

    // Strict currency mismatch prevention
    let usd = Money::new(10, Currency::Usd);
    assert!(price1.add(&usd).is_err());

    // Balanced double-entry accounting ledger entries
    let balanced_ledger = vec![
        LedgerEntry {
            account: "ASSETS:BANK_ACCOUNT".to_string(),
            entry_type: LedgerEntryType::Debit,
            amount: 750_000,
        },
        LedgerEntry {
            account: "REVENUE:SUBSCRIPTION_PLANS".to_string(),
            entry_type: LedgerEntryType::Credit,
            amount: 750_000,
        },
    ];
    assert!(validate_double_entry_balance(&balanced_ledger).is_ok());

    // Unbalanced ledger entries must be strictly rejected
    let unbalanced_ledger = vec![
        LedgerEntry {
            account: "ASSETS:BANK_ACCOUNT".to_string(),
            entry_type: LedgerEntryType::Debit,
            amount: 750_000,
        },
        LedgerEntry {
            account: "REVENUE:SUBSCRIPTION_PLANS".to_string(),
            entry_type: LedgerEntryType::Credit,
            amount: 700_000,
        },
    ];
    assert!(validate_double_entry_balance(&unbalanced_ledger).is_err());
}

#[test]
fn test_e2e_search_and_ranking_determinism() {
    // 1. Text normalization
    let raw_search = "  رِستورانِ سنتي ايران  ";
    let normalized = normalize_persian_text(raw_search);
    assert_eq!(normalized, "رستوران سنتی ایران");

    let parsed = parse_search_query(raw_search).unwrap();
    assert!(parsed.tokens.contains(&"رستوران".to_string()));
    assert!(parsed.tokens.contains(&"سنتی".to_string()));
    assert!(parsed.tokens.contains(&"ایران".to_string()));

    // 2. Ranking scores
    let dist_score = calculate_distance_score(Some(1200.0), 10.0);
    assert!(dist_score > 0.8 && dist_score < 1.0);

    let freshness = calculate_freshness_score(chrono::Utc::now(), 45.0);
    assert_eq!(freshness, 1.0);
}

#[test]
fn test_e2e_security_ssrf_and_path_traversal_guards() {
    // SSRF Guard
    assert!(validate_public_destination_url("https://platform.com/api").is_ok());
    assert!(validate_public_destination_url("http://127.0.0.1/admin").is_err());
    assert!(validate_public_destination_url("http://169.254.169.254/latest/meta-data").is_err());

    // Path Traversal Guard
    assert!(sanitize_storage_key("businesses/0191/logo.jpg").is_ok());
    assert!(sanitize_storage_key("../../../etc/shadow").is_err());
    assert!(sanitize_storage_key("/root/.ssh/authorized_keys").is_err());
}

#[test]
fn test_e2e_seo_schema_and_structured_data() {
    let json_ld = StructuredDataJsonLd {
        context: "https://schema.org".to_string(),
        schema_type: "LocalBusiness".to_string(),
        name: "رستوران ارکیده".to_string(),
        description: Some("رستوران ایرانی و فرنگی".to_string()),
        url: "https://platform.com/businesses/orkideh".to_string(),
        telephone: Some("02188888888".to_string()),
        address: Some(StructuredAddress {
            street_address: "میدان آرژانتین، نبش خیابان الوند".to_string(),
            address_locality: "تهران".to_string(),
            address_region: "تهران".to_string(),
            address_country: "IR".to_string(),
        }),
        geo: Some(StructuredGeo {
            latitude: 35.7575,
            longitude: 51.4111,
        }),
    };

    assert_eq!(json_ld.schema_type, "LocalBusiness");
    assert!(json_ld.address.is_some());
    assert!(json_ld.geo.is_some());
}