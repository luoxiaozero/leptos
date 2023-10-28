//! Callbacks define a standard way to store functions and closures. They are useful
//! for component properties, because they can be used to define optional callback functions,
//! which generic props don’t support.
//!
//! # Usage
//! Callbacks can be created manually from any function or closure, but the easiest way
//! to create them is to use `#[prop(into)]]` when defining a component.
//! ```
//! # use leptos::*;
//! #[component]
//! fn MyComponent(
//!     #[prop(into)] render_number: Callback<i32, String>,
//! ) -> impl IntoView {
//!     view! {
//!         <div>
//!             {render_number.call(1)}
//!             // callbacks can be called multiple times
//!             {render_number.call(42)}
//!         </div>
//!     }
//! }
//! // you can pass a closure directly as `render_number`
//! fn test() -> impl IntoView {
//!     view! {
//!         <MyComponent render_number=|x: i32| x.to_string()/>
//!     }
//! }
//! ```
//!
//! *Notes*:
//! - The `render_number` prop can receive any type that implements `Fn(i32) -> String`.
//! - Callbacks are most useful when you want optional generic props.
//! - All callbacks implement the [`Callable`] trait, and can be invoked with `my_callback.call(input)`. On nightly, you can even do `my_callback(input)`
//! - The callback types implement [`Copy`], so they can easily be moved into and out of other closures, just like signals.
//!
//! # Types
//! This modules implements 2 callback types:
//! - [`Callback`]
//! - [`SyncCallback`]
//!
//! Use `SyncCallback` when you want the function to be `Sync` and `Send`.

use crate::{store_value, StoredValue};
use std::{fmt, future::Future, pin::Pin, rc::Rc, sync::Arc};

/// A wrapper trait for calling callbacks.
pub trait Callable<In: 'static, Out: 'static = ()> {
    /// calls the callback with the specified argument.
    fn call(&self, input: In) -> Out;
}

/// Callbacks define a standard way to store functions and closures.
///
/// # Example
/// ```
/// # use leptos::*;
/// # use leptos::{Callable, Callback};
/// #[component]
/// fn MyComponent(
///     #[prop(into)] render_number: Callback<i32, String>,
/// ) -> impl IntoView {
///     view! {
///         <div>
///             {render_number.call(42)}
///         </div>
///     }
/// }
///
/// fn test() -> impl IntoView {
///     view! {
///         <MyComponent render_number=move |x: i32| x.to_string()/>
///     }
/// }
/// ```

pub struct Callback<In: 'static, Out: 'static = ()>(
    StoredValue<Box<dyn Fn(In) -> Out>>,
);

impl<In> fmt::Debug for Callback<In> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str("Callback")
    }
}

impl<In, Out> Clone for Callback<In, Out> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<In, Out> Copy for Callback<In, Out> {}

impl<In, Out> Callback<In, Out> {
    /// Creates a new callback from the given function.
    pub fn new<F>(f: F) -> Callback<In, Out>
    where
        F: Fn(In) -> Out + 'static,
    {
        Self(store_value(Box::new(f)))
    }
}

impl<In: 'static, Out: 'static> Callable<In, Out> for Callback<In, Out> {
    fn call(&self, input: In) -> Out {
        self.0.with_value(|f| f(input))
    }
}

macro_rules! impl_from_fn {
    ($ty:ident) => {
        #[cfg(not(feature = "nightly"))]
        impl<F, In, T, Out> From<F> for $ty<In, Out>
        where
            F: Fn(In) -> T + 'static,
            T: Into<Out> + 'static,
        {
            fn from(f: F) -> Self {
                Self::new(move |x| f(x).into())
            }
        }

        paste::paste! {
            #[cfg(feature = "nightly")]
            auto trait [<NotRaw $ty>] {}

            #[cfg(feature = "nightly")]
            impl<A, B> ![<NotRaw $ty>] for $ty<A, B> {}

            #[cfg(feature = "nightly")]
            impl<F, In, T, Out> From<F> for $ty<In, Out>
            where
                F: Fn(In) -> T + [<NotRaw $ty>] + 'static,
                T: Into<Out> + 'static,
            {
                fn from(f: F) -> Self {
                    Self::new(move |x| f(x).into())
                }
            }
        }
    };
}

impl_from_fn!(Callback);

#[cfg(feature = "nightly")]
impl<In, Out> FnOnce<(In,)> for Callback<In, Out> {
    type Output = Out;

    extern "rust-call" fn call_once(self, args: (In,)) -> Self::Output {
        Callable::call(&self, args.0)
    }
}

#[cfg(feature = "nightly")]
impl<In, Out> FnMut<(In,)> for Callback<In, Out> {
    extern "rust-call" fn call_mut(&mut self, args: (In,)) -> Self::Output {
        Callable::call(&*self, args.0)
    }
}

#[cfg(feature = "nightly")]
impl<In, Out> Fn<(In,)> for Callback<In, Out> {
    extern "rust-call" fn call(&self, args: (In,)) -> Self::Output {
        Callable::call(self, args.0)
    }
}

/// A callback type that is `Send` and `Sync` if its input type is `Send` and `Sync`.
/// Otherwise, you can use exactly the way you use [`Callback`].
pub struct SyncCallback<In: 'static, Out: 'static = ()>(
    StoredValue<Arc<dyn Fn(In) -> Out>>,
);

impl<In> fmt::Debug for SyncCallback<In> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str("SyncCallback")
    }
}

impl<In, Out> Callable<In, Out> for SyncCallback<In, Out> {
    fn call(&self, input: In) -> Out {
        self.0.with_value(|f| f(input))
    }
}

