# The Vendor CLI

This tool allows users to install, manage and update dependencies. See the
[example](example/) to see how it works. Full disclaimer, this has been
a pet personal project used to learn Rust.

## Usage
1. `vendor init` initialises a `.vendor.yml` file in the working directory
3. `vendor add` adds a dependency in the `.vendor.yml` file
5. `vendor install` downloads and vendors the vendor the specified dependencies
   1. The first time this command is executed, it will generate a `.vendor-lock.yml`
      which keeps track of the locked reference that has been vendored (eg. a specific commit)
   2. If the lock file is already present, it will vendor the depencies locked to
      whatever reference the dependency is locked at
6. `vendor update` ignores the `vendor-lock.yml` and fetches newest dependencies
   according to the refname that is specified in the `.vendor.yml` file
