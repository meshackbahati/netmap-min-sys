# Changelog for netmap-min-sys

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-08-17

### Added
- Compile a small C shim (`c/netmap_shim.c`) that re-exports the `nm_open`,
  `nm_close`, `nm_mmap` and `nm_inject` helpers from `netmap_user.h` as
  `ffi_nm_*` symbols, so they can be linked from Rust. These helpers are
  `static` in the C header and are otherwise not exported by any library.
- Hand-written `nm_desc`, `nm_pkthdr` and `nm_stat` structs matching the C
  layout on x86-64 (bindgen emits an incorrect opaque layout for these due to
  the recursive `nm_desc` <-> `nm_pkthdr` cycle). Layout verified with
  `sizeof`/`offsetof` assertions in the test suite.
- Rust implementations of the `NETMAP_TXRING`, `NETMAP_RXRING`, `NETMAP_IF`
  and `NETMAP_BUF` offset macros from `netmap_user.h`.
- Manual ioctl number constants (`NIOCTXSYNC`, `NIOCRXSYNC`, `NIOCREGIF`,
  `NIOCCTRL`) that bindgen cannot expand from the `_IO`/`_IOWR` macros.
- Legacy registration-mode aliases `NR_REG_SW_ONLY`, `NR_REG_NIC_ONLY` and
  `NR_REG_NIC_AND_SW` mapping onto the current `NR_REG_*` API.
- Unit tests covering struct sizes/alignment, ioctl constants and
  registration-mode values.

### Changed
- Bumped to `0.3.0` and switched the edition to `2021` for broad toolchain
  compatibility.
- Fixed the `sys`/bindgen surface so bindings are generated under
  `NETMAP_WITH_LIBS`, exposing the `nmreq_*`, `nm_desc`, `netmap_if`,
  `netmap_ring` and `netmap_slot` family to downstream crates. Previously the
  allowlist was too narrow and almost nothing was emitted.
- `disable-netmap-kernel` now produces a genuinely empty crate (no stale
  `mod ffi {}` or duplicated types) so it builds cleanly on systems without
  the Netmap headers.
- The Netmap library directory (`$NETMAP_LOCATION/lib`) is now passed to the
  linker via `cargo:rustc-link-search=native`, and `NETMAP_INCLUDE_PATH` /
  `NETMAP_LIB_PATH` are exposed as `cargo:rustc-env` for downstream crates.

### Fixed
- Removed the duplicated `NM_ERRBUF_SIZE`, `NR_REG_*` and opaque
  `nm_desc`/`nm_pkthdr`/`nm_stat` definitions that made the crate fail to
  compile once the bindings were actually generated.

## [0.2.2] - 2025-08-03

### Added
- Created this `CHANGELOG.md` file.
- Created `README.md` explaining prerequisites, build configuration using
  `NETMAP_LOCATION` and `DISABLE_NETMAP_KERNEL` environment variables.

### Changed
- Modified `build.rs` to explicitly pass the include path
  (`$NETMAP_LOCATION/include` or default `/usr/local/include`) to `bindgen`
  using `.clang_arg()`. This makes the discovery of Netmap headers more
  robust, especially for non-standard installation locations of the Netmap C
  library.
