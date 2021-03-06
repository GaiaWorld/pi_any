#![deny(unsafe_code)]
//! https://github.com/fkoep/downcast-rs
//! 该库参考了[downcast-rs](https://github.com/fkoep/downcast-rs), 为Box<dyn Trait>、Rc<dyn Trait>、Arc<dyn Trait>实现了downcast接口（向下造型）
//!
//! 为什么不直接使用[downcast-rs](https://github.com/fkoep/downcast-rs)?
//! downcast-rs仅为Box<dyn Trait>、Rc<dyn Trait>提供了`downcast`,未提供Arc<dyn Trait>的`downcast`方法
//! 
//! 补充：你不应该再使用本库，后续可能删除本库，因为最新的[downcast-rs](https://github.com/fkoep/downcast-rs)已经提供了Arc<dyn Trait>的downcast接口（2021.5.21）
//!
//! 下面的注释，拷贝了[downcast-rs](https://github.com/fkoep/downcast-rs)的README.md, 并不说明本库的用法。
//! Rust enums are great for types where all variations are known beforehand. But in
//! the case where you want to implement a container of user-defined types, an
//! open-ended type like a **trait object** is needed. In some cases, it is useful to
//! cast the trait object back into its original concrete type to access additional
//! functionality and performant inlined implementations.
//!
//! `downcast-rs` adds downcasting support to trait objects using only safe Rust. It
//! supports **type parameters**, **associated types**, and **constraints**.
//!
//! To make a trait downcastable, make it extend the `any::BoxAny` trait and
//! invoke `impl_downcast!` on it as follows:
//!
//! ```
//! # #[macro_use]
//! # extern crate any;
//! # use any::BoxAny;
//! trait Trait: BoxAny {}
//! impl_downcast!(Trait);
//!
//! // With type parameters.
//! trait TraitGeneric1<T>: BoxAny {}
//! impl_downcast!(TraitGeneric1<T>);
//!
//! // With associated types.
//! trait TraitGeneric2: BoxAny { type G; type H; }
//! impl_downcast!(TraitGeneric2 assoc G, H);
//!
//! // With constraints on types.
//! trait TraitGeneric3<T: Copy>: BoxAny {
//!     type H: Clone;
//! }
//! impl_downcast!(TraitGeneric3<T> assoc H where T: Copy, H: Clone);
//!
//! // With concrete types.
//! trait TraitConcrete1<T: Copy>: BoxAny {}
//! impl_downcast!(concrete TraitConcrete1<u32>);
//!
//! trait TraitConcrete2<T: Copy>: BoxAny { type H; }
//! impl_downcast!(concrete TraitConcrete2<u32> assoc H=f64);
//! # fn main() {}
//! ```
//!
//! # Example without generics
//!
//! ```
//! // Import macro via `macro_use` pre-1.30.
//! #[macro_use]
//! extern crate any;
//! use any::BoxAny;
//!
//! // To create a trait with downcasting methods, extend `BoxAny` and run
//! // `impl_downcast!()` on the trait.
//! trait Base: BoxAny {}
//! impl_downcast!(Base);
//!
//! // Concrete types implementing Base.
//! #[derive(Debug)]
//! struct Foo(u32);
//! impl Base for Foo {}
//! #[derive(Debug)]
//! struct Bar(f64);
//! impl Base for Bar {}
//!
//! fn main() {
//!     // Create a trait object.
//!     let mut base: Box<Base> = Box::new(Foo(42));
//!
//!     // Try sequential downcasts.
//!     if let Some(foo) = base.downcast_ref::<Foo>() {
//!         assert_eq!(foo.0, 42);
//!     } else if let Some(bar) = base.downcast_ref::<Bar>() {
//!         assert_eq!(bar.0, 42.0);
//!     }
//!
//!     assert!(base.is::<Foo>());
//!
//!     // Fail to convert `Box<Base>` into `Box<Bar>`.
//!     let res = base.downcast::<Bar>();
//!     assert!(res.is_err());
//!     let base = res.unwrap_err();
//!     // Convert `Box<Base>` into `Box<Foo>`.
//!     assert_eq!(42, base.downcast::<Foo>().map_err(|_| "Shouldn't happen.").unwrap().0);
//! }
//! ```
//!
//! # Example with a generic trait with associated types and constraints
//!
//! ```
//! // Can call macro via namespace since rust 1.30.
//! extern crate any;
//! use any::BoxAny;
//!
//! // To create a trait with downcasting methods, extend `BoxAny` and run
//! // `impl_downcast!()` on the trait.
//! trait Base<T: Clone>: BoxAny { type H: Copy; }
//! downcast_rs::impl_downcast!(Base<T> assoc H where T: Clone, H: Copy);
//! // or: impl_downcast!(concrete Base<u32> assoc H=f32)
//!
//! // Concrete types implementing Base.
//! struct Foo(u32);
//! impl Base<u32> for Foo { type H = f32; }
//! struct Bar(f64);
//! impl Base<u32> for Bar { type H = f32; }
//!
//! fn main() {
//!     // Create a trait object.
//!     let mut base: Box<Base<u32, H=f32>> = Box::new(Bar(42.0));
//!
//!     // Try sequential downcasts.
//!     if let Some(foo) = base.downcast_ref::<Foo>() {
//!         assert_eq!(foo.0, 42);
//!     } else if let Some(bar) = base.downcast_ref::<Bar>() {
//!         assert_eq!(bar.0, 42.0);
//!     }
//!
//!     assert!(base.is::<Bar>());
//! }
//! ```

