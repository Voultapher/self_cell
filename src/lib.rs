#![no_std]

pub extern crate alloc;

// use once_cell::sync::OnceCell as Sync_OnceCell;
// use once_cell::unsync::OnceCell as Unsync_OnceCell;

// use crate::unsafe_once_self_cell::DependentInner;

// pub mod unsafe_once_self_cell;

// pub trait OnceCellCompatible<T> {
//     fn new() -> Self;
//     fn get(&self) -> Option<&T>;
//     fn get_or_init<F>(&self, f: F) -> &T
//     where
//         F: FnOnce() -> T;
//     fn take(&mut self) -> Option<T>;
// }

// // HKT sigh.

// #[derive(Debug)]
// pub struct UnsyncOnceCell(Unsync_OnceCell<DependentInner>);

// impl crate::OnceCellCompatible<DependentInner> for UnsyncOnceCell {
//     fn new() -> Self {
//         UnsyncOnceCell(Unsync_OnceCell::new())
//     }
//     fn get(&self) -> Option<&DependentInner> {
//         self.0.get()
//     }
//     fn get_or_init<F>(&self, f: F) -> &DependentInner
//     where
//         F: FnOnce() -> DependentInner,
//     {
//         self.0.get_or_init(f)
//     }
//     fn take(&mut self) -> Option<DependentInner> {
//         self.0.take()
//     }
// }

// #[macro_export]
// macro_rules! unsync_once_self_cell {
//     ($StructName:ident, $Owner:ty, $Dependent:ty $(, $StructMeta:meta)* $(,)?) => {
//         $(#[$StructMeta])*
//         struct $StructName {
//             unsafe_self_cell: $crate::unsafe_once_self_cell::UnsafeOnceSelfCell<
//                 $Owner,
//                 $crate::UnsyncOnceCell,
//             >,
//         }

//         impl $StructName {
//             pub fn new(owner: $Owner) -> Self {
//                 Self {
//                     unsafe_self_cell: unsafe {
//                         $crate::unsafe_once_self_cell::UnsafeOnceSelfCell::new(owner)
//                     },
//                 }
//             }

//             pub fn get_owner<'a>(&'a self) -> &'a $Owner {
//                 unsafe { self.unsafe_self_cell.get_owner() }
//             }

//             pub fn get_or_init_dependent<'a>(&'a self) -> &'a $Dependent {
//                 unsafe {
//                     self.unsafe_self_cell
//                         .get_or_init_dependent(|owner_ref| owner_ref.into())
//                 }
//             }

//             pub fn dependent_is_none(&self) -> bool {
//                 self.unsafe_self_cell.dependent_is_none()
//             }
//         }

//         impl Drop for $StructName {
//             fn drop(&mut self) {
//                 unsafe {
//                     self.unsafe_self_cell.drop_dependent::<$Dependent>();
//                 }
//             }
//         }
//     };
// }

// #[derive(Debug)]
// pub struct SyncOnceCell(Sync_OnceCell<DependentInner>);

// impl crate::OnceCellCompatible<DependentInner> for SyncOnceCell {
//     fn new() -> Self {
//         SyncOnceCell(Sync_OnceCell::new())
//     }
//     fn get(&self) -> Option<&DependentInner> {
//         self.0.get()
//     }
//     fn get_or_init<F>(&self, f: F) -> &DependentInner
//     where
//         F: FnOnce() -> DependentInner,
//     {
//         self.0.get_or_init(f)
//     }
//     fn take(&mut self) -> Option<DependentInner> {
//         self.0.take()
//     }
// }

// // A mutable pointer that only gets changed in 2 ways:
// //
// // 1.
// // get_or_init, sync::OnceCell takes care of establishing a happens-before
// // relationship between a potential write and read of the lazy init.
// //
// // 2.
// // drop_dependent_unconditional, might overwrite the OnceCell with it's
// // default empty state. This hinges on OnceCell::take pulling out the
// // value only exactly once even if called concurrently. Which is given,
// // because the Rust type system ensures only exactly one &mut can exist
// // at any time. And a &mut is required for calling drop_dependent_unconditional.
// unsafe impl Send for SyncOnceCell {}
// unsafe impl Sync for SyncOnceCell {}

