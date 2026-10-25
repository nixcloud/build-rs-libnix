# build-rs-libnix

`build-rs-libnix` turns the output of a crate's `build.rs` into files that nix builds can read. It is a helper of
[cargo (libnix)](https://github.com/nixcloud/cargo), the cargo fork that builds every crate in its own nix derivation.

A `build.rs` talks to cargo by printing `cargo:` lines (or `cargo::` lines, the syntax since Rust 1.77):

    cargo:rustc-cfg=freebsd11
    cargo:rustc-check-cfg=cfg(espidf_time32)
    cargo:rustc-link-lib=static=sqlite3
    cargo:rustc-link-search=native=/nix/store/…-sqlite-3.48.0/lib
    cargo:rustc-env=VAR=VALUE
    cargo:include=/build/libsqlite3-sys-0.31.0/sqlite3
    cargo:warning=This is a custom build warning from build.rs!

Both prefixes name the same directives. The one difference is the metadata of a `links` package: `cargo:KEY=VALUE`
in the old syntax, `cargo::metadata=KEY=VALUE` in the new one; both become the same `DEP_<links>_<KEY>`.

Normally cargo reads these lines itself
(`parse_metadata` in [`custom_build.rs`](https://github.com/rust-lang/cargo/blob/master/src/cargo/core/compiler/custom_build.rs))
and keeps the result in memory for the next `rustc` calls. With cargo (libnix), the build script runs inside a nix
derivation and the crates that use its result are built in other derivations, so the result has to be stored in the
derivation's output. `build-rs-libnix` does that:

    build-rs-libnix --script-output <file with the build.rs output> --out-dir $out/nix

It writes four files:

| file | contents | used by |
|---|---|---|
| `rustc-arguments` | `--cfg`, `--check-cfg`, `-l`, `-L`, `-C link-arg` | the crate the build script belongs to |
| `rustc-propagated-arguments` | `-L` from `rustc-link-search` | every crate that depends on it, so the final link finds native libraries |
| `environment-variables` | `rustc-env` values, and `DEP_<links>_*` for crates with a `links` key | the crate, and the build scripts of the crates that depend on it |
| `rustc-link-arg-benches` | `-C link-arg` for benchmarks | benchmarks |

`warning` lines are printed, so they show up in `nix build -L` and in cargo's output.

## Why a separate program?

cargo (libnix) writes one derivation per cargo unit. Each build-script run calls `build-rs-libnix`, so its store path
is an input of those derivations: it is part of their hashes, and therefore of the hashes of every crate built after
them.

**If the parser were part of cargo**, the cargo binary would be that input. Any change to cargo, even a comment,
would change the hash of every build-script run and of everything that depends on one. In practice that is almost
the whole dependency graph: change one line in cargo, run `cargo build`, and every crate builds again. When you work
on cargo (libnix) itself, you'd have to wait for a full rebuild each time. A binary cache wouldn't help either,
because every cargo version would produce different hashes.

**As a separate package** with a fixed version (cargo (libnix) pins a commit and hash in `build_rs_libnix.nix`),
the units' hashes change only when this parser is updated on purpose. You can change cargo as often as you like, and
the crates already in the nix store, or in a binary cache, are reused.

It's also quicker to work on: this crate builds in seconds and its tests run on captured `build.rs` outputs (in
`build-rs-libnix/test/`). Building cargo (libnix) takes about 440 units.

**Integrating it into cargo later?** Once the format settles, the code could move into the cargo repository, for
example as a workspace crate that cargo calls. The rebuild problem stays, though: it has to be built as its own small
derivation with its own version, not as part of the cargo binary, or every cargo change rebuilds everything again.

## Supported directives

| directive | result |
|---|---|
| `rustc-cfg` | `--cfg` |
| `rustc-check-cfg` | `--check-cfg` |
| `rustc-link-lib` | `-l` |
| `rustc-link-search` | `-L` for the crate, propagated to its dependents |
| `rustc-link-arg` | `-C link-arg`, for the crate's own targets |
| `rustc-link-arg-benches` | `-C link-arg` for benchmarks |
| `rustc-env` | an environment variable for the crate |
| `include`, `root`, `conf`, `version_number`, `static` (also as `cargo::metadata=KEY=VALUE`) | `DEP_<links>_<KEY>` for dependents' build scripts (needs `CARGO_MANIFEST_LINKS`) |
| `warning` | printed |
| `error` | printed; the build fails once the whole output is read, like with cargo |

**Ignored on purpose** (nix rebuilds a unit when its inputs change, so cargo's change tracking isn't needed):
`rerun-if-changed`, `rerun-if-env-changed`, `rerun-if-changed-bin`, `rerun-if-changed-glob`, `rerun-if-changed-dir`,
`rerun-if-changed-recursive`, `rerun-if-changed-env`, `lib_dir`.

**Not implemented yet**, the build fails with "not implemented yet": `rustc-flags`, `cargo:metadata` (old syntax),
`rustc-cdylib-link-arg` / `rustc-link-arg-cdylib`, `rustc-link-arg-bin`, `rustc-link-arg-bins`,
`rustc-link-arg-tests`, `rustc-link-arg-examples`. If you hit one, please open an issue with the crate's name.

Any other `cargo:KEY=VALUE` line (or `cargo::metadata=KEY=VALUE` with another key) is printed as a warning and
ignored.

## Usage

### From nix

```nix
# flake
inputs.build-rs-libnix.url = "github:nixcloud/build-rs-libnix";
# … inputs.build-rs-libnix.packages.${system}.default

# without flakes, from a checkout
build_rs_libnix = pkgs.callPackage ./default.nix { };
```

`nix build github:nixcloud/build-rs-libnix` builds it, and the binary is `result/bin/build-rs-libnix`.

### By hand

    cargo run -- --script-output build-rs-libnix/test/output1 --out-dir nix/

prints

    warning: This is a custom build warning from build.rs!
    build.rs related nix files written to 'nix/'

## Hacking

### Getting real build.rs outputs

The upstream cargo keeps the output of every build script it runs:

    find target -path '*/build/*/output'
    target/debug/build/openssl-sys-bf6c2c38618f44c9/output
    target/debug/build/parking_lot_core-7bae69a33df63e20/output
    …

    grep '^cargo:' target/debug/build/openssl-sys-bf6c2c38618f44c9/output

Copy an interesting one to `build-rs-libnix/test/` and add a test for it.

### Tests

    cargo test

The tests are in `build-rs-libnix/src/tests.rs`.

### Releasing a new version

1. bump `version` in `Cargo.toml` and `default.nix`
2. update the pinned `rev`, `sha256` and `cargoHash` in cargo (libnix)'s
   `src/compiler/nix_build/templates/build_rs_libnix.nix.handlebars`

Every unit with a build script, and everything that depends on one, is rebuilt once after that. That's the reason
to update it on purpose, not on every change.

## License

Like cargo, this project is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