use std::any::Any;
use std::sync::Arc;
use std::rc::Rc;

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any { self }
}

pub trait AsMutAny: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsMutAny for T {
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub trait BoxAny: AsAny + AsMutAny {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: AsAny + AsMutAny> BoxAny for T {
     fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub trait RcAny: AsAny + 'static {
    fn into_any(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: AsAny> RcAny for T {
     fn into_any(self: Rc<Self>) -> Rc<dyn Any> { self }
}

pub trait ArcAny: AsAny + 'static + Send + Sync {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + 'static + Send + Sync>;
}

impl<T: AsAny + 'static + Send + Sync> ArcAny for T {
     fn into_any(self: Arc<Self>) -> Arc<dyn Any + 'static + Send + Sync> { self }
}


/// Adds downcasting support to traits that extend `any::BoxAny` by defining forwarding
/// methods to the corresponding implementations on `std::any::Any` in the standard library.
///
/// See https://users.rust-lang.org/t/how-to-create-a-macro-to-impl-a-provided-type-parametrized-trait/5289
/// for why this is implemented this way to support templatized traits.
#[macro_export(local_inner_macros)]
macro_rules! impl_downcast {
    (@impl_full   
        $trait_:ident [$($param_types:tt)*]
        for [$($forall_types:ident),*]
        where [$($preds:tt)*]
    ) => {
        impl_downcast! {
            @inject_where
                [impl<$($forall_types),*> dyn $trait_<$($param_types)*>]
                types [$($forall_types),*]
                where [$($preds)*]
                [{
                    impl_downcast! { @impl_body $trait_ [$($param_types)*] }
                    impl_downcast! { @impl_body_mut $trait_ [$($param_types)*] }
                }]
        }
    };

    (@impl_body_box $trait_:ident [$($types:tt)*]) => {
        /// Returns true if the trait object wraps an object of type `__T`.
        #[inline]
        pub fn downcast<__T: $trait_<$($types)*>>(
            self: ::std::boxed::Box<Self>
        ) -> ::std::result::Result<::std::boxed::Box<__T>, ::std::boxed::Box<Self>> {
            if self.is::<__T>() {
                Ok($crate::BoxAny::into_any(self).downcast::<__T>().unwrap())
            } else {
                Err(self)
            }
        }
    };

    (@impl_body_rc $trait_:ident [$($types:tt)*]) => {
        /// Returns true if the trait object wraps an object of type `__T`.
        #[inline]
        pub fn downcast<__T: $trait_<$($types)*>>(
            self: ::std::rc::Rc<Self>
        ) -> ::std::result::Result<::std::rc::Rc<__T>, ::std::rc::Rc<Self>> {
            if self.is::<__T>() {
                Ok($crate::RcAny::into_any(self).downcast::<__T>().unwrap())
            } else {
                Err(self)
            }
        }
    };

    (@impl_body_arc $trait_:ident [$($types:tt)*]) => {
        /// Returns true if the trait object wraps an object of type `__T`.
        #[inline]
        pub fn downcast<__T: $trait_<$($types)*>>(
            self: ::std::sync::Arc<Self>
        ) -> ::std::result::Result<::std::sync::Arc<__T>, ::std::sync::Arc<Self>> {
            if self.is::<__T>() {
                Ok($crate::ArcAny::into_any(self).downcast::<__T>().unwrap())
            } else {
                Err(self)
            }
        }
    };

    (@impl_body_mut $trait_:ident [$($types:tt)*]) => {
        /// Returns a mutable reference to the object within the trait object if it is of type
        /// `__T`, or `None` if it isn't.
        #[inline]
        pub fn downcast_mut<__T: $trait_<$($types)*>>(&mut self) -> ::std::option::Option<&mut __T> {
            $crate::AsMutAny::as_any_mut(self).downcast_mut::<__T>()
        }
    };

    (@impl_body $trait_:ident [$($types:tt)*]) => {
        /// Returns true if the trait object wraps an object of type `__T`.
        #[inline]
        pub fn is<__T: $trait_<$($types)*>>(&self) -> bool {
            $crate::AsAny::as_any(self).is::<__T>()
        }
        /// Returns a reference to the object within the trait object if it is of type `__T`, or
        /// `None` if it isn't.
        #[inline]
        pub fn downcast_ref<__T: $trait_<$($types)*>>(&self) -> ::std::option::Option<&__T> {
            $crate::AsAny::as_any(self).downcast_ref::<__T>()
        }   
    };

    (@inject_where [$($before:tt)*] types [] where [] [$($after:tt)*]) => {
        impl_downcast! { @as_item $($before)* $($after)* }
    };

    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [] [$($after:tt)*]) => {
        impl_downcast! {
            @as_item
                $($before)*
                where $( $types: ::std::any::Any + 'static ),*
                $($after)*
        }
    };
    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [$($preds:tt)+] [$($after:tt)*]) => {
        impl_downcast! {
            @as_item
                $($before)*
                where
                    $( $types: ::std::any::Any + 'static, )*
                    $($preds)*
                $($after)*
        }
    };

    (@as_item $i:item) => { $i };

    // No type parameters.
    ($trait_:ident   ) => { impl_downcast! { @impl_full $trait_ [] for [] where [] } };
    ($trait_:ident <>) => { impl_downcast! { @impl_full $trait_ [] for [] where [] } };
    // Type parameters.
    ($trait_:ident < $($types:ident),* >) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [$($types),*] where [] }
    };
    // Type parameters and where clauses.
    ($trait_:ident < $($types:ident),* > where $($preds:tt)+) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [$($types),*] where [$($preds)*] }
    };
    // Associated types.
    ($trait_:ident assoc $($atypes:ident),*) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [] }
    };
    // Associated types and where clauses.
    ($trait_:ident assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [$($preds)*] }
    };
    // Type parameters and associated types.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),*) => {
        impl_downcast! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where []
        }
    };
    // Type parameters, associated types, and where clauses.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where [$($preds)*]
        }
    };
    // Concretely-parametrized types.
    (concrete $trait_:ident < $($types:ident),* >) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [] where [] }
    };
    // Concretely-associated types types.
    (concrete $trait_:ident assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $aty),*] for [] where [] }
    };
    // Concretely-parametrized types with concrete associated types.
    (concrete $trait_:ident < $($types:ident),* > assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast! { @impl_full $trait_ [$($types),*, $($atypes = $aty),*] for [] where [] }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! impl_downcast_box {
    (@impl_full  
        $trait_:ident [$($param_types:tt)*]
        for [$($forall_types:ident),*]
        where [$($preds:tt)*]
    ) => {
        impl_downcast! {
            @inject_where
                [impl<$($forall_types),*> dyn $trait_<$($param_types)*>]
                types [$($forall_types),*]
                where [$($preds)*]
                [{
                    impl_downcast! { @impl_body $trait_ [$($param_types)*] }
                    impl_downcast! { @impl_body_box $trait_ [$($param_types)*] }
                    impl_downcast! { @impl_body_mut $trait_ [$($param_types)*] }
                }]
        }
    };

    (@inject_where [$($before:tt)*] types [] where [] [$($after:tt)*]) => {
        impl_downcast_box! { @as_item $($before)* $($after)* }
    };

    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [] [$($after:tt)*]) => {
        impl_downcast_box! {
            @as_item
                $($before)*
                where $( $types: ::std::any::Any + 'static ),*
                $($after)*
        }
    };
    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [$($preds:tt)+] [$($after:tt)*]) => {
        impl_downcast_box! {
            @as_item
                $($before)*
                where
                    $( $types: ::std::any::Any + 'static, )*
                    $($preds)*
                $($after)*
        }
    };

    (@as_item $i:item) => { $i };

    // No type parameters.
    ($trait_:ident   ) => { impl_downcast_box! { @impl_full $trait_ [] for [] where [] } };
    ($trait_:ident <>) => { impl_downcast_box! { @impl_full $trait_ [] for [] where [] } };
    // Type parameters.
    ($trait_:ident < $($types:ident),* >) => {
        impl_downcast_box! { @impl_full $trait_ [$($types),*] for [$($types),*] where [] }
    };
    // Type parameters and where clauses.
    ($trait_:ident < $($types:ident),* > where $($preds:tt)+) => {
        impl_downcast_box! { @impl_full $trait_ [$($types),*] for [$($types),*] where [$($preds)*] }
    };
    // Associated types.
    ($trait_:ident assoc $($atypes:ident),*) => {
        impl_downcast_box! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [] }
    };
    // Associated types and where clauses.
    ($trait_:ident assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_box! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [$($preds)*] }
    };
    // Type parameters and associated types.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),*) => {
        impl_downcast_box! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where []
        }
    };
    // Type parameters, associated types, and where clauses.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_box! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where [$($preds)*]
        }
    };
    // Concretely-parametrized types.
    (concrete $trait_:ident < $($types:ident),* >) => {
        impl_downcast_box! { @impl_full $trait_ [$($types),*] for [] where [] }
    };
    // Concretely-associated types types.
    (concrete $trait_:ident assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_box! { @impl_full $trait_ [$($atypes = $aty),*] for [] where [] }
    };
    // Concretely-parametrized types with concrete associated types.
    (concrete $trait_:ident < $($types:ident),* > assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_box! { @impl_full $trait_ [$($types),*, $($atypes = $aty),*] for [] where [] }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! impl_downcast_rc {
    (@impl_full   
        $trait_:ident [$($param_types:tt)*]
        for [$($forall_types:ident),*]
        where [$($preds:tt)*]
    ) => {
        impl_downcast! {
            @inject_where
                [impl<$($forall_types),*> dyn $trait_<$($param_types)*>]
                types [$($forall_types),*]
                where [$($preds)*]
                [{
                    impl_downcast! { @impl_body $trait_ [$($param_types)*] }
                    impl_downcast! { @impl_body_rc $trait_ [$($param_types)*] }
                }]
        }
    };

    (@inject_where [$($before:tt)*] types [] where [] [$($after:tt)*]) => {
        impl_downcast_rc! { @as_item $($before)* $($after)* }
    };

    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [] [$($after:tt)*]) => {
        impl_downcast_rc! {
            @as_item
                $($before)*
                where $( $types: ::std::any::Any + 'static ),*
                $($after)*
        }
    };
    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [$($preds:tt)+] [$($after:tt)*]) => {
        impl_downcast_rc! {
            @as_item
                $($before)*
                where
                    $( $types: ::std::any::Any + 'static, )*
                    $($preds)*
                $($after)*
        }
    };

    (@as_item $i:item) => { $i };

    // No type parameters.
    ($trait_:ident   ) => { impl_downcast_rc! { @impl_full $trait_ [] for [] where [] } };
    ($trait_:ident <>) => { impl_downcast_rc! { @impl_full $trait_ [] for [] where [] } };
    // Type parameters.
    ($trait_:ident < $($types:ident),* >) => {
        impl_downcast_rc! { @impl_full $trait_ [$($types),*] for [$($types),*] where [] }
    };
    // Type parameters and where clauses.
    ($trait_:ident < $($types:ident),* > where $($preds:tt)+) => {
        impl_downcast_rc! { @impl_full $trait_ [$($types),*] for [$($types),*] where [$($preds)*] }
    };
    // Associated types.
    ($trait_:ident assoc $($atypes:ident),*) => {
        impl_downcast_rc! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [] }
    };
    // Associated types and where clauses.
    ($trait_:ident assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_rc! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [$($preds)*] }
    };
    // Type parameters and associated types.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),*) => {
        impl_downcast_rc! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where []
        }
    };
    // Type parameters, associated types, and where clauses.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_rc! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where [$($preds)*]
        }
    };
    // Concretely-parametrized types.
    (concrete $trait_:ident < $($types:ident),* >) => {
        impl_downcast_rc! { @impl_full $trait_ [$($types),*] for [] where [] }
    };
    // Concretely-associated types types.
    (concrete $trait_:ident assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_rc! { @impl_full $trait_ [$($atypes = $aty),*] for [] where [] }
    };
    // Concretely-parametrized types with concrete associated types.
    (concrete $trait_:ident < $($types:ident),* > assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_rc! { @impl_full $trait_ [$($types),*, $($atypes = $aty),*] for [] where [] }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! impl_downcast_arc {
    (@impl_full   
        $trait_:ident [$($param_types:tt)*]
        for [$($forall_types:ident),*]
        where [$($preds:tt)*]
    ) => {
        impl_downcast! {
            @inject_where
                [impl<$($forall_types),*> dyn $trait_<$($param_types)*>]
                types [$($forall_types),*]
                where [$($preds)*]
                [{
                    impl_downcast! { @impl_body $trait_ [$($param_types)*] }
                    impl_downcast! { @impl_body_arc $trait_ [$($param_types)*] }
                }]
        }
    };

    (@inject_where [$($before:tt)*] types [] where [] [$($after:tt)*]) => {
        impl_downcast_arc! { @as_item $($before)* $($after)* }
    };

    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [] [$($after:tt)*]) => {
        impl_downcast_arc! {
            @as_item
                $($before)*
                where $( $types: ::std::any::Any + 'static ),*
                $($after)*
        }
    };
    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [$($preds:tt)+] [$($after:tt)*]) => {
        impl_downcast_arc! {
            @as_item
                $($before)*
                where
                    $( $types: ::std::any::Any + 'static, )*
                    $($preds)*
                $($after)*
        }
    };

    (@as_item $i:item) => { $i };

    // No type parameters.
    ($trait_:ident   ) => { impl_downcast_arc! { @impl_full $trait_ [] for [] where [] } };
    ($trait_:ident <>) => { impl_downcast_arc! { @impl_full $trait_ [] for [] where [] } };
    // Type parameters.
    ($trait_:ident < $($types:ident),* >) => {
        impl_downcast_arc! { @impl_full $trait_ [$($types),*] for [$($types),*] where [] }
    };
    // Type parameters and where clauses.
    ($trait_:ident < $($types:ident),* > where $($preds:tt)+) => {
        impl_downcast_arc! { @impl_full $trait_ [$($types),*] for [$($types),*] where [$($preds)*] }
    };
    // Associated types.
    ($trait_:ident assoc $($atypes:ident),*) => {
        impl_downcast_arc! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [] }
    };
    // Associated types and where clauses.
    ($trait_:ident assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_arc! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [$($preds)*] }
    };
    // Type parameters and associated types.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),*) => {
        impl_downcast_arc! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where []
        }
    };
    // Type parameters, associated types, and where clauses.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast_arc! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where [$($preds)*]
        }
    };
    // Concretely-parametrized types.
    (concrete $trait_:ident < $($types:ident),* >) => {
        impl_downcast_arc! { @impl_full $trait_ [$($types),*] for [] where [] }
    };
    // Concretely-associated types types.
    (concrete $trait_:ident assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_arc! { @impl_full $trait_ [$($atypes = $aty),*] for [] where [] }
    };
    // Concretely-parametrized types with concrete associated types.
    (concrete $trait_:ident < $($types:ident),* > assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast_arc! { @impl_full $trait_ [$($types),*, $($atypes = $aty),*] for [] where [] }
    };
}

