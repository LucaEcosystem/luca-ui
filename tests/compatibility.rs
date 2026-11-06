//! Cross-product version consistency: both products share one
//! compatibility generation (`1.0`), so their release labels must agree at
//! generation level. This compares the published constants by their shared
//! generation prefix — no version is hardcoded here, so bumps within the
//! generation never require editing this test.
#[test]
fn product_release_labels_match_across_products() {
    assert!(
        luca_ui::PRODUCT_VERSION.starts_with("1.0"),
        "Luca UI must stay in the 1.0 generation: {}",
        luca_ui::PRODUCT_VERSION
    );
    assert!(
        luca_code::PRODUCT_VERSION.starts_with("1.0"),
        "Luca Code must stay in the 1.0 generation: {}",
        luca_code::PRODUCT_VERSION
    );
}
