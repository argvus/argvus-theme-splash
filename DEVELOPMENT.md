# Development

The repository follows the ARGVUS `skeleton-rs-pkg` layout: the Cargo
workspace lives at the root, Rust crates are under `crates/`, Arch packaging
is split into `packaging/arch/ci` and `packaging/arch/local`, and local package
creation is handled by `tools/sh/pkgbuild_local.sh`.

Run `make check` for format, Clippy, and tests. Run `make validate` for shell
and PKGBUILD metadata checks.
