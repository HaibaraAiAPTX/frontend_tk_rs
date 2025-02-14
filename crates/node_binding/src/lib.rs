#![deny(clippy::all)]

mod bootstrap;
mod gen;
mod utils;

#[macro_use]
extern crate napi_derive;

pub use gen::gen::frontend_tk_gen;
