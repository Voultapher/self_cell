mod once_self_cell;

pub trait OnceCellCompatible<T> {
    fn new() -> Self;
    fn get(&self) -> Option<&T>;
    fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T;
    fn take(&mut self) -> Option<T>;
}

// HKT sigh.

pub mod unsync {
    use crate::once_self_cell::DependentInner;
    use once_cell::unsync::OnceCell;

    #[derive(Debug)]
    pub struct UnsyncOnceCell(OnceCell<DependentInner>);

    impl crate::OnceCellCompatible<DependentInner> for UnsyncOnceCell {
        fn new() -> Self {
            UnsyncOnceCell(OnceCell::<DependentInner>::new())
        }
        fn get(&self) -> Option<&DependentInner> {
            self.0.get()
        }
        fn get_or_init<F>(&self, f: F) -> &DependentInner
        where
            F: FnOnce() -> DependentInner,
        {
            self.0.get_or_init(f)
        }
        fn take(&mut self) -> Option<DependentInner> {
            self.0.take()
        }
    }

    pub type OnceSelfCell<Owner> = crate::once_self_cell::OnceSelfCell<Owner, UnsyncOnceCell>;
}

pub mod sync {
    use crate::once_self_cell::DependentInner;
    use once_cell::sync::OnceCell;

    #[derive(Debug)]
    pub struct SyncOnceCell(OnceCell<DependentInner>);

    impl crate::OnceCellCompatible<DependentInner> for SyncOnceCell {
        fn new() -> Self {
            SyncOnceCell(OnceCell::<DependentInner>::new())
        }
        fn get(&self) -> Option<&DependentInner> {
            self.0.get()
        }
        fn get_or_init<F>(&self, f: F) -> &DependentInner
        where
            F: FnOnce() -> DependentInner,
        {
            self.0.get_or_init(f)
        }
        fn take(&mut self) -> Option<DependentInner> {
            self.0.take()
        }
    }

    // A mutable pointer that only gets changed in 2 ways:
    //
    // 1.
    // get_or_init, sync::OnceCell takes care of establishing a happens-before
    // relationship between a potential write and read of the lazy init.
    //
    // 2.
    // drop_dependent_unconditional, might overwrite the OnceCell with it's
    // default empty state. This hinges on OnceCell::take pulling out the
    // value only exactly once even if called concurrently. Which is given,
    // because the Rust type system ensures only exactly one &mut can exist
    // at any time. And a &mut is required for calling drop_dependent_unconditional.
    unsafe impl Send for SyncOnceCell {}
    unsafe impl Sync for SyncOnceCell {}

    pub type OnceSelfCell<Owner> = crate::once_self_cell::OnceSelfCell<Owner, SyncOnceCell>;
}

pub mod custom {
    // User provided OnceCell. Has to implement OnceCellCompatible.
    pub type OnceSelfCell<Owner, DependentCell> =
        crate::once_self_cell::OnceSelfCell<Owner, DependentCell>;
}
