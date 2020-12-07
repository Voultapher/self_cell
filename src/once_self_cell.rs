use core::fmt::{Debug, Error, Formatter};
use core::mem::transmute;

use crate::OnceCellCompatible;

pub type DependentInner = *mut u8;

pub struct OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    // It's crucial these members are private.
    owner_ptr: *mut Owner,

    // Store lifetime dependent stuff as 'void pointers', transmute out as needed.
    dependent_ptr: DependentCell,
}

impl<Owner, DependentCell> OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    pub fn new(owner: Owner) -> Self {
        OnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(owner)),
            dependent_ptr: DependentCell::new(),
        }
    }

    pub fn get_owner<'a>(&'a self) -> &'a Owner {
        // I can't see how moving the reference would be bad, and accessing that reference
        // after self has been moved is impossible too.
        unsafe { &*self.owner_ptr }
    }

    pub fn get_or_init_dependent<'a, Dependent>(
        &'a self,
        make_dependent: impl FnOnce(&'a Owner) -> Dependent,
    ) -> &'a Dependent {
        // Self referential structs are currently not supported with safe vanilla Rust.
        // The only reasonable safe alternative is to expect the user to juggle 2 separate
        // data structures which is a mess. The library solution rental is both no longer
        // maintained and really heavy to compile. So begrudgingly we roll our own version.
        // There are 5 core invariants we require for this to be safe to use.
        //
        // 1. owner is initialized when OnceSelfCell is constructed.
        // 2. owner is NEVER changed again.
        // 3. The pointer to dependent never changes, even when moved.
        // 4. The only access to dependent is as immutable reference.
        // 5. owner lives longer than dependent.
        //
        // Drop also needs adjusting.

        // Store the opaque pointer inside the once_cell if not yet initialized.
        let dependent_void_ptr = self.dependent_ptr.get_or_init(|| {
            // We know owner comes from a pointer and lives longer enough for this ref.
            let owner = unsafe { transmute::<*mut Owner, &'a Owner>(self.owner_ptr) };

            let dependent_ptr = Box::into_raw(Box::new(make_dependent(owner)));

            unsafe { transmute::<*mut Dependent, DependentInner>(dependent_ptr) }
        });

        // In this function we have access to the correct Dependent type and lifetime,
        // so we can turn the pointer back into the concrete pointer.
        let dependent_ptr =
            unsafe { transmute::<DependentInner, *mut Dependent>(*dependent_void_ptr) };

        // Return the dereference of the Dependent type pointer, which we know is initialized
        // because we just called get_or_init.
        unsafe { &*dependent_ptr }
    }

    // This allows users to query whether the dependent has already been initialized.
    pub fn dependent_is_none(&self) -> bool {
        // No need to transmute, we are not looking at the content, which is just a pointer.
        self.dependent_ptr.get().is_none()
    }

    // unsafe because the user has to make sure that Dependent is the same type as used in
    // get_or_init_dependent.
    //
    // Call regardless whether dependent was initialized or not.
    //
    // If this is not called dependent is leaked.
    pub unsafe fn drop_dependent_unconditional<Dependent>(&mut self) {
        // After take the regular drop of OnceCell can take cope by itself.
        if let Some(dependent_void_ptr) = self.dependent_ptr.take() {
            let dependent_ptr = transmute::<DependentInner, *mut Dependent>(dependent_void_ptr);

            let dependent_box = Box::from_raw(dependent_ptr);

            drop(dependent_box);
        }
    }
}

impl<Owner, DependentCell> Drop for OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    fn drop(&mut self) {
        // After drop is run, Rust will recursively try to drop all of the fields of self.
        // So it will automatically clean up the OnceCell.

        unsafe {
            drop(Box::from_raw(self.owner_ptr));
        }
    }
}

// drop_dependent_unconditional takes &mut self, so that's not a thread concern anyway.
// And get_or_init_dependent should be as thread compatible as OnceCell.
// Owner never gets changed after init.
unsafe impl<Owner, DependentCell> Send for OnceSelfCell<Owner, DependentCell>
where
    // Only derive Send if Owner and DependentCell is also Send
    Owner: Debug + Clone + Eq + Send,
    DependentCell: Debug + OnceCellCompatible<DependentInner> + Send,
{
}

unsafe impl<Owner, DependentCell> Sync for OnceSelfCell<Owner, DependentCell>
where
    // Only derive Sync if Owner and DependentCell is also Sync
    Owner: Debug + Clone + Eq + Sync,
    DependentCell: Debug + OnceCellCompatible<DependentInner> + Sync,
{
}

impl<Owner, DependentCell> Clone for OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    fn clone(&self) -> Self {
        // The cloned instance has a non yet initialized dependent.
        OnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(self.get_owner().clone())),
            dependent_ptr: DependentCell::new(),
        }
    }
}

impl<Owner, DependentCell> PartialEq for OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    fn eq(&self, other: &Self) -> bool {
        *self.get_owner() == *other.get_owner()
    }
}

impl<Owner, DependentCell> Eq for OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
}

impl<Owner, DependentCell> Debug for OnceSelfCell<Owner, DependentCell>
where
    Owner: Debug + Clone + Eq,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // We don't know the type of dependent so it's gonna show up as Uninit or pointer.
        // That's not ideal but better than nothing.
        // Users are free to implement a better Debug impl on top, they should already have
        // a function that pulls out the dependent with the correct type.
        write!(
            fmt,
            "OnceSelfCell {{ owner: {:?}, dependent_ptr: {:?} }}",
            self.get_owner(),
            self.dependent_ptr
        )
    }
}
