//! This crate provides a protocol for Volta's error handling, including a subtrait
//! of the [`failure`](https://github.com/rust-lang-nursery/failure) crate's
//! [`Fail`](https://docs.rs/failure/0.1.1/failure/trait.Fail.html) trait to manage
//! the interface between errors and process exit codes.
//!
//! # The `VoltaFail` trait
//!
//! The main interface for Volta errors is `VoltaFail`, which extends the
//! [`Fail`](https://docs.rs/failure/0.1.1/failure/trait.Fail.html) trait from the
//! [`failure`](https://github.com/rust-lang-nursery/failure) library with an additional
//! method. The `VoltaFail::exit_code()` method allows each error type to indicate what
//! the process exit code should be if the error is the reason for exiting Volta.
//!
//! # The `VoltaError` type and `Fallible` functions
//!
//! The main error type provided by this crate is `VoltaError`. This acts more
//! or less as the "root" error type for Volta; all Volta error types can be
//! coerced into this type.
//!
//! If you don't have any need for more specific static information about the errors
//! that can be produced by a function, you should define its signature to return
//! `Result<T, VoltaError>` (where `T` is whatever type you want for successful
//! results of the function).
//!
//! This is so common that you can use `Fallible<T>` as a shorthand.
//!
//! ## Example
//!
//! As a running example, we'll build a little parser for hex-encoded RGB triples.
//! The type could be defined as a struct of three bytes:
//!
//! ```
//! #[derive(Debug)]
//! struct Rgb { r: u8, g: u8, b: u8 }
//! ```
//!
//! A function that decodes a single two-digit component could then use `Fallible`
//! for its signature:
//!
//! ```
//! use volta_fail::Fallible;
//! #
//! # #[derive(Debug)]
//! # struct Rgb { r: u8, g: u8, b: u8 }
//!
//! // same as: fn parse_component(src: &str, i: usize) -> Result<u8, VoltaError>
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     // ...
//! #    Ok(17)
//! }
//! ```
//!
//! # Creating custom error types
//!
//! To create an error type in Volta, add a `#[derive]` attribute to derive the `Fail`
//! trait before the type declaration, and add a `#[fail(display = "...")]` attribute to
//! construct the error message string.
//!
//! Continuing with the running example, we could create an error type for running past
//! the end of the input string:
//!
//! ## Example
//!
//! ```
//! // required for `#[derive(Fail)]` and `#[fail(...)]` attributes
//! use failure::Fail;
//!
//! use volta_fail::{ExitCode, VoltaFail};
//! use volta_fail_derive::*;
//!
//! #[derive(Debug, Fail, VoltaFail)]
//! #[fail(display = "unexpected end of string")]
//! #[volta_fail(code = "InvalidArguments")]
//! struct UnexpectedEndOfString;
//! ```
//!
//! # Throwing errors
//!
//! The `throw!` macro is a convenient syntax for an early exit with an error. It
//! can be used inside any function with a `Result` return type (often a `Fallible<T>`).
//! The argument expression can evaluate to any type that implements a coercion to
//! the declared error type.
//!
//! ## Example
//!
//! ```
//! # use failure::Fail;
//! # use volta_fail::{ExitCode, Fallible, VoltaFail};
//! use volta_fail::throw;
//!
//! # use volta_fail_derive::*;
//! # #[derive(Debug, Fail, VoltaFail)]
//! # #[fail(display = "unexpected end of string")]
//! # #[volta_fail(code = "InvalidArguments")]
//! # struct UnexpectedEndOfString;
//! #
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     if i + 2 > src.len() {
//!         // UnexpectedEndOfString implements VoltaFail, so it coerces to VoltaError
//!         throw!(UnexpectedEndOfString);
//!     }
//!
//!     // ...
//! #   Ok(0)
//! }
//! ```
//!
//! # Using third-party error types
//!
//! When using a third-party library that has error types of its own, those error types
//! need to be converted to Volta errors. Since third party libraries have not been
//! designed with Volta's end-user error messages in mind, third-party error types are
//! not automatically converted into Volta errors.
//!
//! Instead, this crate provides a couple of extension traits that you can import to
//! add an `with_context()` method to errors (`FailExt`) or `Result`s (`ResultExt`). This
//! method will convert any third-party error to a Volta error.
//!
//! ## Cause chains
//!
//! Since errors get propagated up from lower abstraction layers to higher ones, the
//! higher layers of abstraction often need to add contextual information to the error
//! messages, producing higher quality messages.
//!
//! For example, the `ParseIntError` produced by `u8::from_str_radix` does not tell
//! the end user that we were parsing an integer in the context of parsing an RGB
//! value.
//!
//! To add contextual information to a lower layer's error, we use the `with_context`
//! method and pass it a closure that takes a reference to the lower layer's error
//! and uses it to construct a new higher-level error.
//!
//! A powerful feature of `with_context` is that it saves the lower-level
//! error message as part of a _cause_ chain, which Volta's top-level can then use
//! to produce in-depth diagnostics in a log file or for `--verbose` error reporting.
//! Most error handling logic should not need to work with cause chains, so this is
//! all handled automatically.
//!
//! ## Example
//!
//! ```
//! # use failure::Fail;
//! # use volta_fail::{throw, ExitCode, Fallible, VoltaFail};
//! # use volta_fail_derive::*;
//! # #[derive(Debug, Fail, VoltaFail)]
//! # #[fail(display = "unexpected end of string")]
//! # #[volta_fail(code = "InvalidArguments")]
//! # struct UnexpectedEndOfString;
//!
//! use std::fmt::Display;
//! // add `with_context()` extension method to Results
//! use volta_fail::ResultExt;
//!
//! #[derive(Debug, Fail, VoltaFail)]
//! #[fail(display = "invalid RGB string: {}", details)]
//! #[volta_fail(code = "InvalidArguments")]
//! struct InvalidRgbString { details: String }
//!
//! impl InvalidRgbString {
//!     fn new<D: Display>(details: &D) -> InvalidRgbString {
//!         InvalidRgbString { details: format!("{}", details) }
//!     }
//! }
//!
//! fn parse_component(src: &str, i: usize) -> Fallible<u8> {
//!     if i + 2 > src.len() {
//!         // UnexpectedEndOfString implements VoltaFail, so it coerces to VoltaError
//!         throw!(UnexpectedEndOfString);
//!     }
//!
//!     // convert the std::num::ParseIntError into a VoltaError
//!     u8::from_str_radix(&src[i..i + 2], 16).with_context(InvalidRgbString::new)
//! }
//! ```
//!
//! Notice that you can use `with_context` to wrap any kind of error, including
//! errors that may already be user-friendly. So you can always use this to add
//! even more clarity to any errors. For instance, in our running example of an
//! RGB parser, a higher layer may want to add context about _which_ RGB string
//! was being parsed and where it came from (say, the filename and line number).

use std::convert::{From, Into};
use std::fmt;
use std::process::exit;

use failure::{Backtrace, Fail};
use serde::Serialize;

/// A temporary polyfill for `throw!` until the new `failure` library includes it.
#[macro_export]
macro_rules! throw {
    ($e:expr) => {
        return Err(::std::convert::Into::into($e));
    };
}

/// Exit codes supported by the VoltaFail trait.
#[derive(Copy, Clone, Debug, Serialize)]
pub enum ExitCode {
    /// No error occurred.
    Success = 0,

    /// An unknown error occurred.
    UnknownError = 1,

    /// An invalid combination of command-line arguments was supplied.
    InvalidArguments = 3,

    /// No match could be found for the requested version string.
    NoVersionMatch = 4,

    /// A network error occurred.
    NetworkError = 5,

    /// A required environment variable was unset or invalid.
    EnvironmentError = 6,

    /// A file could not be read or written.
    FileSystemError = 7,

    /// Package configuration is missing or incorrect.
    ConfigurationError = 8,

    /// The command or feature is not yet implemented.
    NotYetImplemented = 9,

    /// The requested executable could not be run.
    ExecutionFailure = 126,

    /// The requested executable is not available.
    ExecutableNotFound = 127,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        exit(self as i32);
    }
}