// pub mod m;

#[cfg(test)]
mod test {
    macro_rules! test_mod {
        (
            $test_name:ident,
            trait $base_trait:path { $($base_impl:tt)* },
            type $base_type:ty,
            { $($def:tt)+ }
        ) => {
            mod $test_name {
				#[allow(unused_imports)]
                use crate::BoxAny;

                // A trait that can be downcast.
                $($def)*

                // Concrete type implementing Base.
                #[derive(Debug)]
                struct Foo(u32);
                impl $base_trait for Foo { $($base_impl)* }
                #[derive(Debug)]
                struct Bar(f64);
                impl $base_trait for Bar { $($base_impl)* }

                // Functions that can work on references to Base trait objects.
                fn get_val(base: &::std::boxed::Box<$base_type>) -> u32 {
                    match base.downcast_ref::<Foo>() {
                        Some(val) => val.0,
                        None => 0
                    }
                }
                fn set_val(base: &mut ::std::boxed::Box<$base_type>, val: u32) {
                    if let Some(foo) = base.downcast_mut::<Foo>() {
                        foo.0 = val;
                    }
                }

                #[test]
                fn test() {
                    let mut base: ::std::boxed::Box<$base_type> = ::std::boxed::Box::new(Foo(42));
                    assert_eq!(get_val(&base), 42);

                    // Try sequential downcasts.
                    if let Some(foo) = base.downcast_ref::<Foo>() {
                        assert_eq!(foo.0, 42);
                    } else if let Some(bar) = base.downcast_ref::<Bar>() {
                        assert_eq!(bar.0, 42.0);
                    }

                    set_val(&mut base, 6*9);
                    assert_eq!(get_val(&base), 6*9);

                    assert!(base.is::<Foo>());

                    // Fail to convert Box<Base> into Box<Bar>.
                    let res = base.downcast::<Bar>();
                    assert!(res.is_err());
                    let base = res.unwrap_err();
                    // Convert Box<Base> into Box<Foo>.
                    assert_eq!(
                        6*9, base.downcast::<Foo>().map_err(|_| "Shouldn't happen.").unwrap().0);
                }
            }
        };

        (
            $test_name:ident,
            trait $base_trait:path { $($base_impl:tt)* },
            { $($def:tt)+ }
        ) => {
            test_mod! {
                $test_name, trait $base_trait { $($base_impl:tt)* }, type dyn $base_trait, { $($def)* }
            }
        }
    }

