/*!
A full-featured example REST service using [`axum`], [`sqlx`], [`tokio`], and
[`tracing`].
*/

// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
// https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(unreachable_pub, unused_extern_crates)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(
    debug_assertions,
    allow(clippy::missing_panics_doc, clippy::missing_errors_doc)
)]

pub mod config;
pub mod db;
mod links;
pub mod server;
pub mod telemetry;
#[cfg(test)]
mod test_helpers;
