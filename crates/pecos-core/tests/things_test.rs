use pecos_core::Phase;

#[test]
fn test_default_phase() {
    assert_eq!(Phase::default(), Phase::PlusOne);
}
