//! # Overview
//!
//! `self_cell` provides one macro-rules macro: [`self_cell`]. With this macro
//! you can create self-referential structs that are safe-to-use in stable Rust,
//! without leaking the struct internal lifetime.
//!
//! In a nutshell, the API looks *roughly* like this:
//!
//! ```ignore
//! // User code:
//!
//! self_cell!(
//!     struct NewStructName {
//!         owner: Owner,
//!
//!         #[covariant]
//!         dependent: Dependent,
//!     }
//!
//!     impl {Debug}
//! );
//!
//! // Generated by macro:
//!
//! struct NewStructName(...);
//!
//! impl NewStructName {
//!     fn new(
//!         owner: Owner,
//!         dependent_builder: impl for<'a> ::core::ops::FnOnce(&'a Owner) -> Dependent<'a>
//!     ) -> NewStructName { ... }
//!     fn borrow_owner<'a>(&'a self) -> &'a Owner { ... }
//!     fn borrow_dependent<'a>(&'a self) -> &'a Dependent<'a> { ... }
//!     [...]
//!     // See the macro level documentation for a list of all generated functions,
//!     // section "Generated API".
//! }
//!
//! impl Debug for NewStructName { ... }
//! ```
//!
//! Self-referential structs are currently not supported with safe vanilla Rust.
//! The only reasonable safe alternative is to have the user juggle 2 separate
//! data structures which is a mess. The library solution ouroboros is expensive
//! to compile due to its use of procedural macros.
//!
//! This alternative is `no_std`, uses no proc-macros, some self contained
//! unsafe and works on stable Rust, and is miri tested. With a total of less
//! than 300 lines of implementation code, which consists mostly of type and
//! trait implementations, this crate aims to be a good minimal solution to the
//! problem of self-referential structs.
//!
//! It has undergone [community code
//! review](https://users.rust-lang.org/t/experimental-safe-to-use-proc-macro-free-self-referential-structs-in-stable-rust/52775)
//! from experienced Rust users.
//!
//! ### Fast compile times
//!
//! ```txt
//! $ rm -rf target && cargo +nightly build -Z timings
//!
//! Compiling self_cell v0.7.0
//! Completed self_cell v0.7.0 in 0.2s
//! ```
//!
//! Because it does **not** use proc-macros, and has 0 dependencies
//! compile-times are fast.
//!
//! Measurements done on a slow laptop.
//!
//! ### A motivating use case
//!
//! ```rust
//! use self_cell::self_cell;
//!
//! #[derive(Debug, Eq, PartialEq)]
//! struct Ast<'a>(pub Vec<&'a str>);
//!
//! self_cell!(
//!     struct AstCell {
//!         owner: String,
//!
//!         #[covariant]
//!         dependent: Ast,
//!     }
//!
//!     impl {Debug, Eq, PartialEq}
//! );
//!
//! fn build_ast_cell(code: &str) -> AstCell {
//!     // Create owning String on stack.
//!     let pre_processed_code = code.trim().to_string();
//!
//!     // Move String into AstCell, then build Ast inplace.
//!     AstCell::new(
//!        pre_processed_code,
//!        |code| Ast(code.split(' ').filter(|word| word.len() > 1).collect())
//!     )
//! }
//!
//! fn main() {
//!     let ast_cell = build_ast_cell("fox = cat + dog");
//!
//!     println!("ast_cell -> {:?}", &ast_cell);
//!     println!("ast_cell.borrow_owner() -> {:?}", ast_cell.borrow_owner());
//!     println!("ast_cell.borrow_dependent().0[1] -> {:?}", ast_cell.borrow_dependent().0[1]);
//! }
//! ```
//!
//! ```txt
//! $ cargo run
//!
//! ast_cell -> AstCell { owner: "fox = cat + dog", dependent: Ast(["fox", "cat", "dog"]) }
//! ast_cell.borrow_owner() -> "fox = cat + dog"
//! ast_cell.borrow_dependent().0[1] -> "cat"
//! ```
//!
//! There is no way in safe Rust to have an API like `build_ast_cell`, as soon
//! as `Ast` depends on stack variables like `pre_processed_code` you can't
//! return the value out of the function anymore. You could move the
//! pre-processing into the caller but that gets ugly quickly because you can't
//! encapsulate things anymore. Note this is a somewhat niche use case,
//! self-referential structs should only be used when there is no good
//! alternative.
//!
//! Under the hood, it heap allocates a struct which it initializes first by
//! moving the owner value to it and then using the reference to this now
//! Pin/Immovable owner to construct the dependent inplace next to it. This
//! makes it safe to move the generated SelfCell but you have to pay for the
//! heap allocation.
//!
//! See the documentation for [`self_cell`] to dive further into the details.
//!
//! Or take a look at the advanced examples:
//! - [Example how to handle dependent construction that can
//!   fail](https://github.com/Voultapher/self_cell/tree/main/examples/fallible_dependent_construction)
//!
//! - [How to build a lazy AST with
//!   self_cell](https://github.com/Voultapher/self_cell/tree/main/examples/lazy_ast)
//!
//! - [How to handle dependents that take a mutable reference](https://github.com/Voultapher/self_cell/tree/main/examples/mut_ref_to_owner_in_builder) see also [`MutBorrow`]
//!
//! - [How to use an owner type with
//!     lifetime](https://github.com/Voultapher/self_cell/tree/main/examples/owner_with_lifetime)
//!
//! ### Min required rustc version
//!
//! By default the minimum required rustc version is 1.51.
//!
//! There is an optional feature you can enable called "old_rust" that enables
//! support down to rustc version 1.36. However this requires polyfilling std
//! library functionality for older rustc with technically UB versions. Testing
//! does not show older rustc versions (ab)using this. Use at your own risk.
//!
//! The minimum versions are a best effor and may change with any new major
//! release.

