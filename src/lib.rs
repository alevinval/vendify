use std::env;

mod cache;
pub mod cli;
mod control;
mod deps;
mod filters;
mod installer;
mod lock;
mod preset;
mod repository;
mod spec;
mod spec_lock;
mod yaml;

#[cfg(test)]
mod test_utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