// #[macro_export]
// macro_rules! sync_once_self_cell {
//     ($StructName:ident, $Owner:ty, $Dependent:ty $(, $StructMeta:meta)* $(,)?) => {
//         $(#[$StructMeta])*
//         struct $StructName {
//             unsafe_self_cell: $crate::unsafe_once_self_cell::UnsafeOnceSelfCell<
//                 $Owner,
//                 $crate::SyncOnceCell,
//             >,
//         }

//         impl $StructName {
//             pub fn new(owner: $Owner) -> Self {
//                 Self {
//                     unsafe_self_cell: unsafe {
//                         $crate::unsafe_once_self_cell::UnsafeOnceSelfCell::new(owner)
//                     },
//                 }
//             }

//             pub fn get_owner<'a>(&'a self) -> &'a $Owner {
//                 unsafe { self.unsafe_self_cell.get_owner() }
//             }

//             pub fn get_or_init_dependent<'a>(&'a self) -> &'a $Dependent {
//                 unsafe {
//                     self.unsafe_self_cell
//                         .get_or_init_dependent(|owner_ref| owner_ref.into())
//                 }
//             }

//             pub fn dependent_is_none(&self) -> bool {
//                 self.unsafe_self_cell.dependent_is_none()
//             }
//         }

//         impl Drop for $StructName {
//             fn drop(&mut self) {
//                 unsafe {
//                     self.unsafe_self_cell.drop_dependent::<$Dependent>();
//                 }
//             }
//         }
//     };
// }

// // pub mod custom {
// //     // User provided OnceCell. Has to implement OnceCellCompatible.
// //     pub type OnceSelfCell<Owner, DependentStaticLifetime, DependentCell> =
// //         crate$crate::OnceSelfCell<Owner, DependentStaticLifetime, DependentCell>;
// // }

pub mod unsafe_self_cell;

