//! Various verbose shorthand types related to `PhantomData`.
//!
//! # Variance
//! 
//! Variance can be very confusing to beginners. Generally,
//! when one talks about "subtype" and "supertype" in Rust,
//! it is specifically about lifetimes in the current version.
//!
//! Specifically, any reference type `&'any T` is a subtype of
//! the static reference type `&'static T`. A reference with a
//! shorter lifetime is a subtype of another reference with a
//! longer lifetime.
//! 
//! ### Covariant
//! 
//! When a `Foo<T>` is "covariant" over `T`, it *shares* the
//! same subtyping rules with `T`, i.e. `Foo<&'a T>` is a subtype
//! of `Foo<&'static T>`. 
//! When a lifetime parameter is covariant, it suggests that a
//! shorter lifetime parameter is a subtype of a longer lifetime
//! parameter, i.e. `Foo<'a>` is a subtype of `Foo<'static>`.
//! 
//! ### Contravariant
//! 
//! Contravariant is rare, it *reverses* the normal subtyping
//! rules. `Foo<'static>` is a subtype of `Foo<'a>` if the lifetimes
//! are contravariant. It is the property of argument types of
//! functions. `fn(&'static ())` is the most strict and would be
//! the super type for all `fn(&'a ())`.
//! 
//! ### Invariant
//! 
//! If a parameter is invariant, it cannot be changed. Only equal
//! arguments are sub/super types of themselves. `T` is invariant
//! in `&mut T` because you can perform both read and write
//! operations on it, and you cannot write a `T2` with a shorter
//! lifetime than `T`, nor read a `T3` with a longer lifetime.
//! `T` is invariant in `fn(T) -> T` for similar reasons.
//! 
//! ### Drop Check
//! 
//! A difference between the [`CovariantOver`] type offered in this
//! crate and the [`Owns`] type is that `Owns` will cause the compiler
//! to use "drop check". Drop checking prevents use of dropped values
//! in another type's destructor. The examples below are taken and
//! adapted from the [respective Rustonomicon chapter]:
//! 
//! Imagine a custom [`Box`](std::boxed::Box) type defined like this:
//! 
//! ```
//! use std::ptr::NonNull;
//! struct MyBox<T> {
//!     inner: NonNull<T>,
//! }
//! 
//! impl<T> Drop for MyBox<T> {
//!     fn drop(&mut self) { /* free memory.. */ }
//! }
//! ```
//! 
//! Although [`NonNull`](std::ptr::NonNull) is covariant over `T`,
//! `MyBox` must use [`PhantomData`] to signify that it owns `T`,
//! otherwise it could allow access to dropped data in a destructor:
//! 
//! ```
//! # use std::ptr::NonNull;
//! # struct MyBox<T> {
//! #     inner: NonNull<T>,
//! # }
//! # impl<T> MyBox<T> {
//! #     fn new(inner: T) -> Self { Self { inner: NonNull::dangling() } }
//! # }
//! struct Inspector<'a>(&'a u8);
//! impl<'a> Drop for Inspector<'a> {
//!    fn drop(&mut self) {
//!        println!("I was only {} days from retirement!", self.0);
//!    }
//! }
//! struct World<'a> {
//!     inspector: Option<MyBox<Inspector<'a>>>,
//!     days: Box<u8>,
//! }
//! 
//! fn main() {
//!     let mut world = World {
//!         inspector: None,
//!         days: Box::new(1),
//!     };
//!     world.inspector = Some(MyBox::new(Inspector(&world.days)));
//!     // Let's say `days` happens to get dropped first.
//!     // Then when Inspector is dropped, it will try to read free'd memory!
//! }
//! ```
//! 
//! In this example, marking `MyBox` with a [`PhantomData<T>`] or [`Owns<T>`]
//! resolves the unsoundness issue:
//! 
//! ```compile_fail
//! # use std::ptr::NonNull;
//! # use phtm::Owns<T>
//! struct MyBox<T> {
//!     inner: NonNull<T>,
//!     _owns_t: Owns<T>,
//! }
//! # impl<T> MyBox<T> {
//! #     fn new(inner: T) -> Self { Self { inner: NonNull::dangling() } }
//! # }
//! # struct Inspector<'a>(&'a u8);
//! # impl<'a> Drop for Inspector<'a> {
//! #   fn drop(&mut self) {
//! #        println!("I was only {} days from retirement!", self.0);
//! #   }
//! # }
//! # struct World<'a> {
//! #     inspector: Option<MyBox<Inspector<'a>>>,
//! #     days: Box<u8>,
//! # }
//! 
//! fn main() {
//!     let mut world = World {
//!         inspector: None,
//!         days: Box::new(1),
//!     };
//!
//!     world.inspector = Some(MyBox::new(Inspector(&world.days)));
//!     // ^ now fails to compile!
//! }
//! ```
//! 
//! [respective Rustonomicon chapter]: https://doc.rust-lang.org/nomicon/dropck.html
//! 
//! # Markers
//! 
//! The [`Send`] and [`Sync`] markers can be controlled by adding
//! [`NotSendOrSync`] or [`NotSync`] to your type and/or writing
//! `unsafe impl`s of the markers to your type. The markers control
//! what types can be sent between threads safely. The non-atomic
//! reference counted type [`Rc`](std::rc::Rc) cannot be sent
//! between threads because there might be two threads holding the
//! same object and could increase the reference counter
//! non-atomically, causing a data race.
//! 
//! Usually your type is automatically not [`Send`] when there are
//! fields of pointers or single-threaded containers such as `Rc`,
//! but there could be benefits of explicitly defining a type as not
//! [`Send`] or not [`Sync`].
//! 
//! Doing so can prevent semver hazards when a public type suddenly
//! stops implementing any of these marker types, causing a breaking
//! change, while explicitly adding the marker types allows future
//! possibilities of having single-threaded containers without
//! bumping the major version.

