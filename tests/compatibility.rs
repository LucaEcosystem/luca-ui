//! Cross-product version consistency: both products share one
//! compatibility generation, so their release labels must agree. This
//! compares the published constants against each other — no version is
//! hardcoded here, so bumps never require editing this test.
#[test]
fn product_release_labels_match_across_products() {
    assert_eq!(luca_ui::PRODUCT_VERSION, luca_code::PRODUCT_VERSION);
}
