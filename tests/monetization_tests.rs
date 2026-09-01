use domain::monetization::{
    validate_double_entry_balance, Currency, LedgerEntry, LedgerEntryType, Money,
};

#[test]
fn test_money_precise_arithmetic() {
    let m1 = Money::new(100_000, Currency::Irr);
    let m2 = Money::new(50_000, Currency::Irr);

    let sum = m1.add(&m2).unwrap();
    assert_eq!(sum.amount, 150_000);
    assert_eq!(sum.currency, Currency::Irr);

    let diff = m1.subtract(&m2).unwrap();
    assert_eq!(diff.amount, 50_000);
}

#[test]
fn test_money_currency_mismatch_rejected() {
    let irr = Money::new(100_000, Currency::Irr);
    let usd = Money::new(10, Currency::Usd);

    assert!(irr.add(&usd).is_err());
    assert!(irr.subtract(&usd).is_err());
}

#[test]
fn test_double_entry_ledger_balanced_invariant() {
    let balanced_entries = vec![
        LedgerEntry {
            account: "ASSETS:BANK".to_string(),
            entry_type: LedgerEntryType::Debit,
            amount: 500_000,
        },
        LedgerEntry {
            account: "REVENUE:SUBSCRIPTION".to_string(),
            entry_type: LedgerEntryType::Credit,
            amount: 500_000,
        },
    ];
    assert!(validate_double_entry_balance(&balanced_entries).is_ok());

    let unbalanced_entries = vec![
        LedgerEntry {
            account: "ASSETS:BANK".to_string(),
            entry_type: LedgerEntryType::Debit,
            amount: 500_000,
        },
        LedgerEntry {
            account: "REVENUE:SUBSCRIPTION".to_string(),
            entry_type: LedgerEntryType::Credit,
            amount: 400_000,
        },
    ];
    assert!(validate_double_entry_balance(&unbalanced_entries).is_err());
}