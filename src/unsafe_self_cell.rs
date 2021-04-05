use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::ptr::drop_in_place;

extern crate alloc;

use alloc::alloc::{dealloc, Layout};

// Self referential structs are currently not supported with safe vanilla Rust.
// The only reasonable safe alternative is to expect the user to juggle 2 separate
// data structures which is a mess. The library solution rental is both no longer
// maintained and really heavy to compile. So begrudgingly I rolled my own version.
// These are some of the core invariants we require for this to be safe to use.
//
// 1. owner is initialized when UnsafeSelfCell is constructed.
// 2. owner is NEVER changed again.
// 3. The pointer to owner and dependent never changes, even when moved.
// 4. The only access to owner and dependent is as immutable reference.
// 5. owner lives longer than dependent.

// #[repr(C)] // TODO is this necessary?
pub struct JoinedCell<Owner, Dependent> {
    pub owner: Owner,
    pub dependent: Dependent,
}

// Library controlled struct that marks all accesses as unsafe.
// Because the macro generated struct impl can be extended, could be unsafe.
pub struct UnsafeSelfCell<Owner: 'static, DependentStatic: 'static> {
    // It's crucial these members are private.
    // *mut even though *const might be enough to mark this type itself
    // as invariant(covariance).
    joined_void_ptr: *mut u8,

    owner_marker: PhantomData<Owner>,
    // DependentStatic is only used to correctly derive Send and Sync.
    dependent_marker: PhantomData<DependentStatic>,
}

impl<Owner, DependentStatic> UnsafeSelfCell<Owner, DependentStatic> {
    pub unsafe fn new(
        joined_void_ptr: *mut u8,
        // init_dependent: for<'a> fn(&'a Owner, *mut Dependent),
    ) -> Self
// where
        // Owner
        // for<'a> Dependent: From<&'a Owner> + core::fmt::Debug,
    {
        // let layout = Layout::from_size_align_unchecked(
        //     size_of::<JoinedCell<Owner, DependentStatic>>(),
        //     align_of::<JoinedCell<Owner, DependentStatic>>(),
        // );

        // let joined_void_ptr = alloc(layout);

        // let joined_ptr =
        //     transmute::<*mut u8, *mut JoinedCell<Owner, DependentStatic>>(joined_void_ptr);

        // // Move owner into newly allocated space.
        // addr_of_mut!((*joined_ptr).owner).write(owner);

        // // Initialize dependent with owner reference in final place.
        // // init_dependent(&(*joined_ptr).owner, addr_of_mut!((*joined_ptr).dependent));
        // // init_dependent(&(*joined_ptr).owner);
        // // addr_of_mut!((*joined_ptr).dependent).write((&(*joined_ptr).owner).into());

        // // // #[repr(C)]
        // // struct JoinedCellMaybeUninit<Owner, Dependent> {
        // //     owner: Owner,
        // //     dependent: MaybeUninit<Dependent>,
        // // }

        // // // We move owner to the heap final location.
        // // // Then use that value to inplace init the dependent.
        // // let mut joined_box_raw = Box::new(JoinedCellMaybeUninit::<Owner, Dependent> {
        // //     owner,
        // //     dependent: MaybeUninit::uninit().assume_init(),
        // // });

        // // // We know the heap allocated owner will outlive this function,
        // // // so this transmute is safe.
        // // let owner_ref: &'a Owner = transmute::<_, &'a Owner>(&joined_box_raw.owner);

        // // dbg!(owner_ref);

        // // joined_box_raw
        // //     .dependent
        // //     .as_mut_ptr()
        // //     .write(owner_ref.into());

        // // // Type erase pointer to store inside struct.
        // // let joined_void_ptr = transmute::<*mut JoinedCellMaybeUninit<Owner, Dependent>, *mut u8>(
        // //     Box::into_raw(joined_box_raw),
        // // );

        // // let joined_ptr = transmute::<*mut u8, *mut JoinedCell<Owner, Dependent>>(joined_void_ptr);

        // // dbg!(&(*joined_ptr).dependent);

        Self {
            joined_void_ptr,
            owner_marker: PhantomData,
            dependent_marker: PhantomData,
        }
    }

    pub unsafe fn borrow_owner<'a, Dependent>(&'a self) -> &'a Owner {
        let joined_ptr =
            transmute::<*mut u8, *mut JoinedCell<Owner, Dependent>>(self.joined_void_ptr);

        &(*joined_ptr).owner
    }

    pub unsafe fn borrow_dependent<'a, Dependent>(&'a self) -> &'a Dependent {
        let joined_ptr =
            transmute::<*mut u8, *mut JoinedCell<Owner, Dependent>>(self.joined_void_ptr);

        &(*joined_ptr).dependent
    }

    // Any subsequent use of this struct other than dropping it is UB.
    pub unsafe fn drop_joined<Dependent>(&mut self) {
        let joined_ptr =
            transmute::<*mut u8, *mut JoinedCell<Owner, Dependent>>(self.joined_void_ptr);

        drop_in_place(joined_ptr);

        let layout = Layout::from_size_align_unchecked(
            size_of::<JoinedCell<Owner, Dependent>>(),
            align_of::<JoinedCell<Owner, Dependent>>(),
        );

        dealloc(self.joined_void_ptr, layout);
    }
}

unsafe impl<Owner, DependentStatic> Send for UnsafeSelfCell<Owner, DependentStatic>
where
    // Only derive Send if Owner and DependentStatic is also Send
    Owner: Send,
    DependentStatic: Send,
{
}

unsafe impl<Owner, DependentStatic> Sync for UnsafeSelfCell<Owner, DependentStatic>
where
    // Only derive Sync if Owner and DependentStatic is also Sync
    Owner: Sync,
    DependentStatic: Sync,
{
}