#![cfg_attr(not(doc), no_std)] // intra doc links need std
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, rust_2018_idioms, future_incompatible)]
#![deny(rustdoc::broken_intra_doc_links, missing_docs)]

use core::cell::Cell;

#[doc(no_inline)]
pub use core::marker::PhantomData;

#[doc(no_inline)]
pub use core::marker::PhantomPinned;

/// Verbose version of `PhantomData`.
/// 
/// It is covariant over `T` with drop checking.
///
/// See the [crate root documentation] for details on variance
/// and drop checking.
///
/// [crate root documentation]: index.html
pub type Owns<T> = PhantomData<T>;

/// Alias for phantom `&'a T`. Both parameters are covariant.
///
/// If you do not have a lifetime handy, you could use
/// [`HasImmPtrTo`].
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type ImmutablyReferences<'a, T> = PhantomData<&'a T>;

/// Alias for `PhantomData<&'a mut T>`. `'a` is covariant
/// while `T` is invariant.
///
/// If you do not have a lifetime handy, you could use
/// [`HasMutPtrTo`].
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type MutablyReferences<'a, T> = PhantomData<&'a mut T>;

/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type HasImmPtrTo<T> = PhantomData<*const T>;

/// Marks the containing type as having mutable pointers to `T`.
///
/// `T` is invariant, it also marks the containing type as not
/// [`Send`] and not [`Sync`].
///
/// See the [crate root documentation] for details on variance
/// and marker traits.
///
/// [crate root documentation]: index.html
pub type HasMutPtrTo<T> = PhantomData<*mut T>;

/// Mark a type as covariant.
/// 
/// This marker does not "own" the type, i.e. `T` does not get
/// dropped when the containing type is dropped. If `T` could
/// be dropped, use [`Owns`] instead.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type CovariantOver<T> = PhantomData<fn() -> T>;

/// Marks a lifetime as covariant.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type CovariantOverLt<'co> = PhantomData<ContravariantOver<&'co ()>>;

/// Marks a type as contravariant.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type ContravariantOver<T> = PhantomData<fn(T)>;

/// Marks a lifetime as contravariant.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type ContraVariantOverLt<'contra> = ContravariantOver<&'contra ()>;

/// Marks a type as invariant.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type InvariantOver<T> = PhantomData<fn(T) -> T>;

/// Marks a lifetime as invariant.
///
/// See the [crate root documentation] for details on variance.
///
/// [crate root documentation]: index.html
pub type InvariantOverLt<'inv> = InvariantOver<&'inv ()>;

/// Marks the containing type as not [`Sync`].
///
/// See the [crate root documentation] for details on marker
/// traits.
///
/// [crate root documentation]: index.html
pub type NotSync = PhantomData<Cell<()>>;

/// Marks the containing type as not [`Send`] and not [`Sync`].
///
/// See the [crate root documentation] for details on marker
/// traits.
///
/// [crate root documentation]: index.html
pub type NotSendOrSync = PhantomData<*const ()>;