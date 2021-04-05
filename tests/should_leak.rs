use self_cell::self_cell;

#[test]
#[ignore]
fn panic_in_from_owner() {
    // Check that no UB happens when from owner panics.

    struct Dependent<'a>(&'a String);

    impl<'a> From<&'a String> for Dependent<'a> {
        fn from(_: &'a String) -> Self {
            panic!()
        }
    }

    self_cell!(PanicCell, {}, from, String, Dependent, covariant);

    let result = std::panic::catch_unwind(|| {
        let _panic_cell = PanicCell::new("some longer string it is".into());
    });
    assert!(result.is_err());
}
