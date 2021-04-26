use core::fmt::{Debug, Error, Formatter};
use core::hash::{Hash, Hasher};
use core::mem::transmute;

use crate::OnceCellCompatible;

// To properly clean up we need to store a function with the right type drop call.
pub type VoidPtr = *mut u8;
pub type DependentInner = VoidPtr;

// Library controlled struct that marks all accesses as unsafe.
// Because the macro generated struct impl can be extended, could be unsafe.
pub struct UnsafeOnceSelfCell<Owner: 'static, DependentCell: OnceCellCompatible<DependentInner>> {
    // It's crucial these members are private.
    owner_ptr: *mut Owner,

    // Store make_dependent function pointer to ensure only one is used.
    // make_dependent_void_ptr: VoidPtr,

    // Store lifetime dependent stuff as 'void pointers', transmute out as needed.
    dependent_cell: DependentCell,
}

impl<Owner, DependentCell> UnsafeOnceSelfCell<Owner, DependentCell>
where
    DependentCell: OnceCellCompatible<DependentInner>,
{
    pub unsafe fn new(owner: Owner) -> Self {
        UnsafeOnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(owner)),
            dependent_cell: DependentCell::new(),
        }
    }

    pub unsafe fn get_owner<'a>(&'a self) -> &'a Owner {
        // I can't see how moving the reference would be bad, and accessing that reference
        // after self has been moved is impossible too.
        &*self.owner_ptr
    }

    pub unsafe fn get_or_init_dependent<'a, Dependent>(
        &'a self,
        make_dependent: fn(&'a Owner) -> Dependent,
    ) -> &'a Dependent {
        // Self referential structs are currently not supported with safe vanilla Rust.
        // The only reasonable safe alternative is to expect the user to juggle 2 separate
        // data structures which is a mess. The library solution rental is both no longer
        // maintained and really heavy to compile. So begrudgingly I rolled my own version.
        // These are some of the core invariants we require for this to be safe to use.
        //
        // 1. owner is initialized when UnsafeOnceSelfCell is constructed.
        // 2. owner is NEVER changed again.
        // 3. The pointer to dependent never changes, even when moved.
        // 4. The only access to dependent is as immutable reference.
        // 5. owner lives longer than dependent.

        let dependent_void_ptr = self.dependent_cell.get_or_init(|| {
            // We know owner comes from a pointer and lives longer enough for this ref.
            let owner = transmute::<*mut Owner, &'a Owner>(self.owner_ptr);

            let dependent_ptr = Box::into_raw(Box::new(make_dependent(owner)));

            transmute::<*mut Dependent, VoidPtr>(dependent_ptr)
        });

        // In this function we have access to the correct Dependent type and lifetime,
        // so we can turn the pointer back into the concrete pointer.
        let dependent_ptr = transmute::<VoidPtr, *mut Dependent>(*dependent_void_ptr);

        // Return the dereference of the Dependent type pointer, which we know is initialized
        // because we just called get_or_init.
        &*dependent_ptr
    }

    pub unsafe fn drop_dependent<Dependent>(&mut self) {
        // IMPORTANT: drop dependent before owner.

        // After calling take the regular drop of OnceCell can cope by itself.
        if let Some(dependent_void_ptr) = self.dependent_cell.take() {
            let dependent_ptr = transmute::<VoidPtr, *mut Dependent>(dependent_void_ptr);

            let dependent_box = Box::from_raw(dependent_ptr);

            drop(dependent_box);
        }
    }

    // This allows users to query whether the dependent has already been initialized.
    pub fn dependent_is_none(&self) -> bool {
        // No need to transmute, we are not looking at the content, which is just a pointer.
        self.dependent_cell.get().is_none()
    }
}

impl<Owner, DependentCell> Drop for UnsafeOnceSelfCell<Owner, DependentCell>
where
    DependentCell: OnceCellCompatible<DependentInner>,
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
unsafe impl<Owner, DependentCell> Send for UnsafeOnceSelfCell<Owner, DependentCell>
where
    // Only derive Send if Owner and DependentCell is also Send
    Owner: Send,
    DependentCell: OnceCellCompatible<DependentInner> + Send,
{
}

unsafe impl<Owner, DependentCell> Sync for UnsafeOnceSelfCell<Owner, DependentCell>
where
    // Only derive Sync if Owner and DependentCell is also Sync
    Owner: Sync,
    DependentCell: OnceCellCompatible<DependentInner> + Sync,
{
}

impl<Owner, DependentCell> Clone for UnsafeOnceSelfCell<Owner, DependentCell>
where
    Owner: Clone,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn clone(&self) -> Self {
        // The cloned instance has a non yet initialized dependent.
        UnsafeOnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(unsafe { self.get_owner() }.clone())),
            dependent_cell: DependentCell::new(),
        }
    }
}

impl<Owner, DependentCell> PartialEq for UnsafeOnceSelfCell<Owner, DependentCell>
where
    Owner: PartialEq,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.get_owner() == *other.get_owner() }
    }
}

impl<Owner, DependentCell> Eq for UnsafeOnceSelfCell<Owner, DependentCell>
where
    Owner: Eq,
    DependentCell: OnceCellCompatible<DependentInner>,
{
}

impl<Owner, DependentCell> Hash for UnsafeOnceSelfCell<Owner, DependentCell>
where
    Owner: Hash,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.get_owner() }.hash(state);
    }
}

impl<Owner, DependentCell> Debug for UnsafeOnceSelfCell<Owner, DependentCell>
where
    Owner: Debug,
    DependentCell: Debug + OnceCellCompatible<DependentInner>,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // We don't know the type of dependent so it's gonna show up as Uninit or pointer.
        // That's not ideal but better than nothing.
        // Users are free to implement a better Debug impl on top, they should already have
        // a function that pulls out the dependent with the correct type.
        write!(
            fmt,
            "UnsafeOnceSelfCell {{ owner: {:?}, dependent_cell: {:?} }}",
            unsafe { self.get_owner() },
            self.dependent_cell
        )
    }
}
