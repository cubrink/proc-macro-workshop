//! Rustc-style diagnostic messages for proc-macros on stable Rust.
//!
//! On nightly, `rustc` exposes a [`Diagnostic`] API that allows proc-macros to
//! emit richly formatted error messages with `note` and `help` annotations.
//! This is not available on stable. This module provides a lightweight
//! alternative that formats the same visual output into a [`syn::Error`], which
//! can be emitted via [`syn::Error::to_compile_error`] in the usual way.
//!
//! # Vendoring
//!
//! This file is designed to be vendored directly into any proc-macro crate.
//! No additional dependencies are required beyond `syn` and `proc_macro2`,
//! which you almost certainly already have. All external crate paths are fully
//! qualified (e.g. `::std::string::String` rather than `String`) so that this
//! file can be dropped into any module without conflicting with local imports
//! or type aliases.
//!
//! # Usage
//!
//! Build a diagnostic starting with either [`Diagnostic::message`] or
//! [`Diagnostic::span`], whichever you have first. Both are required before the
//! diagnostic can be converted into a [`syn::Error`] - this is enforced at
//! compile time via the [`SpanlessDiagnostic`] and [`MessagelessDiagnostic`]
//! typestates.
//!
//! ```rust,no_run
//! let err: syn::Error = Diagnostic::message("attribute `each` requires field type to be `Vec<_>`")
//!     .span(field.span())
//!     .note("required by the `each` attribute")
//!     .help("change the field type to `Vec<T>`, or remove the `each` attribute")
//!     .into();
//! ```
//!
//! Which emits:
//!
//! ```text
//! error: attribute `each` requires field type to be `Vec<_>`
//!   |
//!   = note: required by the `each` attribute
//!   = help: change the field type to `Vec<T>`, or remove the `each` attribute
//! ```
//!
//! # Annotations
//!
//! Both `note` and `help` annotations can be chained in any order and render
//! beneath the primary message in insertion order.
//!
//! - **note** - additional context about why the error occurred
//! - **help** - a suggestion for how to fix it
//!
//! All functions consuming user-facing error messaging (`message`, `note`, `help`)
//! take &str, rather than `impl Into<String>` purposely. This increases friction
//! when attempting to pass types with non-prose string representations from being
//! passed in.
//!
//! # Error propagation (optional)
//!
//! This file also includes opinionated approaches for aggregating multiple
//! errors across a proc-macro invocation, so that all errors can be reported
//! to the user at once rather than bailing on the first failure. Any modules
//! present are optional - delete whichever modules you don't want.
//!
//! - [`aggregate_errors`] - collect errors locally with [`AggregateError`],
//!   which provides convenience functions to collect and emit multiple
//!   [`syn::Error`]'s as a single syn::Error.
//!

#[allow(unused_imports)]
pub use diagnostic::{Diagnostic, MessagelessDiagnostic, SpanlessDiagnostic};

// Uncomment to opt-in to local error aggregation.
#[allow(unused_imports)]
pub use aggregate_errors::AggregateError;

mod diagnostic {
    /// An intermediate [`Diagnostic`] that has a message but no span yet.
    ///
    /// Obtain one via [`Diagnostic::message`]. Call [`SpanlessDiagnostic::span`]
    /// to produce a complete [`Diagnostic`].
    #[derive(Debug, Clone)]
    #[must_use = "call .span() to complete the diagnostic"]
    pub struct SpanlessDiagnostic {
        message: ::std::string::String,
    }

    impl SpanlessDiagnostic {
        /// Attach a span to this diagnostic, producing a complete [`Diagnostic`].
        pub fn span(self, span: ::proc_macro2::Span) -> Diagnostic {
            Diagnostic {
                span,
                message: self.message,
                annotations: ::std::vec::Vec::new(),
            }
        }
    }

    /// An intermediate [`Diagnostic`] that has a span but no message yet.
    ///
    /// Obtain one via [`Diagnostic::span`]. Call [`MessagelessDiagnostic::message`]
    /// to produce a complete [`Diagnostic`].
    #[derive(Debug, Clone)]
    #[must_use = "call .message() to complete the diagnostic"]
    pub struct MessagelessDiagnostic {
        span: ::proc_macro2::Span,
    }

