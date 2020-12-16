use core::any::type_name;
use core::fmt::{Debug, Error, Formatter};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::mem::transmute;

use crate::OnceCellCompatible;

// To properly clean up we need to store a function with the right type drop call.
pub type VoidPtr = *mut u8;
pub type DependentInner = (VoidPtr, fn(VoidPtr));

pub struct OnceSelfCell<
    Owner: 'static,
    DependentStatic: 'static,
    DependentCell: OnceCellCompatible<DependentInner>,
> {
    // It's crucial these members are private.
    owner_ptr: *mut Owner,

    // Store make_dependent function pointer to ensure only one is used.
    make_dependent_void_ptr: VoidPtr,

    // Store lifetime dependent stuff as 'void pointers', transmute out as needed.
    dependent_cell: DependentCell,

    // We need DependentStatic to ensure dependent is transmuted into the same type every time.
    phantom: PhantomData<DependentStatic>,
}

impl<Owner, DependentStatic, DependentCell> OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    DependentCell: OnceCellCompatible<DependentInner>,
{
    pub fn new<'a, Dependent>(owner: Owner, make_dependent: fn(&'a Owner) -> Dependent) -> Self {
        // Arguably this is quite hacky, but with the current set of compilers this gets the job
        // done of preventing accidental UB, while also being optimized out:
        // https://godbolt.org/z/9MMdE7.
        //
        // Should one day rustc decide to implement this in a way that breaks this,
        // it can be adjusted.
        // Maybe by then HKTs are a thing and a lot of this gets moot anyway.
        //
        // Until then rustc 1.38-1.48 all worked as expected.
        assert_eq!(type_name::<DependentStatic>(), type_name::<Dependent>());

        let make_dependent_void_ptr =
            unsafe { transmute::<fn(&'a Owner) -> Dependent, VoidPtr>(make_dependent) };

        OnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(owner)),
            make_dependent_void_ptr,
            dependent_cell: DependentCell::new(),
            phantom: PhantomData,
        }
    }

    pub fn get_owner<'a>(&'a self) -> &'a Owner {
        // I can't see how moving the reference would be bad, and accessing that reference
        // after self has been moved is impossible too.
        unsafe { &*self.owner_ptr }
    }

    pub fn get_or_init_dependent<'a, Dependent>(&'a self) -> &'a Dependent {
        // Arguably this is quite hacky, but with the current set of compilers this gets the job
        // done of preventing accidental UB, while also being optimized out:
        // https://godbolt.org/z/9MMdE7.
        //
        // Should one day rustc decide to implement this in a way that breaks this,
        // it can be adjusted.
        // Maybe by then HKTs are a thing and a lot of this gets moot anyway.
        //
        // Until then rustc 1.38-1.48 all worked as expected.
        assert_eq!(type_name::<DependentStatic>(), type_name::<Dependent>());

        // Self referential structs are currently not supported with safe vanilla Rust.
        // The only reasonable safe alternative is to expect the user to juggle 2 separate
        // data structures which is a mess. The library solution rental is both no longer
        // maintained and really heavy to compile. So begrudgingly I rolled my own version.
        // There are some of the core invariants we require for this to be safe to use.
        //
        // 1. owner is initialized when OnceSelfCell is constructed.
        // 2. owner is NEVER changed again.
        // 3. The pointer to dependent never changes, even when moved.
        // 4. The only access to dependent is as immutable reference.
        // 5. owner lives longer than dependent.

        let (dependent_void_ptr, _) = self.dependent_cell.get_or_init(|| {
            // We know owner comes from a pointer and lives longer enough for this ref.
            let owner = unsafe { transmute::<*mut Owner, &'a Owner>(self.owner_ptr) };

            let make_dependent = unsafe {
                transmute::<VoidPtr, fn(&'a Owner) -> Dependent>(self.make_dependent_void_ptr)
            };

            let dependent_ptr = Box::into_raw(Box::new(make_dependent(owner)));

            let dependent_void_ptr = unsafe { transmute::<*mut Dependent, VoidPtr>(dependent_ptr) };

            // For the sync variant to be correct creating drop_fn has to happen
            // inside the same critical section.
            let drop_fn = |dependent_void_ptr: VoidPtr| {
                // We assume this function is only called with a valid dependent_void_ptr.
                let dependent_ptr =
                    unsafe { transmute::<VoidPtr, *mut Dependent>(dependent_void_ptr) };

                let dependent_box = unsafe { Box::from_raw(dependent_ptr) };

                drop(dependent_box);
            };

            (dependent_void_ptr, drop_fn)
        });

        // In this function we have access to the correct Dependent type and lifetime,
        // so we can turn the pointer back into the concrete pointer.
        let dependent_ptr = unsafe { transmute::<VoidPtr, *mut Dependent>(*dependent_void_ptr) };

        // Return the dereference of the Dependent type pointer, which we know is initialized
        // because we just called get_or_init.
        unsafe { &*dependent_ptr }
    }

    // This allows users to query whether the dependent has already been initialized.
    pub fn dependent_is_none(&self) -> bool {
        // No need to transmute, we are not looking at the content, which is just a pointer.
        self.dependent_cell.get().is_none()
    }
}

impl<Owner, DependentStatic, DependentCell> Drop
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn drop(&mut self) {
        // After drop is run, Rust will recursively try to drop all of the fields of self.
        // So it will automatically clean up the OnceCell.

        // IMPORTANT: drop dependent before owner.

        // After calling take the regular drop of OnceCell can cope by itself.
        if let Some((dependent_void_ptr, drop_fn)) = self.dependent_cell.take() {
            drop_fn(dependent_void_ptr);
        }

        unsafe {
            drop(Box::from_raw(self.owner_ptr));
        }
    }
}

// drop_dependent_unconditional takes &mut self, so that's not a thread concern anyway.
// And get_or_init_dependent should be as thread compatible as OnceCell.
// Owner never gets changed after init.
unsafe impl<Owner, DependentStatic, DependentCell> Send
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    // Only derive Send if Owner and DependentCell is also Send
    Owner: Send,
    DependentCell: OnceCellCompatible<DependentInner> + Send,
{
}

unsafe impl<Owner, DependentStatic, DependentCell> Sync
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    // Only derive Sync if Owner and DependentCell is also Sync
    Owner: Sync,
    DependentCell: OnceCellCompatible<DependentInner> + Sync,
{
}

impl<Owner, DependentStatic, DependentCell> Clone
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    Owner: Clone,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn clone(&self) -> Self {
        // The cloned instance has a non yet initialized dependent.
        OnceSelfCell {
            owner_ptr: Box::into_raw(Box::new(self.get_owner().clone())),
            dependent_cell: DependentCell::new(),
            make_dependent_void_ptr: self.make_dependent_void_ptr,
            phantom: PhantomData,
        }
    }
}

impl<Owner, DependentStatic, DependentCell> PartialEq
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    Owner: PartialEq,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn eq(&self, other: &Self) -> bool {
        *self.get_owner() == *other.get_owner()
    }
}

impl<Owner, DependentStatic, DependentCell> Eq
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    Owner: Eq,
    DependentCell: OnceCellCompatible<DependentInner>,
{
}

impl<Owner, DependentStatic, DependentCell> Hash
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
where
    Owner: Hash,
    DependentCell: OnceCellCompatible<DependentInner>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_owner().hash(state);
    }
}

impl<Owner, DependentStatic, DependentCell> Debug
    for OnceSelfCell<Owner, DependentStatic, DependentCell>
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
            "OnceSelfCell {{ owner: {:?}, dependent_cell: {:?} }}",
            self.get_owner(),
            self.dependent_cell
        )
    }
}
