//!
//! @file invoker.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief A macro to generate a structure for giving closures to C FFI.
//! @bug No known bugs.
//!

use crate::abstract_type;

abstract_type! {
    /// @brief Abstract pointer type of all invokers that are being passed for Fn.
    pub type Invoker;

    /// @brief Abstract pointer type of all invokers that are being passed for FnMut.
    pub type InvokerMut;

    /// @brief Abstract pointer type of all invokers that are being passed for FnOnce.
    pub type InvokerOnce;
}

///
/// @brief Used to generate Invoker structures.
///
/// The structures will be generated under a module with the given name. The module will contain
/// the invoker type and the type of the C-FFI function it uses.
///
#[macro_export]
macro_rules! invoker {
    // Primary match arm, generates the invokers.
    ( $($(#[$meta:meta])*
    $scope:vis type $name:ident = $trait:ident(
        $($($pre_args:ident: $pre_types:ty),+,)?
        @this
        $(,$($post_args:ident: $post_types:ty),+)?
    ) $(-> $ret:ty)? );+; ) => {
        $(
            #[allow(non_snake_case)]
            $scope mod $name {
                $(#[$meta])* #[repr(C)] pub struct Invoker<F> {
                    func: $crate::core::option::Option<F>
                }

                $crate::invoker! { @impl $trait(
                    $($($pre_types),*)?;
                    $($($post_types),*)?
                ) $(-> $ret)? {
                    ///
                    /// @brief Creates a new invoker with the given closure
                    ///
                    pub fn new(
                        func: F
                    ) -> Self {
                        Self {
                            func: $crate::core::option::Option::Some(func)
                        }
                    }

                    ///
                    /// @brief Returns a reference to this invokers associated invoke function.
                    ///
                    pub fn as_fn(
                        &mut self
                    ) -> super::$name::Function {
                        Self::invoke
                    }
                }}

                $crate::invoker! {
                    @fn $trait(
                        $($($pre_args: $pre_types),*,)?
                        @this
                        $(,$($post_args: $post_types),*)?
                    ) $(-> $ret)?
                }
            }
        )*
    };

    // Generate impl blocks for the function type.

    ( @impl $trait:ident(;) $(-> $ret:ty)? { $($body:tt)* } ) => {
        impl<F: $trait() $(-> $ret)?> Invoker<F> { $($body)* }
    };

    ( @impl $trait:ident($($pre_types:ty),+;) $(-> $ret:ty)? { $($body:tt)* } ) => {
        impl<F: $trait($($pre_types),*) $(-> $ret)?> Invoker<F> { $($body)* }
    };

    ( @impl $trait:ident(;$($post_types:ty),+) $(-> $ret:ty)? { $($body:tt)* } ) => {
        impl<F: $trait($($post_types),*) $(-> $ret)?> Invoker<F> { $($body)* }
    };

    ( @impl $trait:ident(
        $($pre_types:ty),+;
        $($post_types:ty),+
    ) $(-> $ret:ty)? { $($body:tt)* } ) => {
        impl<F: $trait($($pre_types),*, $($post_types),*) $(-> $ret)?> Invoker<F> { $($body)* }
    };

    // Generate implementations.

    ( @fn FnOnce(
        $($($pre_args:ident: $pre_types:ty),*,)?
        @this
        $(,$($post_args:ident: $post_types:ty),*)?
    ) $(-> $ret:ty)? ) => {
        ///
        /// @brief The type for the FFI function for this invoker.
        ///
        pub type Function = unsafe extern "C" fn(
            $($($pre_types),*,)?
            *mut $crate::InvokerOnce
            $(,$($post_types),*)?
        ) $(-> $ret)?;

        $crate::invoker! { @impl FnOnce(
            $($($pre_types),*)?;
            $($($post_types),*)?
        ) $(-> $ret)? {
            ///
            /// @brief Creates an InvokerOnce pointer from this invoker.
            ///
            pub fn as_invoker(
                &mut self
            ) -> *mut $crate::InvokerOnce {
                self as *mut Self as *mut $crate::InvokerOnce
            }

            ///
            /// @brief Invokes the given invoker exactly once.
            ///
            /// The caller must ensure that the given invoker is forgotten at the call sight,
            /// as it will be dropped by this function.
            ///
            pub unsafe extern "C" fn invoke(
                $($($pre_args: $pre_types),*,)?
                this: *mut $crate::InvokerOnce
                $(,$($post_args: $post_types),*)?
            ) $(-> $ret)? {
                // Make sure "this" is the correct type. The user might trick us into generating
                // the wrong one.
                let _: *mut $crate::InvokerOnce = this;

                let func = (this as *mut Self).as_mut().unwrap().func.take().unwrap();
                (func)($($($pre_args),*,)? $($($post_args),*)?)
            }
        }}
    };

    ( @fn FnMut(
        $($($pre_args:ident: $pre_types:ty),*,)?
        @this
        $(,$($post_args:ident: $post_types:ty),*)?
    ) $(-> $ret:ty)? ) => {
        ///
        /// @brief The type for the FFI function for this invoker.
        ///
        pub type Function = unsafe extern "C" fn(
            $($($pre_types),*,)?
            *mut $crate::InvokerMut
            $(,$($post_types),*)?
        ) $(-> $ret)?;

        $crate::invoker! { @impl FnMut(
            $($($pre_types),*)?;
            $($($post_types),*)?
        ) $(-> $ret)? {
            ///
            /// @brief Creates an InvokerMut pointer from this invoker.
            ///
            pub fn as_invoker(
                &mut self
            ) -> *mut $crate::InvokerMut {
                self as *mut Self as *mut $crate::InvokerMut
            }

            ///
            /// @brief Invokes the given mutable invoker.
            ///
            /// The caller must ensure that the given invoker is forgotten at the call sight,
            /// as it will be dropped by this function.
            ///
            pub unsafe extern "C" fn invoke(
                $($($pre_args: $pre_types),*,)?
                this: *mut $crate::InvokerMut
                $(,$($post_args: $post_types),*)?
            ) $(-> $ret)? {
                // Make sure "this" is the correct type. The user might trick us into generating
                // the wrong one.
                let _: *mut $crate::InvokerMut = this;

                let func = (this as *mut Self).as_mut().unwrap().func.as_mut().unwrap();
                (func)($($($pre_args),*,)? $($($post_args),*)?)
            }
        }}
    };

    ( @fn Fn(
        $($($pre_args:ident: $pre_types:ty),*,)?
        @this
        $(,$($post_args:ident: $post_types:ty),*)?
    ) $(-> $ret:ty)? ) => {
        ///
        /// @brief The type for the FFI function for this invoker.
        ///
        pub type Function = unsafe extern "C" fn(
            $($($pre_types),*,)?
            *const $crate::Invoker
            $(,$($post_types),*)?
        ) $(-> $ret)?;

        $crate::invoker! { @impl Fn(
            $($($pre_types),*)?;
            $($($post_types),*)?
        ) $(-> $ret)? {
            ///
            /// @brief Creates an Invoker pointer from this invoker.
            ///
            pub fn as_invoker(
                &self
            ) -> *const $crate::Invoker {
                self as *const Self as *const $crate::Invoker
            }

            ///
            /// @brief Invokes the given constant invoker.
            ///
            pub unsafe extern "C" fn invoke(
                $($($pre_args: $pre_types),*,)?
                this: *const $crate::Invoker
                $(,$($post_args: $post_types),*)?
            ) $(-> $ret)? {
                // Make sure "this" is the correct type. The user might trick us into generating
                // the wrong one.
                let _: *mut $crate::Invoker = this;

                let func = (this as *const Self).as_ref().unwrap().func.as_ref().unwrap();
                (func)($($($pre_args),*,)? $($($post_args),*)?)
            }
        }}
    };
}