    test_mod!(non_generic, trait Base {}, {
        trait Base: BoxAny {}
        impl_downcast_box!(Base);
    });

    test_mod!(generic, trait Base<u32> {}, {
        trait Base<T>: BoxAny {}
        impl_downcast_box!(Base<T>);
    });

    test_mod!(constrained_generic, trait Base<u32> {}, {
        // Should work even if standard objects in the prelude are aliased to something else.
        #[allow(dead_code)] struct Box;
        #[allow(dead_code)] struct Option;
        #[allow(dead_code)] struct Result;
        trait Base<T: Copy>: BoxAny {}
        impl_downcast_box!(Base<T> where T: Copy);
    });

    test_mod!(associated, trait Base { type H = f32; }, type dyn Base<H=f32>, {
        trait Base: BoxAny { type H; }
        impl_downcast_box!(Base assoc H);
    });

    test_mod!(constrained_associated, trait Base { type H = f32; }, type dyn Base<H=f32>, {
        trait Base: BoxAny { type H: Copy; }
        impl_downcast_box!(Base assoc H where H: Copy);
    });

    test_mod!(param_and_associated, trait Base<u32> { type H = f32; }, type dyn Base<u32, H=f32>, {
        trait Base<T>: BoxAny { type H; }
        impl_downcast_box!(Base<T> assoc H);
    });

    test_mod!(constrained_param_and_associated, trait Base<u32> { type H = f32; }, type dyn Base<u32, H=f32>, {
        trait Base<T: Clone>: BoxAny { type H: Copy; }
        impl_downcast_box!(Base<T> assoc H where T: Clone, H: Copy);
    });

    test_mod!(concrete_parametrized, trait Base<u32> {}, {
        trait Base<T>: BoxAny {}
        impl_downcast_box!(concrete Base<u32>);
    });

    test_mod!(concrete_associated, trait Base { type H = u32; }, type dyn Base<H=u32>, {
        trait Base: BoxAny { type H; }
        impl_downcast_box!(concrete Base assoc H=u32);
    });

    test_mod!(concrete_parametrized_associated, trait Base<u32> { type H = f32; }, type dyn Base<u32, H=f32>, {
        trait Base<T>: crate::BoxAny { type H; }
        impl_downcast_box!(concrete Base<u32> assoc H=f32);
    });
}