# netmap-min-sys

This crate provides low-level FFI (Foreign Function Interface) bindings to the
[Netmap](https://netmap.org/) C library. Netmap is a framework for very fast
packet I/O from userspace.

`netmap-min-sys` is a "sys" crate: it handles C header parsing, library
linking and exposes raw, unsafe bindings. Higher-level, safe abstractions are
provided by the `netmap-rs` crate, which depends on this one.

## What is exposed

- **bindgen-generated bindings** for the `netmap_*`, `nmreq_*` and `NR_*` /
  `NS_*` / `NETMAP_*` families from `<net/netmap.h>` and `<net/netmap_user.h>`.
- **`nm_open` / `nm_close` / `nm_mmap` / `nm_inject`** — the standard Netmap
  descriptor helpers. These are `static` functions inside `netmap_user.h`, so
  no shared library exports them; this crate compiles them in through a small
  C shim (`c/netmap_shim.c`), so you only need the Netmap *headers*, not a
  built `libnetmap`.
- **`NETMAP_TXRING` / `NETMAP_RXRING` / `NETMAP_IF` / `NETMAP_BUF`** — safe-ish
  Rust functions implementing the offset macros from `netmap_user.h`.
- **ioctl numbers** `NIOCRXSYNC`, `NIOCTXSYNC`, `NIOCREGIF`, `NIOCCTRL`, which
  bindgen cannot expand from the `_IO`/`_IOWR` macros.

## Prerequisites

To compile and use this crate (and, by extension, `netmap-rs` with its `sys`
feature), you must have the Netmap C headers installed on your system.

1. **Install the Netmap headers:** clone the
   [netmap repository](https://github.com/netmap/netmap), `cd sys`, then copy
   the `net/` headers into an include directory (see
   `netmap-rs/scripts/install_netmap.sh` for a complete example).
2. **Install Clang:** the `bindgen` tool used by this crate's build script to
   generate Rust bindings from C headers requires `clang`.
   (e.g. `sudo apt install clang libclang-dev` on Debian/Ubuntu).

## Build Configuration

The build script (`build.rs`) locates your Netmap installation through the
`NETMAP_LOCATION` environment variable (defaulting to `/usr/local`).

### Standard Installation

If the Netmap headers are installed in a standard location (e.g.
`/usr/local/include/net/netmap.h`), the build script finds them automatically.

### Custom Netmap Installation Path (`NETMAP_LOCATION`)

Set `NETMAP_LOCATION` to the root directory that contains the `include` and
`lib` subdirectories.

```bash
NETMAP_LOCATION=/opt/netmap cargo build
```

The build script then:
- instructs `bindgen` to look for headers in `$NETMAP_LOCATION/include`;
- passes `$NETMAP_LOCATION/lib` to the linker;
- exposes `NETMAP_INCLUDE_PATH` and `NETMAP_LIB_PATH` as `cargo:rustc-env`
  values so downstream crates can reuse them.

### Disabling Netmap (`disable-netmap-kernel`)

On platforms where Netmap is unavailable, build the crate as a no-op with the
`disable-netmap-kernel` feature or the `DISABLE_NETMAP_KERNEL` environment
variable. The build script skips `bindgen` and the C shim, and the crate
exports an empty API.

```bash
cargo build --features disable-netmap-kernel
# or
DISABLE_NETMAP_KERNEL=1 cargo build
```

## Usage

This crate is not typically used directly. The `netmap-rs` crate provides safe
Rust abstractions over the raw bindings exposed here. If you are using
`netmap-rs`, enable its `sys` feature, which pulls in and configures this
`-sys` crate.

## License
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)).
- MIT license ([LICENSE-MIT](LICENSE-MIT)).