    impl MessagelessDiagnostic {
        /// Attach a message to this diagnostic, producing a complete [`Diagnostic`].
        pub fn message(self, message: &::std::primitive::str) -> Diagnostic {
            Diagnostic {
                span: self.span,
                message: message.to_owned(),
                annotations: ::std::vec::Vec::new(),
            }
        }
    }

    /// A compiler diagnostic for use in proc-macros.
    ///
    /// Renders in the style of `rustc` diagnostics, with optional `note` and `help`
    /// annotations below the primary message. Converts into a [`syn::Error`] via
    /// [`Into`], which can then be emitted with [`syn::Error::to_compile_error`].
    ///
    /// Start building with either [`Diagnostic::span`] or [`Diagnostic::message`]
    /// depending on which you have first. Both are required before the diagnostic
    /// can be converted into a [`syn::Error`] - this is enforced at compile time.
    ///
    /// # Example
    ///
    /// ```rust
    /// let err: syn::Error = Diagnostic::message("attribute `each` requires field type to be `Vec<_>`")
    ///     .span(field.span())
    ///     .help("change the field type to `Vec<T>`, or remove the `each` attribute")
    ///     .into();
    /// ```
    #[derive(Debug, Clone)]
    #[must_use = "diagnostic does nothing unless converted to syn::Error or emitted"]
    pub struct Diagnostic {
        span: ::proc_macro2::Span,
        message: ::std::string::String,
        annotations: ::std::vec::Vec<::std::string::String>,
    }

    impl Diagnostic {
        /// Begin building a diagnostic with a span, to be followed by a message.
        ///
        /// Returns a [`MessagelessDiagnostic`]. Call `.message()` on it to produce
        /// a complete [`Diagnostic`].
        pub fn span(span: ::proc_macro2::Span) -> MessagelessDiagnostic {
            MessagelessDiagnostic { span }
        }

        /// Begin building a diagnostic with a message, to be followed by a span.
        ///
        /// Returns a [`SpanlessDiagnostic`]. Call `.span()` on it to produce
        /// a complete [`Diagnostic`].
        pub fn message(message: &::std::primitive::str) -> SpanlessDiagnostic {
            SpanlessDiagnostic {
                message: message.to_owned(),
            }
        }

        /// Append a `note` annotation to this diagnostic.
        ///
        /// Notes provide additional context about why the error occurred.
        /// Annotations render in insertion order beneath the primary message.
        pub fn note(mut self, note: &::std::primitive::str) -> Self {
            self.annotations.push(::std::format!("note: {note}"));
            self
        }

        /// Append a `help` annotation to this diagnostic.
        ///
        /// Help annotations suggest what the user should do to fix the error.
        /// Annotations render in insertion order beneath the primary message.
        pub fn help(mut self, help: &::std::primitive::str) -> Self {
            self.annotations.push(::std::format!("help: {help}"));
            self
        }

        /// Convert this diagnostic into a [`proc_macro2::TokenStream`] containing a compile error.
        ///
        /// This is a convenience shorthand for converting to [`syn::Error`] and then
        /// calling [`syn::Error::to_compile_error`]:
        ///
        /// ```rust,no_run
        /// // These are equivalent:
        /// diagnostic.to_compile_error()
        /// syn::Error::from(diagnostic).to_compile_error()
        /// ```
        pub fn to_compile_error(self) -> ::proc_macro2::TokenStream {
            ::syn::Error::from(self).to_compile_error()
        }
    }

    impl ::std::fmt::Display for Diagnostic {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::write!(f, "{}", self.message)?;
            if !self.annotations.is_empty() {
                ::std::write!(f, "\n  |")?;
                for annotation in &self.annotations {
                    ::std::write!(f, "\n  = {annotation}")?;
                }
            }
            ::std::result::Result::Ok(())
        }
    }

    impl ::std::convert::From<Diagnostic> for ::syn::Error {
        fn from(diagnostic: Diagnostic) -> Self {
            ::syn::Error::new(diagnostic.span, diagnostic.to_string())
        }
    }

    impl<T> ::std::convert::Into<::syn::Result<T>> for Diagnostic {
        fn into(self) -> ::syn::Result<T> {
            ::syn::Result::Err(self.into())
        }
    }

    impl ::std::error::Error for Diagnostic {}
}

