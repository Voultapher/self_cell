use self_cell::{self_cell, MutBorrow};

#[allow(dead_code)]
type Dependent<'a> = &'a str;

self_cell!(
    struct SelfCell {
        owner: String,

        #[covariant, async_builder]
        dependent: Dependent,
    }

    impl {Debug}
);

#[cfg(test)]
const OWNER_STR: &str = "some longer string xxx with even more chars";

#[test]
fn async_self_cell() {
    let cell = smol::block_on(async {
        let owner = OWNER_STR.to_string();
        let capture_idx = 33;
        SelfCell::new(owner, async |owner| &owner[capture_idx..]).await
    });

    assert_eq!(cell.borrow_dependent(), &"more chars");
}

#[test]
fn async_self_cell_try_new() {
    let cell = smol::block_on(async {
        let owner = OWNER_STR.to_string();
        let capture_idx = 33;
        SelfCell::try_new(owner, async |owner| Ok::<_, ()>(&owner[capture_idx..])).await
    })
    .unwrap();

    assert_eq!(cell.borrow_dependent(), &"more chars");
}

#[test]
fn async_self_cell_try_new_or_recover() {
    let cell = smol::block_on(async {
        let owner = OWNER_STR.to_string();
        let capture_idx = 33;
        SelfCell::try_new_or_recover(owner, async |owner| Ok::<_, ()>(&owner[capture_idx..])).await
    })
    .unwrap();

    assert_eq!(cell.borrow_dependent(), &"more chars");
}

#[test]
fn async_self_cell_recover() {
    let (owner, err) = smol::block_on(async {
        let owner = OWNER_STR.to_string();
        SelfCell::try_new_or_recover(owner, async |owner| Err(owner.len())).await
    })
    .unwrap_err();

    assert_eq!(err, OWNER_STR.len());
    assert_eq!(owner, OWNER_STR);
}

#[test]
fn async_self_cell_with_sleep() {
    smol::block_on(async {
        let owner = OWNER_STR.to_string();
        let capture_idx = 33;
        let cell = SelfCell::new(owner, async |owner| {
            smol::Timer::after(std::time::Duration::from_millis(100)).await;
            &owner[capture_idx..]
        })
        .await;
        assert_eq!(cell.borrow_dependent(), &"more chars");
    });
}

#[test]
fn async_self_cell_with_mut_borrow() {
    type MutStringRef<'a> = &'a mut String;

    self_cell!(
        struct MutStringCell {
            owner: MutBorrow<String>,

            #[covariant, async_builder]
            dependent: MutStringRef,
        }
    );

    let mut cell = smol::block_on(async {
        let owner = MutBorrow::new(OWNER_STR.to_string());
        MutStringCell::new(owner, async |owner| owner.borrow_mut()).await
    });

    assert_eq!(**cell.borrow_dependent(), OWNER_STR);
    cell.with_dependent_mut(|_owner, dependent| {
        dependent.pop();
    });
    assert_eq!(**cell.borrow_dependent(), OWNER_STR[..OWNER_STR.len() - 1]);
}