impl<In, Out> Clone for SyncCallback<In, Out> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<In: 'static, Out: 'static> SyncCallback<In, Out> {
    /// Creates a new callback from the given function.
    pub fn new<F>(fun: F) -> Self
    where
        F: Fn(In) -> Out + 'static,
    {
        Self(store_value(Arc::new(fun)))
    }
}

impl_from_fn!(SyncCallback);

#[cfg(feature = "nightly")]
impl<In, Out> FnOnce<(In,)> for SyncCallback<In, Out> {
    type Output = Out;

    extern "rust-call" fn call_once(self, args: (In,)) -> Self::Output {
        Callable::call(&self, args.0)
    }
}

#[cfg(feature = "nightly")]
impl<In, Out> FnMut<(In,)> for SyncCallback<In, Out> {
    extern "rust-call" fn call_mut(&mut self, args: (In,)) -> Self::Output {
        Callable::call(&*self, args.0)
    }
}

#[cfg(feature = "nightly")]
impl<In, Out> Fn<(In,)> for SyncCallback<In, Out> {
    extern "rust-call" fn call(&self, args: (In,)) -> Self::Output {
        Callable::call(self, args.0)
    }
}

/// Callbacks define a standard way to store functions and closures.
///
/// `Callback` asynchronous version
///
/// # Example
/// ```
/// # use leptos::*;
/// # use leptos::AsyncCallback;
/// #[component]
/// fn MyComponent(
///     #[prop(into)] on_add: AsyncCallback<i32, bool>,
/// ) -> impl IntoView {
///     let number = create_rw_signal(0);
///     let on_click = move |_| {
///         let allow = on_add.call(number.get_untracked()).await;
///         if allow {
///             number.update(|num| *num += 1);
///         }
///     };
///     view! {
///         <button on:click=on_click>
///             "Click: "
///             {move || number.get()}
///         </button>
///     }
/// }
///
/// fn test() -> impl IntoView {
///     let on_add = move |num| {
///         async move {
///             // Add asynchronous logic
///             num != 3
///         }
///     };
///     view! {
///         <MyComponent on_add />
///     }
/// }
/// ```
pub struct AsyncCallback<In: 'static, Out: 'static = ()>(
    #[allow(clippy::complexity)]
    StoredValue<Rc<dyn Fn(In) -> Pin<Box<dyn Future<Output = Out>>>>>,
);

impl<In> fmt::Debug for AsyncCallback<In> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str("AsyncCallback")
    }
}

impl<In, Out> Clone for AsyncCallback<In, Out> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<In, Out> Copy for AsyncCallback<In, Out> {}

impl<In, Out> AsyncCallback<In, Out> {
    /// Creates a new callback from the given function.
    pub fn new<F, Fu>(f: F) -> Self
    where
        F: Fn(In) -> Fu + 'static,
        Fu: Future<Output = Out> + 'static,
    {
        let f = Rc::new(move |input: In| {
            let fut = f(input);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Out>>>
        });
        Self(StoredValue::new(f))
    }

    /// calls the callback with the specified argument.
    pub async fn call(&self, input: In) -> Out {
        let f = self.0.get_value();
        f(input).await
    }
}

impl<F, In, Fu, Out> From<F> for AsyncCallback<In, Out>
where
    F: Fn(In) -> Fu + 'static,
    Fu: Future<Output = Out> + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        callback::{AsyncCallback, Callback, SyncCallback},
        create_runtime,
    };

    struct NoClone {}

    #[test]
    fn clone_callback() {
        let rt = create_runtime();
        let callback = Callback::new(move |_no_clone: NoClone| NoClone {});
        let _cloned = callback.clone();
        rt.dispose();
    }

    #[test]
    fn clone_sync_callback() {
        let rt = create_runtime();
        let callback = SyncCallback::new(move |_no_clone: NoClone| NoClone {});
        let _cloned = callback.clone();
        rt.dispose();
    }

    #[test]
    fn clone_async_callback() {
        let rt = create_runtime();
        let callback =
            AsyncCallback::new(move |_no_clone: NoClone| async { NoClone {} });
        let _cloned = callback.clone();
        rt.dispose();
    }

    #[test]
    fn callback_from() {
        let rt = create_runtime();
        let _callback: Callback<(), String> = (|()| "test").into();
        rt.dispose();
    }

    #[test]
    fn callback_from_html() {
        let rt = create_runtime();
        use leptos::{
            html::{AnyElement, HtmlElement},
            *,
        };

        let _callback: Callback<String, HtmlElement<AnyElement>> =
            (|x: String| {
                view! {
                    <h1>{x}</h1>
                }
            })
            .into();
        rt.dispose();
    }

    #[test]
    fn sync_callback_from() {
        let rt = create_runtime();
        let _callback: SyncCallback<(), String> = (|()| "test").into();
        rt.dispose();
    }

    #[test]
    fn sync_callback_from_html() {
        use leptos::{
            html::{AnyElement, HtmlElement},
            *,
        };

        let rt = create_runtime();

        let _callback: SyncCallback<String, HtmlElement<AnyElement>> =
            (|x: String| {
                view! {
                    <h1>{x}</h1>
                }
            })
            .into();

        rt.dispose();
    }

    #[test]
    fn async_callback_from() {
        let rt = create_runtime();
        let _callback: AsyncCallback<(), String> =
            (|()| async { "test".to_string() }).into();
        rt.dispose();
    }

    #[test]
    fn async_callback_from_html() {
        let rt = create_runtime();
        use leptos::{
            html::{HtmlElement, H1},
            *,
        };

        let _callback: AsyncCallback<String, HtmlElement<H1>> =
            (|x: String| async move {
                view! {
                    <h1>{x}</h1>
                }
            })
            .into();
        rt.dispose();
    }
}