// =============================================================================
// Aggregate errors (optional)
//
// An explicit error aggregator you pass around to collect multiple errors,
// then combine at the end. Useful when you want local, structured control over
// where errors are collected.
//
// If you don't need this, delete this module and its `pub use` line above.
// =============================================================================
#[allow(dead_code)]
mod aggregate_errors {
    /// A collection of [`syn::Error`] values that can be merged into one.
    ///
    /// Useful when validating multiple inputs in a proc-macro and wanting to
    /// report all discovered errors together rather than returning after the
    /// first one. Push errors in as they are encountered, then combine at the
    /// end.
    ///
    /// This can be used in two ways:
    ///
    /// - **Passed around explicitly** - create one at your top-level entry
    ///   point and pass it by `&mut` reference through each validation call,
    ///   collecting all errors before returning.
    /// - **Used locally** - collect errors within a single function, combine
    ///   them, and return the result as a normal `syn::Result`. This composes
    ///   naturally with `?`-based error propagation elsewhere.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let mut errors = AggregateError::new();
    ///
    /// // Collect all passing fields
    /// let my_fields: Vec<MyFieldStruct> = self.fields.iter()
    ///     .flat_map(|syn_field| {
    ///         // Fallible conversion from syn's field to your intermediate representation
    ///         let result: syn::Result<MyFieldStruct> = MyFieldStruct::try_from(syn_field);  
    ///         // Aggregates errors for Err variants, returns Some<T> for Ok variants
    ///         errors.push_result(result)
    ///     })
    ///   .collect()
    ///
    /// // If any errors have been collected so far, exit the function early
    /// errors.combine()?;
    /// todo!("create struct with validated inputs")
    /// ```
    #[derive(Debug, Clone, Default)]
    pub struct AggregateError {
        errors: ::std::vec::Vec<::syn::Error>,
    }

    impl AggregateError {
        /// Create an empty error aggregator.
        pub fn new() -> Self {
            Self {
                errors: ::std::vec::Vec::new(),
            }
        }

        /// Append an error to the aggregator.
        ///
        /// Accepts any type that can be converted into a [`syn::Error`],
        /// including [`super::Diagnostic`].
        pub fn push(&mut self, error: impl ::std::convert::Into<::syn::Error>) {
            self.errors.push(error.into());
        }

        /// Handle a [`syn::Result`], collecting the error if present.
        ///
        /// Returns `Some(value)` if the result is `Ok`, or `None` if it is `Err`
        /// (after pushing the error into this aggregator). This is designed to
        /// work naturally with [`Iterator::flat_map`] or [`Iterator::filter_map`]
        /// to process a collection while accumulating all errors:
        ///
        /// ```rust,no_run
        /// let fields: Vec<_> = input.fields.iter()
        ///     .filter_map(|f| errors.push_result(validate(f)))
        ///     .collect();
        /// ```
        pub fn push_result<T>(&mut self, result: ::syn::Result<T>) -> ::std::option::Option<T> {
            match result {
                ::std::result::Result::Ok(v) => ::std::option::Option::Some(v),
                ::std::result::Result::Err(e) => {
                    self.push(e);
                    ::std::option::Option::None
                }
            }
        }

        /// Combine all collected errors into a single [`syn::Error`].
        ///
        /// Returns `Ok(())` if no errors were collected, or `Err` containing
        /// all errors merged via [`syn::Error::combine`] in insertion order.
        pub fn combine(self) -> ::syn::Result<()> {
            match self.errors.into_iter().reduce(|mut acc, err| {
                acc.combine(err);
                acc
            }) {
                ::std::option::Option::None => ::std::result::Result::Ok(()),
                ::std::option::Option::Some(e) => ::std::result::Result::Err(e),
            }
        }

        /// Returns `true` if no errors have been collected.
        pub fn is_empty(&self) -> ::std::primitive::bool {
            self.errors.is_empty()
        }

        /// Returns the number of aggregated errors
        pub fn len(&self) -> ::std::primitive::usize {
            self.errors.len()
        }
    }

    impl ::std::iter::Extend<::syn::Error> for AggregateError {
        fn extend<T: IntoIterator<Item = ::syn::Error>>(&mut self, iter: T) {
            self.errors.extend(iter);
        }
    }

    impl ::std::iter::Extend<super::diagnostic::Diagnostic> for AggregateError {
        fn extend<T: IntoIterator<Item = super::diagnostic::Diagnostic>>(&mut self, iter: T) {
            self.errors
                .extend(iter.into_iter().map(::std::convert::Into::into));
        }
    }
}
