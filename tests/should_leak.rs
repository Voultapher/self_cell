use self_cell::self_cell;

#[test]
#[ignore]
fn panic_in_from_owner() {
    // Check that no UB happens when from owner panics.

    type Dependent<'a> = &'a String;

    self_cell!(
        struct PanicCell {
            owner: String,

            #[covariant]
            dependent: Dependent,
        }
    );

    let result = std::panic::catch_unwind(|| {
        let _panic_cell = PanicCell::new("some longer string it is".into(), |_| panic!());
    });
    assert!(result.is_err());
}
