//! This library allows to print manually-maintained messages on panic.
//!
//! When your program panics, it prints backtrace. However, if panic
//! occurs inside loop, it is not clear which iteration was the cause.
//! It is possible to use log, but printing slows down execution considerably
//! and you get lots of entries, while only last ones are required.
//!
//! Panic context lets you set value which is remembered, but not printed anywhere
//! until panic occurs. It is also automatically forgotten at the end of scope.
//!
//! # Example
//!
//! ```should_panic
//! #[macro_use] extern crate panic_context;
//!
//! use panic_context::panic_context;
//!
//! static ITEMS: &[&str] = &["foo", "bar", "yo", "nope"];
//!
//! fn get_len(item: &str) -> usize { item.len() }
//! fn calc_sig(item: &str) -> &str { &item[3..] }
//!
//! fn main() {
//!     let step = panic_context("step: ");
//!
//!     step.update("calculate lengths");
//!     for item in ITEMS {
//!         panic_context!("item: {}", item);
//!         get_len(item);
//!     }
//!
//!     step.update("calculate signatures");
//!     for item in ITEMS {
//!         panic_context!("item: {}", item);
//!         calc_sig(item);
//!     }
//!
//!     panic!("boom!");
//! }
//! ```
//!
//! When this code panics inside `calc_sig`, you will see:
//!
//! ```text
//! Panic context:
//! step: calculate signatures
//! item: yo
//! thread 'main' panicked at '...', src/libcore/str/mod.rs:2162
//! note: Run with `RUST_BACKTRACE=1` for a backtrace.
//! ```

#![doc(html_root_url="https://docs.rs/panic-context/0.1.0")]

#[macro_use] extern crate lazy_static;

use std::panic;
use std::collections::BTreeMap;
use std::cell::RefCell;
use std::sync::Mutex;

use std::io::Write;

lazy_static! {
    static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
}

struct Values {
    next_id: usize,
    values: BTreeMap<usize, String>,
}
thread_local! {
    static VALUES: RefCell<Values> = RefCell::new(Values {
        next_id: 0,
        values: BTreeMap::new(),
    });
}

/// Initializes the panic hook.
///
/// After this method is called, all panics will be logged rather than printed
/// to standard error.
fn init() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        VALUES.with(|traces| {
            let traces = traces.borrow();
            let stderr = std::io::stderr();
            let mut handle = stderr.lock();
            let _ = handle.write(b"Panic context:\n");
            for (_, value) in traces.values.iter() {
                let _ = handle.write(value.as_bytes()).unwrap();
                let _ = handle.write(b"\n").unwrap();
            }
        });
        previous_hook(info);
    }));
}

fn ensure_initialized() {
    let mut initialized = INITIALIZED.lock().unwrap();
    if !*initialized {
        init();
        *initialized = true;
    }
}

fn add_entry(value: Option<String>) -> usize {
    VALUES.with(move |values| {
        let mut values = values.borrow_mut();
        let id = values.next_id;
        values.next_id += 1;
        if let Some(v) = value {
            values.values.insert(id, v);
        }
        id
    })
}

fn update_entry(id: usize, value: String) {
    VALUES.with(|values| {
        let mut values = values.borrow_mut();
        values.values.insert(id, value);
    })
}

#[must_use]
pub struct UpdatablePanicContext {
    id: usize,
    prefix: &'static str,
}
impl UpdatablePanicContext {
    pub fn new(prefix: &'static str) -> Self {
        ensure_initialized();
        let id = add_entry(None);
        UpdatablePanicContext { id, prefix }
    }

    pub fn update<T: Into<String>>(&self, value: T) {
        let mut buf = self.prefix.to_string();
        buf += &value.into();
        update_entry(self.id, buf);
    }
}

#[must_use]
pub struct PanicContext {
    id: usize,
}
impl PanicContext {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        ensure_initialized();

        let id = VALUES.with(|values| {
            let mut values = values.borrow_mut();
            let id = values.next_id;
            values.next_id += 1;
            values.values.insert(id, msg.into());
            id
        });
        PanicContext { id }
    }
}

impl Drop for PanicContext {
    fn drop(&mut self) {
        VALUES.with(|values| {
            let mut values = values.borrow_mut();
            values.values.remove(&self.id)
        });
    }
}

/// Creates panic context whose message may be updated.
///
/// `prefix` string will be prepended to each value provided to
/// `update` method.
///
/// # Usage
///
/// Bind panic context and use `update` method to update message.
/// Message is automatically removed when panic context goes out
/// of scope.
///
/// # Example
///
/// ```no_run
/// # use panic_context::panic_context;
/// # fn main() {
/// let step = panic_context("step: ");
/// step.update("calculate lengths");
/// // ...
/// step.update("calculate signatures");
/// // ...
/// panic!("boom!");
/// # }
/// ```
///
/// Result:
///
/// ```text
/// Panic context:
/// step: calculate signatures
/// thread 'main' panicked at '...', src/libcore/str/mod.rs:2162
/// note: Run with `RUST_BACKTRACE=1` for a backtrace.
/// ```

pub fn panic_context(prefix: &'static str) -> UpdatablePanicContext {
    UpdatablePanicContext::new(prefix)
}

/// Remembers message to show it on panic inside current scope.
///
/// All arguments are passed to `format!` in order to compose message.
/// Context is forgotten at the end of current scope.
///
/// # Uses
///
/// Message is always remembered in both debug and release builds.
/// See [`debug_panic_context!`] for context which is not enabled in
/// release builds by default.
///
/// [`debug_panic_context!`]: macro.debug_panic_context.html
///
/// # Examples
///
/// ```no_run
/// #[macro_use] extern crate panic_context;
/// # fn main() {
/// # let items: &[&str] = &[];
/// # fn process_item(_: &str) {}
/// for item in items {
///     panic_context!("item: {}", item);
///     process_item(item);
/// }
/// # }
/// ```
///
/// If panic occurs during item processing, then panic context will
/// be printed:
///
/// ```text
/// Panic context:
/// item: cucumber
/// thread 'main' panicked at '...', src/libcore/str/mod.rs:2162
/// note: Run with `RUST_BACKTRACE=1` for a backtrace.
/// ```
#[macro_export]
macro_rules! panic_context {
    ($($arg:tt)+) => (
        let _panic_context = $crate::PanicContext::new(format!($($arg)+));
    )
}

/// Remembers message to show it on panic inside current scope in debug build only.
///
/// Message is forgotten at the end of current scope.
///
/// Unlike [`panic_context!`], `debug_panic_context!` is only enabled in
/// non-optimized builds by default. An optimized build will omit
/// all `debug_panic_context!` statements unless '-C debug-assertions'
/// is passed to the compiler or 'keep-debug-context' crate feature is
/// requested.
///
/// [`panic_context!`]: macro.panic_context.html
#[macro_export]
macro_rules! debug_panic_context {
    ($($arg:tt)+) => (
        #[cfg(or(debug-assertions, keep-debug-context))]
        let _panic_context = $crate::PanicContext::new_with_msg(format!($($arg)+));
    )
}