#[doc(hidden)]
#[macro_export]
macro_rules! _cell_constructor {
    (from, $Owner:ty, $Dependent:ident) => {
        fn new(owner: $Owner) -> Self {
            unsafe {
                // All this has to happen here, because there is not good way
                // of passing the appropriate logic into UnsafeSelfCell::new
                // short of assuming Dependent<'static> is the same as
                // Dependent<'a>, which I'm not confident is safe.

                type JoinedCell<'a> = $crate::unsafe_self_cell::JoinedCell<$Owner, $Dependent<'a>>;

                let layout = $crate::alloc::alloc::Layout::new::<JoinedCell>();

                let joined_void_ptr = $crate::alloc::alloc::alloc(layout);

                let joined_ptr = core::mem::transmute::<*mut u8, *mut JoinedCell>(joined_void_ptr);

                // Move owner into newly allocated space.
                core::ptr::addr_of_mut!((*joined_ptr).owner).write(owner);

                // Initialize dependent with owner reference in final place.
                core::ptr::addr_of_mut!((*joined_ptr).dependent)
                    .write((&(*joined_ptr).owner).into());

                Self {
                    unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell::new(
                        joined_void_ptr,
                    ),
                }
            }
        }
    };
    (try_from, $Owner:ty, $Dependent:ident) => {
        fn try_from<'a>(
            owner: $Owner,
        ) -> Result<Self, <&'a $Owner as core::convert::TryInto<$Dependent<'a>>>::Error> {
            unsafe {
                // All this has to happen here, because there is not good way
                // of passing the appropriate logic into UnsafeSelfCell::new
                // short of assuming Dependent<'static> is the same as
                // Dependent<'a>, which I'm not confident is safe.

                type JoinedCell<'a> = $crate::unsafe_self_cell::JoinedCell<$Owner, $Dependent<'a>>;

                let layout = $crate::alloc::alloc::Layout::new::<JoinedCell>();

                let joined_void_ptr = $crate::alloc::alloc::alloc(layout);

                let joined_ptr = core::mem::transmute::<*mut u8, *mut JoinedCell>(joined_void_ptr);

                // Move owner into newly allocated space.
                core::ptr::addr_of_mut!((*joined_ptr).owner).write(owner);

                type Error<'a> = <&'a $Owner as core::convert::TryInto<$Dependent<'a>>>::Error;

                // Attempt to initialize dependent with owner reference in final place.
                let try_inplace_init = || -> Result<(), Error<'a>> {
                    core::ptr::addr_of_mut!((*joined_ptr).dependent)
                        .write((&(*joined_ptr).owner).try_into()?);

                    Ok(())
                };

                match try_inplace_init() {
                    Ok(()) => Ok(Self {
                        unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell::new(
                            joined_void_ptr,
                        ),
                    }),
                    Err(err) => {
                        // Clean up partially initialized joined_cell.
                        core::ptr::drop_in_place(core::ptr::addr_of_mut!((*joined_ptr).owner));

                        $crate::alloc::alloc::dealloc(joined_void_ptr, layout);

                        Err(err)
                    }
                }
            }
        }
    };
    ($x:ident, $Owner:ty, $Dependent:ident) => {
        compile_error!("This macro only accepts `from` or `try_from`");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _covariant_access {
    (covariant, $Dependent:ident) => {
        fn borrow_dependent<'a>(&'a self) -> &'a $Dependent {
            struct _Covariant<'b>($Dependent<'b>);

            fn _assert_covariance<'x: 'y, 'y>(x: _Covariant<'x>) -> _Covariant<'y> {
                //  This function only compiles for covariant types.
                x // Change the macro invocation to not_covariant.
            }

            unsafe { self.unsafe_self_cell.borrow_dependent() }
        }
    };
    (not_covariant, $Dependent:ident) => {
        // For types that are not covariant it's unsafe to allow
        // returning direct references.
        // For example a lifetime that is too short could be chosen:
        // See https://github.com/Voultapher/once_self_cell/issues/5
    };
    ($x:ident, $Dependent:ident) => {
        compile_error!("This macro only accepts `covariant` or `not_covariant`");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _impl_automatic_derive {
    (Clone, $StructName:ident) => {
        impl Clone for $StructName {
            // TODO doc deep copy behavior.
            fn clone(&self) -> Self {
                Self::new(self.borrow_owner().clone())
            }
        }
    };
    (Debug, $StructName:ident) => {
        impl core::fmt::Debug for $StructName {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
                self.with_dependent(|owner, dependent| {
                    write!(
                        fmt,
                        concat!(
                            stringify!($StructName),
                            " {{ owner: {:?}, dependent: {:?} }}"
                        ),
                        owner, dependent
                    )
                })
            }
        }
    };
    (PartialEq, $StructName:ident) => {
        impl PartialEq for $StructName {
            // TODO document assumes From<&Owner> is deterministic so it's
            // enough to compare owner.
            fn eq(&self, other: &Self) -> bool {
                unsafe { *self.borrow_owner() == *other.borrow_owner() }
            }
        }
    };
    (Eq, $StructName:ident) => {
        // TODO this should only be allowed if owner is Eq.
        impl Eq for $StructName {}
    };
    (Hash, $StructName:ident) => {
        impl core::hash::Hash for $StructName {
            // TODO document assumes From<&Owner> is deterministic so it's
            // enough to hash owner.
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                unsafe { self.borrow_owner() }.hash(state);
            }
        }
    };
    ($x:ident, $StructName:ident) => {
        compile_error!(concat!(
            "No automatic trait impl for trait: ",
            stringify!($x)
        ));
    };
}

#[macro_export]
macro_rules! self_cell {
    (
        $StructName:ident,
        {$($Automatic_derive:ident),*},
        $ConstructorType:ident,
        $Owner:ty,
        $Dependent:ident,
        $Covariance:ident
        $(, $StructMeta:meta)* $(,)?
    ) => {
        $(#[$StructMeta])*
        struct $StructName {
            unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell<
                $Owner,
                $Dependent<'static>
            >
        }

        impl $StructName {
            $crate::_cell_constructor!($ConstructorType, $Owner, $Dependent);

            fn borrow_owner<'a>(&'a self) -> &'a $Owner {
                unsafe { self.unsafe_self_cell.borrow_owner::<$Dependent<'a>>() }
            }

            fn with_dependent<Ret>(&self, func: impl for<'a> FnOnce(&'a $Owner, &'a $Dependent<'a>) -> Ret) -> Ret {
                unsafe {
                    func(
                        self.unsafe_self_cell.borrow_owner::<$Dependent>(),
                        self.unsafe_self_cell.borrow_dependent()
                    )
                }
            }

            $crate::_covariant_access!($Covariance, $Dependent);
        }

        impl Drop for $StructName {
            fn drop<'a>(&mut self) {
                unsafe {
                    self.unsafe_self_cell.drop_joined::<$Dependent>();
                }
            }
        }

        // The user has to choose which traits can and should be automatically
        // implemented for the cell.
        $(
            $crate::_impl_automatic_derive!($Automatic_derive, $StructName);
        )*
    };
}
