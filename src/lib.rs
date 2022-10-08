use std::env;

mod cache;
pub mod cli;
mod control;
mod dependency;
mod filters;
mod git;
mod importer;
mod installer;
mod loadable_config;
mod preset;
mod repository;
mod spec;
mod spec_lock;

#[cfg(test)]
mod test_utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VENDOR_YML: &str = ".vendor.yml";
const VENDOR_LOCK_YML: &str = ".vendor-lock.yml";