#![no_std]

#[doc(hidden)]
pub extern crate alloc;

#[doc(hidden)]
pub mod unsafe_self_cell;

/// This macro declares a new struct of `$StructName` and implements traits
/// based on `$AutomaticDerive`.
///
/// ### Example:
///
/// ```rust
/// use self_cell::self_cell;
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct Ast<'a>(Vec<&'a str>);
///
/// self_cell!(
///     #[doc(hidden)]
///     struct PackedAstCell {
///         owner: String,
///
///         #[covariant]
///         dependent: Ast,
///     }
///
///     impl {Debug, PartialEq, Eq, Hash}
/// );
/// ```
///
/// See the crate overview to get a get an overview and a motivating example.
///
/// ### Generated API:
///
/// The macro implements these constructors:
///
/// ```ignore
/// fn new(
///     owner: $Owner,
///     dependent_builder: impl for<'a> ::core::ops::FnOnce(&'a $Owner) -> $Dependent<'a>
/// ) -> Self
/// ```
///
/// ```ignore
/// fn try_new<Err>(
///     owner: $Owner,
///     dependent_builder: impl for<'a> ::core::ops::FnOnce(&'a $Owner) -> Result<$Dependent<'a>, Err>
/// ) -> Result<Self, Err>
/// ```
///
/// ```ignore
/// fn try_new_or_recover<Err>(
///     owner: $Owner,
///     dependent_builder: impl for<'a> ::core::ops::FnOnce(&'a $Owner) -> Result<$Dependent<'a>, Err>
/// ) -> Result<Self, ($Owner, Err)>
/// ```
///
/// The macro implements these methods:
///
/// ```ignore
/// fn borrow_owner<'a>(&'a self) -> &'a $Owner
/// ```
///
/// ```ignore
/// // Only available if dependent is covariant.
/// fn borrow_dependent<'a>(&'a self) -> &'a $Dependent<'a>
/// ```
///
/// ```ignore
/// fn with_dependent<'outer_fn, Ret>(
///     &'outer_fn self,
///     func: impl for<'a> ::core::ops::FnOnce(&'a $Owner, &'outer_fn $Dependent<'a>
/// ) -> Ret) -> Ret
/// ```
///
/// ```ignore
/// fn with_dependent_mut<'outer_fn, Ret>(
///     &'outer_fn mut self,
///     func: impl for<'a> ::core::ops::FnOnce(&'a $Owner, &'outer_fn mut $Dependent<'a>) -> Ret
/// ) -> Ret
/// ```
///
/// ```ignore
/// fn into_owner(self) -> $Owner
/// ```
///
///
/// ### Parameters:
///
/// - `$Vis:vis struct $StructName:ident` Name of the struct that will be
///   declared, this needs to be unique for the relevant scope. Example: `struct
///   AstCell` or `pub struct AstCell`. `$Vis` can be used to mark the struct
///   and all functions implemented by the macro as public.
///
///   `$(#[$StructMeta:meta])*` allows you specify further meta items for this
///   struct, eg. `#[doc(hidden)] struct AstCell`.
///
/// - `$Owner:ty` Type of owner. This has to have a `'static` lifetime. Example:
///   `String`.
///
/// - `$Dependent:ident` Name of the dependent type without specified lifetime.
///   This can't be a nested type name. As workaround either create a type alias
///   `type Dep<'a> = Option<Vec<&'a str>>;` or create a new-type `struct
///   Dep<'a>(Option<Vec<&'a str>>);`. Example: `Ast`.
///
///   `$Covariance:ident` Marker declaring if `$Dependent` is
///   [covariant](https://doc.rust-lang.org/nightly/nomicon/subtyping.html).
///   Possible Values:
///
///   * **covariant**: This generates the direct reference accessor function
///     `borrow_dependent`. This is only safe to do if this compiles `fn
///     _assert_covariance<'x: 'y, 'y>(x: &'y $Dependent<'x>) -> &'y $Dependent<'y>
///     {x}`. Otherwise you could choose a lifetime that is too short for types
///     with interior mutability like `Cell`, which can lead to UB in safe code.
///     Which would violate the promise of this library that it is safe-to-use.
///     If you accidentally mark a type that is not covariant as covariant, you
///     will get a compile time error.
///
///   * **not_covariant**: This generates no additional code but you can use the
///     `with_dependent` function. See [How to build a lazy AST with
///     self_cell](https://github.com/Voultapher/self_cell/tree/main/examples/lazy_ast)
///     for a usage example.
///
///   In both cases you can use the `with_dependent_mut` function to mutate the
///   dependent value. This is safe to do because notionally you are replacing
///   pointers to a value not the other way around.
///
/// - `impl {$($AutomaticDerive:ident),*},` Optional comma separated list of
///   optional automatic trait implementations. Possible Values:
///
///   * **Debug**: Prints the debug representation of owner and dependent.
///     Example: `AstCell { owner: "fox = cat + dog", dependent: Ast(["fox",
///     "cat", "dog"]) }`
///
///   * **PartialEq**: Logic `*self.borrow_owner() == *other.borrow_owner()`,
///     this assumes that `Dependent<'a>::From<&'a Owner>` is deterministic, so
///     that only comparing owner is enough.
///
///   * **Eq**: Will implement the trait marker `Eq` for `$StructName`. Beware
///     if you select this `Eq` will be implemented regardless if `$Owner`
///     implements `Eq`, that's an unfortunate technical limitation.
///
///   * **Hash**: Logic `self.borrow_owner().hash(state);`, this assumes that
///     `Dependent<'a>::From<&'a Owner>` is deterministic, so that only hashing
///     owner is enough.
///
///   All `AutomaticDerive` are optional and you can implement you own version
///   of these traits. The declared struct is part of your module and you are
///   free to implement any trait in any way you want. Access to the unsafe
///   internals is only possible via unsafe functions, so you can't accidentally
///   use them in safe code.
///
///   There is limited nested cell support. Eg, having an owner with non static
///   references. Eg `struct ChildCell<'a> { owner: &'a String, ...`. You can
///   use any lifetime name you want, except `_q` and only a single lifetime is
///   supported, and can only be used in the owner. Due to macro_rules
///   limitations, no `AutomaticDerive` are supported if an owner lifetime is
///   provided.
///
#[macro_export]
macro_rules! self_cell {
(
    $(#[$StructMeta:meta])*
    $Vis:vis struct $StructName:ident $(<$OwnerLifetime:lifetime>)? {
        owner: $Owner:ty,

        #[$Covariance:ident]
        dependent: $Dependent:ident,
    }

    $(impl {$($AutomaticDerive:ident),*})?
) => {
    #[repr(transparent)]
    $(#[$StructMeta])*
    $Vis struct $StructName $(<$OwnerLifetime>)? {
        unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell<
            $StructName$(<$OwnerLifetime>)?,
            $Owner,
            $Dependent<'static>
        >,

        $(owner_marker: $crate::_covariant_owner_marker!($Covariance, $OwnerLifetime) ,)?
    }

    impl $(<$OwnerLifetime>)? $StructName $(<$OwnerLifetime>)? {
        /// Constructs a new self-referential struct.
        ///
        /// The provided `owner` will be moved into a heap allocated box.
        /// Followed by construction of the dependent value, by calling
        /// `dependent_builder` with a shared reference to the owner that
        /// remains valid for the lifetime of the constructed struct.
        $Vis fn new(
            owner: $Owner,
            dependent_builder: impl for<'_q> ::core::ops::FnOnce(&'_q $Owner) -> $Dependent<'_q>
        ) -> Self {
            use ::core::ptr::NonNull;

            #[allow(unused)]
            use $crate::unsafe_self_cell::MutBorrowDefaultTrait;

            unsafe {
                // All this has to happen here, because there is not good way
                // of passing the appropriate logic into UnsafeSelfCell::new
                // short of assuming Dependent<'static> is the same as
                // Dependent<'_q>, which I'm not confident is safe.

                // For this API to be safe there has to be no safe way to
                // capture additional references in `dependent_builder` and then
                // return them as part of Dependent. Eg. it should be impossible
                // to express: '_q should outlive 'x here `fn
                // bad<'_q>(outside_ref: &'_q String) -> impl for<'x> ::core::ops::FnOnce(&'x
                // Owner) -> Dependent<'x>`.

                type JoinedCell<'_q $(, $OwnerLifetime)?> =
                    $crate::unsafe_self_cell::JoinedCell<$Owner, $Dependent<'_q>>;

                let layout = $crate::alloc::alloc::Layout::new::<JoinedCell>();
                assert!(layout.size() != 0);

                let joined_void_ptr = NonNull::new($crate::alloc::alloc::alloc(layout)).unwrap();

                let mut joined_ptr = joined_void_ptr.cast::<JoinedCell>();

                let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());

                // Move owner into newly allocated space.
                owner_ptr.write(owner);

                $crate::_mut_borrow_unlock!(owner_ptr, $Owner);

                // Drop guard that cleans up should building the dependent panic.
                let drop_guard =
                    $crate::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);

                // Initialize dependent with owner reference in final place.
                dependent_ptr.write(dependent_builder(&*owner_ptr));
                ::core::mem::forget(drop_guard);

                $crate::_mut_borrow_lock!(owner_ptr, $Owner);

                Self {
                    unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell::new(
                        joined_void_ptr,
                    ),
                    $(owner_marker: $crate::_covariant_owner_marker_ctor!($OwnerLifetime) ,)?
                }
            }
        }

        /// Tries to create a new structure with a given dependent builder.
        ///
        /// Consumes owner on error.
        $Vis fn try_new<Err>(
            owner: $Owner,
            dependent_builder:
                impl for<'_q> ::core::ops::FnOnce(&'_q $Owner) -> ::core::result::Result<$Dependent<'_q>, Err>
        ) -> ::core::result::Result<Self, Err> {
            use ::core::ptr::NonNull;

            #[allow(unused)]
            use $crate::unsafe_self_cell::MutBorrowDefaultTrait;

            unsafe {
                // See fn new for more explanation.

                type JoinedCell<'_q $(, $OwnerLifetime)?> =
                    $crate::unsafe_self_cell::JoinedCell<$Owner, $Dependent<'_q>>;

                let layout = $crate::alloc::alloc::Layout::new::<JoinedCell>();
                assert!(layout.size() != 0);

                let joined_void_ptr = NonNull::new($crate::alloc::alloc::alloc(layout)).unwrap();

                let mut joined_ptr = joined_void_ptr.cast::<JoinedCell>();

                let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());

                // Move owner into newly allocated space.
                owner_ptr.write(owner);

                $crate::_mut_borrow_unlock!(owner_ptr, $Owner);

                // Drop guard that cleans up should building the dependent panic.
                let mut drop_guard =
                    $crate::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);

                match dependent_builder(&*owner_ptr) {
                    ::core::result::Result::Ok(dependent) => {
                        dependent_ptr.write(dependent);
                        ::core::mem::forget(drop_guard);

                        $crate::_mut_borrow_lock!(owner_ptr, $Owner);

                        ::core::result::Result::Ok(Self {
                            unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell::new(
                                joined_void_ptr,
                            ),
                            $(owner_marker: $crate::_covariant_owner_marker_ctor!($OwnerLifetime) ,)?
                        })
                    }
                    ::core::result::Result::Err(err) => ::core::result::Result::Err(err)
                }
            }
        }

        /// Tries to create a new structure with a given dependent builder.
        ///
        /// Returns owner on error.
        $Vis fn try_new_or_recover<Err>(
            owner: $Owner,
            dependent_builder:
                impl for<'_q> ::core::ops::FnOnce(&'_q $Owner) -> ::core::result::Result<$Dependent<'_q>, Err>
        ) -> ::core::result::Result<Self, ($Owner, Err)> {
            use ::core::ptr::NonNull;

            #[allow(unused)]
            use $crate::unsafe_self_cell::MutBorrowDefaultTrait;

            unsafe {
                // See fn new for more explanation.

                type JoinedCell<'_q $(, $OwnerLifetime)?> =
                    $crate::unsafe_self_cell::JoinedCell<$Owner, $Dependent<'_q>>;

                let layout = $crate::alloc::alloc::Layout::new::<JoinedCell>();
                assert!(layout.size() != 0);

                let joined_void_ptr = NonNull::new($crate::alloc::alloc::alloc(layout)).unwrap();

                let mut joined_ptr = joined_void_ptr.cast::<JoinedCell>();

                let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());

                // Move owner into newly allocated space.
                owner_ptr.write(owner);

                $crate::_mut_borrow_unlock!(owner_ptr, $Owner);

                // Drop guard that cleans up should building the dependent panic.
                let mut drop_guard =
                    $crate::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);

                match dependent_builder(&*owner_ptr) {
                    ::core::result::Result::Ok(dependent) => {
                        dependent_ptr.write(dependent);
                        ::core::mem::forget(drop_guard);

                        $crate::_mut_borrow_lock!(owner_ptr, $Owner);

                        ::core::result::Result::Ok(Self {
                            unsafe_self_cell: $crate::unsafe_self_cell::UnsafeSelfCell::new(
                                joined_void_ptr,
                            ),
                            $(owner_marker: $crate::_covariant_owner_marker_ctor!($OwnerLifetime) ,)?
                        })
                    }
                    ::core::result::Result::Err(err) => {
                        // In contrast to into_owner ptr::read, here no dependent
                        // ever existed in this function and so we are sure its
                        // drop impl can't access owner after the read.
                        // And err can't return a reference to owner.
                        let owner_on_err = ::core::ptr::read(owner_ptr);

                        // Allowing drop_guard to finish would let it double free owner.
                        // So we dealloc the JoinedCell here manually.
                        ::core::mem::forget(drop_guard);
                        $crate::alloc::alloc::dealloc(joined_void_ptr.as_ptr(), layout);

                        ::core::result::Result::Err((owner_on_err, err))
                    }
                }
            }
        }

        /// Borrows owner.
        $Vis fn borrow_owner<'_q>(&'_q self) -> &'_q $Owner {
            unsafe { self.unsafe_self_cell.borrow_owner::<$Dependent<'_q>>() }
        }

        /// Calls given closure `func` with a shared reference to dependent.
        $Vis fn with_dependent<'outer_fn, Ret>(
            &'outer_fn self,
            func: impl for<'_q> ::core::ops::FnOnce(&'_q $Owner, &'outer_fn $Dependent<'_q>
        ) -> Ret) -> Ret {
            unsafe {
                func(
                    self.unsafe_self_cell.borrow_owner::<$Dependent>(),
                    self.unsafe_self_cell.borrow_dependent()
                )
            }
        }

        /// Calls given closure `func` with an unique reference to dependent.
        $Vis fn with_dependent_mut<'outer_fn, Ret>(
            &'outer_fn mut self,
            func: impl for<'_q> ::core::ops::FnOnce(&'_q $Owner, &'outer_fn mut $Dependent<'_q>) -> Ret
        ) -> Ret {
            let (owner, dependent) = unsafe {
                    self.unsafe_self_cell.borrow_mut()
            };

            func(owner, dependent)
        }

        $crate::_covariant_access!($Covariance, $Vis, $Dependent);

        /// Consumes `self` and returns the the owner.
        $Vis fn into_owner(self) -> $Owner {
            // This is only safe to do with repr(transparent).
            let unsafe_self_cell = unsafe { ::core::mem::transmute::<
                Self,
                $crate::unsafe_self_cell::UnsafeSelfCell<
                    $StructName$(<$OwnerLifetime>)?,
                    $Owner,
                    $Dependent<'static>
                >
            >(self) };

            let owner = unsafe { unsafe_self_cell.into_owner::<$Dependent>() };

            owner
        }
    }

    impl $(<$OwnerLifetime>)? Drop for $StructName $(<$OwnerLifetime>)? {
        fn drop(&mut self) {
            unsafe {
                self.unsafe_self_cell.drop_joined::<$Dependent>();
            }
        }
    }

    // The user has to choose which traits can and should be automatically
    // implemented for the cell.
    $($(
        $crate::_impl_automatic_derive!($AutomaticDerive, $StructName);
    )*)*
};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _covariant_access {
    (covariant, $Vis:vis, $Dependent:ident) => {
        /// Borrows dependent.
        $Vis fn borrow_dependent<'_q>(&'_q self) -> &'_q $Dependent<'_q> {
            fn _assert_covariance<'x: 'y, 'y>(x: &'y $Dependent<'x>) -> &'y $Dependent<'y> {
                //  This function only compiles for covariant types.
                x // Change the macro invocation to not_covariant.
            }

            unsafe { self.unsafe_self_cell.borrow_dependent() }
        }
    };
    (not_covariant, $Vis:vis, $Dependent:ident) => {
        // For types that are not covariant it's unsafe to allow
        // returning direct references.
        // For example a lifetime that is too short could be chosen:
        // See https://github.com/Voultapher/self_cell/issues/5
    };
    ($x:ident, $Vis:vis, $Dependent:ident) => {
        compile_error!("This macro only accepts `covariant` or `not_covariant`");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _covariant_owner_marker {
    (covariant, $OwnerLifetime:lifetime) => {
        // Ensure that contravariant owners don't imply covariance
        // over the dependent. See issue https://github.com/Voultapher/self_cell/issues/18
        ::core::marker::PhantomData<&$OwnerLifetime ()>
    };
    (not_covariant, $OwnerLifetime:lifetime) => {
        // See the discussion in https://github.com/Voultapher/self_cell/pull/29
        //
        // If the dependent is non_covariant, mark the owner as invariant over its
        // lifetime. Otherwise unsound use is possible.
        ::core::marker::PhantomData<fn(&$OwnerLifetime ()) -> &$OwnerLifetime ()>
    };
    ($x:ident, $OwnerLifetime:lifetime) => {
        compile_error!("This macro only accepts `covariant` or `not_covariant`");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _covariant_owner_marker_ctor {
    ($OwnerLifetime:lifetime) => {
        // Helper to optionally expand into PhantomData for construction.
        ::core::marker::PhantomData
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _impl_automatic_derive {
    (Debug, $StructName:ident) => {
        impl ::core::fmt::Debug for $StructName {
            fn fmt(
                &self,
                fmt: &mut ::core::fmt::Formatter,
            ) -> ::core::result::Result<(), ::core::fmt::Error> {
                self.with_dependent(|owner, dependent| {
                    fmt.debug_struct(stringify!($StructName))
                        .field("owner", owner)
                        .field("dependent", dependent)
                        .finish()
                })
            }
        }
    };
    (PartialEq, $StructName:ident) => {
        impl ::core::cmp::PartialEq for $StructName {
            fn eq(&self, other: &Self) -> bool {
                *self.borrow_owner() == *other.borrow_owner()
            }
        }
    };
    (Eq, $StructName:ident) => {
        // TODO this should only be allowed if owner is Eq.
        impl ::core::cmp::Eq for $StructName {}
    };
    (Hash, $StructName:ident) => {
        impl ::core::hash::Hash for $StructName {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.borrow_owner().hash(state);
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

#[doc(hidden)]
#[macro_export]
macro_rules! _mut_borrow_unlock {
    ($owner_ptr:expr, $Owner:ty) => {{
        let wrapper = std::mem::transmute::<
            &mut $Owner,
            &mut $crate::unsafe_self_cell::MutBorrowSpecWrapper<$Owner>,
        >(&mut *$owner_ptr);

        // If `T` is `MutBorrow` will call `unlock`, otherwise a no-op.
        wrapper.unlock();
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _mut_borrow_lock {
    ($owner_ptr:expr, $Owner:ty) => {{
        let wrapper = std::mem::transmute::<
            &$Owner,
            &$crate::unsafe_self_cell::MutBorrowSpecWrapper<$Owner>,
        >(&*$owner_ptr);

        // If `T` is `MutBorrow` will call `lock`, otherwise a no-op.
        wrapper.lock();
    }};
}

pub use unsafe_self_cell::MutBorrow;
