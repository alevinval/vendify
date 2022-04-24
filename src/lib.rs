use std::env;

pub mod cli;
mod core;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VENDOR_YML: &str = ".vendor.yml";
const VENDOR_LOCK_YML: &str = ".vendor-lock.yml";
