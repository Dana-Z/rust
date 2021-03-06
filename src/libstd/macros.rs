// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Standard library macros
//!
//! This modules contains a set of macros which are exported from the standard
//! library. Each macro is available for use when linking against the standard
//! library.

#![unstable(feature = "std_misc")]

/// The entry point for panic of Rust tasks.
///
/// This macro is used to inject panic into a Rust task, causing the task to
/// unwind and panic entirely. Each task's panic can be reaped as the
/// `Box<Any>` type, and the single-argument form of the `panic!` macro will be
/// the value which is transmitted.
///
/// The multi-argument form of this macro panics with a string and has the
/// `format!` syntax for building a string.
///
/// # Example
///
/// ```should_fail
/// # #![allow(unreachable_code)]
/// panic!();
/// panic!("this is a terrible mistake!");
/// panic!(4); // panic with the value of 4 to be collected elsewhere
/// panic!("this is a {} {message}", "fancy", message = "message");
/// ```
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
macro_rules! panic {
    () => ({
        panic!("explicit panic")
    });
    ($msg:expr) => ({
        $crate::rt::begin_unwind($msg, {
            // static requires less code at runtime, more constant data
            static _FILE_LINE: (&'static str, usize) = (file!(), line!());
            &_FILE_LINE
        })
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::begin_unwind_fmt(format_args!($fmt, $($arg)+), {
            // The leading _'s are to avoid dead code warnings if this is
            // used inside a dead function. Just `#[allow(dead_code)]` is
            // insufficient, since the user may have
            // `#[forbid(dead_code)]` and which cannot be overridden.
            static _FILE_LINE: (&'static str, usize) = (file!(), line!());
            &_FILE_LINE
        })
    });
}

/// Use the syntax described in `std::fmt` to create a value of type `String`.
/// See `std::fmt` for more information.
///
/// # Example
///
/// ```
/// format!("test");
/// format!("hello {}", "world!");
/// format!("x = {}, y = {y}", 10, y = 30);
/// ```
#[cfg(stage0)] // NOTE: remove after snapshot
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
macro_rules! format {
    ($($arg:tt)*) => ($crate::fmt::format(format_args!($($arg)*)))
}

/// Equivalent to the `println!` macro except that a newline is not printed at
/// the end of the message.
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
macro_rules! print {
    ($($arg:tt)*) => ($crate::old_io::stdio::print_args(format_args!($($arg)*)))
}

/// Macro for printing to a task's stdout handle.
///
/// Each task can override its stdout handle via `std::old_io::stdio::set_stdout`.
/// The syntax of this macro is the same as that used for `format!`. For more
/// information, see `std::fmt` and `std::old_io::stdio`.
///
/// # Example
///
/// ```
/// println!("hello there!");
/// println!("format {} arguments", "some");
/// ```
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
macro_rules! println {
    ($($arg:tt)*) => ($crate::old_io::stdio::println_args(format_args!($($arg)*)))
}

/// Helper macro for unwrapping `Result` values while returning early with an
/// error if the value of the expression is `Err`. For more information, see
/// `std::io`.
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
macro_rules! try {
    ($expr:expr) => (match $expr {
        $crate::result::Result::Ok(val) => val,
        $crate::result::Result::Err(err) => {
            return $crate::result::Result::Err($crate::error::FromError::from_error(err))
        }
    })
}

/// A macro to select an event from a number of receivers.
///
/// This macro is used to wait for the first event to occur on a number of
/// receivers. It places no restrictions on the types of receivers given to
/// this macro, this can be viewed as a heterogeneous select.
///
/// # Examples
///
/// ```
/// use std::thread::Thread;
/// use std::sync::mpsc;
///
/// // two placeholder functions for now
/// fn long_running_task() {}
/// fn calculate_the_answer() -> u32 { 42 }
///
/// let (tx1, rx1) = mpsc::channel();
/// let (tx2, rx2) = mpsc::channel();
///
/// Thread::spawn(move|| { long_running_task(); tx1.send(()).unwrap(); });
/// Thread::spawn(move|| { tx2.send(calculate_the_answer()).unwrap(); });
///
/// select! (
///     _ = rx1.recv() => println!("the long running task finished first"),
///     answer = rx2.recv() => {
///         println!("the answer was: {}", answer.unwrap());
///     }
/// )
/// ```
///
/// For more information about select, see the `std::sync::mpsc::Select` structure.
#[macro_export]
#[unstable(feature = "std_misc")]
macro_rules! select {
    (
        $($name:pat = $rx:ident.$meth:ident() => $code:expr),+
    ) => ({
        use $crate::sync::mpsc::Select;
        let sel = Select::new();
        $( let mut $rx = sel.handle(&$rx); )+
        unsafe {
            $( $rx.add(); )+
        }
        let ret = sel.wait();
        $( if ret == $rx.id() { let $name = $rx.$meth(); $code } else )+
        { unreachable!() }
    })
}

// When testing the standard library, we link to the liblog crate to get the
// logging macros. In doing so, the liblog crate was linked against the real
// version of libstd, and uses a different std::fmt module than the test crate
// uses. To get around this difference, we redefine the log!() macro here to be
// just a dumb version of what it should be.
#[cfg(test)]
macro_rules! log {
    ($lvl:expr, $($args:tt)*) => (
        if log_enabled!($lvl) { println!($($args)*) }
    )
}

/// Built-in macros to the compiler itself.
///
/// These macros do not have any corresponding definition with a `macro_rules!`
/// macro, but are documented here. Their implementations can be found hardcoded
/// into libsyntax itself.
#[cfg(dox)]
pub mod builtin {
    /// The core macro for formatted string creation & output.
    ///
    /// This macro produces a value of type `fmt::Arguments`. This value can be
    /// passed to the functions in `std::fmt` for performing useful functions.
    /// All other formatting macros (`format!`, `write!`, `println!`, etc) are
    /// proxied through this one.
    ///
    /// For more information, see the documentation in `std::fmt`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::fmt;
    ///
    /// let s = fmt::format(format_args!("hello {}", "world"));
    /// assert_eq!(s, format!("hello {}", "world"));
    ///
    /// ```
    #[macro_export]
    macro_rules! format_args { ($fmt:expr, $($args:tt)*) => ({
        /* compiler built-in */
    }) }

    /// Inspect an environment variable at compile time.
    ///
    /// This macro will expand to the value of the named environment variable at
    /// compile time, yielding an expression of type `&'static str`.
    ///
    /// If the environment variable is not defined, then a compilation error
    /// will be emitted.  To not emit a compile error, use the `option_env!`
    /// macro instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// let path: &'static str = env!("PATH");
    /// println!("the $PATH variable at the time of compiling was: {}", path);
    /// ```
    #[macro_export]
    macro_rules! env { ($name:expr) => ({ /* compiler built-in */ }) }

    /// Optionally inspect an environment variable at compile time.
    ///
    /// If the named environment variable is present at compile time, this will
    /// expand into an expression of type `Option<&'static str>` whose value is
    /// `Some` of the value of the environment variable. If the environment
    /// variable is not present, then this will expand to `None`.
    ///
    /// A compile time error is never emitted when using this macro regardless
    /// of whether the environment variable is present or not.
    ///
    /// # Example
    ///
    /// ```rust
    /// let key: Option<&'static str> = option_env!("SECRET_KEY");
    /// println!("the secret key might be: {:?}", key);
    /// ```
    #[macro_export]
    macro_rules! option_env { ($name:expr) => ({ /* compiler built-in */ }) }

    /// Concatenate identifiers into one identifier.
    ///
    /// This macro takes any number of comma-separated identifiers, and
    /// concatenates them all into one, yielding an expression which is a new
    /// identifier. Note that hygiene makes it such that this macro cannot
    /// capture local variables, and macros are only allowed in item,
    /// statement or expression position, meaning this macro may be difficult to
    /// use in some situations.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(concat_idents)]
    ///
    /// # fn main() {
    /// fn foobar() -> u32 { 23 }
    ///
    /// let f = concat_idents!(foo, bar);
    /// println!("{}", f());
    /// # }
    /// ```
    #[macro_export]
    macro_rules! concat_idents {
        ($($e:ident),*) => ({ /* compiler built-in */ })
    }

    /// Concatenates literals into a static string slice.
    ///
    /// This macro takes any number of comma-separated literals, yielding an
    /// expression of type `&'static str` which represents all of the literals
    /// concatenated left-to-right.
    ///
    /// Integer and floating point literals are stringified in order to be
    /// concatenated.
    ///
    /// # Example
    ///
    /// ```
    /// let s = concat!("test", 10, 'b', true);
    /// assert_eq!(s, "test10btrue");
    /// ```
    #[macro_export]
    macro_rules! concat { ($($e:expr),*) => ({ /* compiler built-in */ }) }

    /// A macro which expands to the line number on which it was invoked.
    ///
    /// The expanded expression has type `usize`, and the returned line is not
    /// the invocation of the `line!()` macro itself, but rather the first macro
    /// invocation leading up to the invocation of the `line!()` macro.
    ///
    /// # Example
    ///
    /// ```
    /// let current_line = line!();
    /// println!("defined on line: {}", current_line);
    /// ```
    #[macro_export]
    macro_rules! line { () => ({ /* compiler built-in */ }) }

    /// A macro which expands to the column number on which it was invoked.
    ///
    /// The expanded expression has type `usize`, and the returned column is not
    /// the invocation of the `column!()` macro itself, but rather the first macro
    /// invocation leading up to the invocation of the `column!()` macro.
    ///
    /// # Example
    ///
    /// ```
    /// let current_col = column!();
    /// println!("defined on column: {}", current_col);
    /// ```
    #[macro_export]
    macro_rules! column { () => ({ /* compiler built-in */ }) }

    /// A macro which expands to the file name from which it was invoked.
    ///
    /// The expanded expression has type `&'static str`, and the returned file
    /// is not the invocation of the `file!()` macro itself, but rather the
    /// first macro invocation leading up to the invocation of the `file!()`
    /// macro.
    ///
    /// # Example
    ///
    /// ```
    /// let this_file = file!();
    /// println!("defined in file: {}", this_file);
    /// ```
    #[macro_export]
    macro_rules! file { () => ({ /* compiler built-in */ }) }

    /// A macro which stringifies its argument.
    ///
    /// This macro will yield an expression of type `&'static str` which is the
    /// stringification of all the tokens passed to the macro. No restrictions
    /// are placed on the syntax of the macro invocation itself.
    ///
    /// # Example
    ///
    /// ```
    /// let one_plus_one = stringify!(1 + 1);
    /// assert_eq!(one_plus_one, "1 + 1");
    /// ```
    #[macro_export]
    macro_rules! stringify { ($t:tt) => ({ /* compiler built-in */ }) }

    /// Includes a utf8-encoded file as a string.
    ///
    /// This macro will yield an expression of type `&'static str` which is the
    /// contents of the filename specified. The file is located relative to the
    /// current file (similarly to how modules are found),
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let secret_key = include_str!("secret-key.ascii");
    /// ```
    #[macro_export]
    macro_rules! include_str { ($file:expr) => ({ /* compiler built-in */ }) }

    /// Includes a file as a byte slice.
    ///
    /// This macro will yield an expression of type `&'static [u8]` which is
    /// the contents of the filename specified. The file is located relative to
    /// the current file (similarly to how modules are found),
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let secret_key = include_bytes!("secret-key.bin");
    /// ```
    #[macro_export]
    macro_rules! include_bytes { ($file:expr) => ({ /* compiler built-in */ }) }

    /// Expands to a string that represents the current module path.
    ///
    /// The current module path can be thought of as the hierarchy of modules
    /// leading back up to the crate root. The first component of the path
    /// returned is the name of the crate currently being compiled.
    ///
    /// # Example
    ///
    /// ```rust
    /// mod test {
    ///     pub fn foo() {
    ///         assert!(module_path!().ends_with("test"));
    ///     }
    /// }
    ///
    /// test::foo();
    /// ```
    #[macro_export]
    macro_rules! module_path { () => ({ /* compiler built-in */ }) }

    /// Boolean evaluation of configuration flags.
    ///
    /// In addition to the `#[cfg]` attribute, this macro is provided to allow
    /// boolean expression evaluation of configuration flags. This frequently
    /// leads to less duplicated code.
    ///
    /// The syntax given to this macro is the same syntax as the `cfg`
    /// attribute.
    ///
    /// # Example
    ///
    /// ```rust
    /// let my_directory = if cfg!(windows) {
    ///     "windows-specific-directory"
    /// } else {
    ///     "unix-directory"
    /// };
    /// ```
    #[macro_export]
    macro_rules! cfg { ($cfg:tt) => ({ /* compiler built-in */ }) }
}