/// The failure trait for all Volta errors.
pub trait VoltaFail: Fail {
    /// Returns the process exit code that should be returned if the process exits with this error.
    fn exit_code(&self) -> ExitCode;
}

/// The `VoltaError` type, which can contain any Volta failure.
#[derive(Debug)]
pub struct VoltaError {
    /// The underlying error.
    error: failure::Error,

    /// The result of `error.exit_code()`.
    exit_code: ExitCode,
}

impl Fail for VoltaError {
    fn cause(&self) -> Option<&dyn Fail> {
        Some(self.error.as_fail())
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }
}

impl fmt::Display for VoltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl VoltaError {
    /// Returns a reference to the underlying failure of this error.
    pub fn as_fail(&self) -> &dyn Fail {
        self.error.as_fail()
    }

    /// Gets a reference to the `Backtrace` for this error.
    pub fn backtrace(&self) -> &Backtrace {
        self.error.backtrace()
    }

    /// Attempts to downcast this error to a particular `VoltaFail` type by reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_ref<T: VoltaFail>(&self) -> Option<&T> {
        self.error.downcast_ref()
    }

    /// Attempts to downcast this error to a particular `VoltaFail` type by mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: VoltaFail>(&mut self) -> Option<&mut T> {
        self.error.downcast_mut()
    }

    /// Returns the process exit code that should be returned if the process exits with this error.
    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }
}

impl<T: VoltaFail> From<T> for VoltaError {
    fn from(failure: T) -> Self {
        let exit_code = failure.exit_code();
        VoltaError {
            error: failure.into(),
            exit_code,
        }
    }
}

/// An extension trait allowing any failure, including failures from external libraries,
/// to be converted to a Volta error. This marks the error as an unknown error, i.e.
/// a non-user-friendly error.
pub trait FailExt {
    fn with_context<F, D>(self, f: F) -> VoltaError
    where
        F: FnOnce(&Self) -> D,
        D: VoltaFail;
}

/// An extension trait for `Result` values, allowing conversion of third-party errors
/// or other lower-layer errors into Volta errors.
pub trait ResultExt<T, E> {
    /// Wrap any error-producing result in a higher-layer error-producing result, pushing
    /// the lower-layer error onto the cause chain.
    fn with_context<F, D>(self, f: F) -> Result<T, VoltaError>
    where
        F: FnOnce(&E) -> D,
        D: VoltaFail;
}

impl<E: Into<failure::Error>> FailExt for E {
    fn with_context<F, D>(self, f: F) -> VoltaError
    where
        F: FnOnce(&Self) -> D,
        D: VoltaFail,
    {
        let display = f(&self);
        let error: failure::Error = self.into();
        let context = error.context(display);
        context.into()
    }
}

impl<T, E: Into<failure::Error>> ResultExt<T, E> for Result<T, E> {
    fn with_context<F, D>(self, f: F) -> Result<T, VoltaError>
    where
        F: FnOnce(&E) -> D,
        D: VoltaFail,
    {
        self.map_err(|err| err.with_context(f))
    }
}

impl<D: VoltaFail> VoltaFail for failure::Context<D> {
    fn exit_code(&self) -> ExitCode {
        self.get_context().exit_code()
    }
}

/// A convenient shorthand for `Result` types that produce `VoltaError`s.
pub type Fallible<T> = Result<T, VoltaError>;
