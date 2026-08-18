//! Raw FFI bindings and helper shims for the [Netmap](https://netmap.org/)
//! kernel-bypass networking framework.
//!
//! The bulk of the API is generated with `bindgen` from the Netmap C headers
//! (`<net/netmap_user.h>` and `<net/netmap.h>`) and re-exported here. On top of
//! the raw bindings this crate also provides:
//!
//! * [`nm_open`] / [`nm_close`] — the standard Netmap descriptor helpers. These
//!   are `static` functions in the C headers (so no shared/static library
//!   exports them); this crate compiles them in via a small C shim.
//! * [`NETMAP_TXRING`] / [`NETMAP_RXRING`] / [`NETMAP_IF`] / [`NETMAP_BUF`] —
//!   Rust implementations of the function-like offset macros from
//!   `netmap_user.h`.
//! * The Netmap ioctl numbers ([`NIOCRXSYNC`], [`NIOCTXSYNC`], [`NIOCREGIF`],
//!   [`NIOCCTRL`]) which bindgen cannot evaluate from the `_IO`/`_IOWR` macros.
//!
//! # Build configuration
//!
//! The netmap headers are discovered through the `NETMAP_LOCATION` environment
//! variable (defaulting to `/usr/local`), looking for headers in
//! `$NETMAP_LOCATION/include`. When Netmap is not needed, enable the
//! `disable-netmap-kernel` feature (or set `DISABLE_NETMAP_KERNEL`) to build an
//! empty crate. If the headers cannot be found at all (for example when the
//! crate is built by crates.io or docs.rs, which never have Netmap installed),
//! the build degrades to the same empty state automatically instead of failing.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

#[cfg(not(netmap_disabled))]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/binding.rs"));
}

#[cfg(not(netmap_disabled))]
pub use bindings::*;

/// Export the helper types only when the netmap bindings are available.
#[cfg(not(netmap_disabled))]
pub mod exports {
    pub use super::bindings::NR_REG_ALL_NIC;
    pub use super::bindings::NR_REG_NIC_SW;
    pub use super::bindings::NR_REG_ONE_NIC;
    pub use super::bindings::NR_REG_PIPE_MASTER;
    pub use super::bindings::NR_REG_PIPE_SLAVE;
    pub use super::bindings::NR_REG_SW;
}

// ---------------------------------------------------------------------------
// Hand-written core structs.
//
// bindgen emits nm_desc (and its nm_pkthdr/nm_stat constituents) with an
// incorrect opaque layout because of the mutually-recursive nm_desc <->
// nm_pkthdr type cycle in netmap_user.h. Their layout is defined below to
// match the C headers exactly (verified with `offsetof`/`sizeof` on x86-64).
// ---------------------------------------------------------------------------

#[cfg(not(netmap_disabled))]
/// A Netmap descriptor (`struct nm_desc` in `netmap_user.h`).
///
/// This is the main handle returned by [`nm_open`] and used to reach the
/// `netmap_if` (via the [`nifp`](Self::nifp) field) and the underlying file
/// descriptor (via the [`fd`](Self::fd) field).
#[repr(C)]
#[derive(Debug)]
pub struct nm_desc {
    /// Points to itself when the descriptor is open; used for sanity checks.
    pub self_: *mut nm_desc,
    /// File descriptor of the `/dev/netmap` device.
    pub fd: i32,
    /// Base address of the mapped Netmap shared-memory region.
    pub mem: *mut core::ffi::c_void,
    /// Size of the mapped region.
    pub memsize: usize,
    /// Non-zero if `mem` is the result of `mmap`.
    pub done_mmap: i32,
    /// Immutable pointer to the `netmap_if` at the start of the region.
    pub nifp: *const netmap_if,
    pub first_tx_ring: u16,
    pub last_tx_ring: u16,
    pub cur_tx_ring: u16,
    pub first_rx_ring: u16,
    pub last_rx_ring: u16,
    pub cur_rx_ring: u16,
    /// The registration request used to open the port.
    pub req: nmreq,
    /// Header of the last packet read via `nm_nextpkt`.
    pub hdr: nm_pkthdr,
    /// A pointer to one of the rings; used to translate buffer indices.
    pub some_ring: *const netmap_ring,
    /// Start of the buffer area within the mapped region.
    pub buf_start: *const core::ffi::c_void,
    /// End of the buffer area within the mapped region.
    pub buf_end: *const core::ffi::c_void,
    pub snaplen: i32,
    pub promisc: i32,
    pub to_ms: i32,
    /// Buffer used by the `nm_*` helpers to report the last error.
    pub errbuf: *mut core::ffi::c_char,
    pub if_flags: u32,
    pub if_reqcap: u32,
    pub if_curcap: u32,
    pub st: nm_stat,
    /// Last error message written by the `nm_*` helpers.
    pub msg: [core::ffi::c_char; NM_ERRBUF_SIZE as usize],
}

#[cfg(not(netmap_disabled))]
/// Netmap packet header returned by the `nm_*` convenience helpers
/// (`struct nm_pkthdr` in `netmap_user.h`).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nm_pkthdr {
    pub ts: timeval,
    pub caplen: core::ffi::c_uint,
    pub len: core::ffi::c_uint,
    pub flags: u64,
    pub d: *mut nm_desc,
    pub slot: *mut netmap_slot,
    pub buf: *mut core::ffi::c_uchar,
}

#[cfg(not(netmap_disabled))]
/// Netmap statistics (`struct nm_stat` in `netmap_user.h`).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nm_stat {
    pub ps_recv: core::ffi::c_uint,
    pub ps_drop: core::ffi::c_uint,
    pub ps_ifdrop: core::ffi::c_uint,
}

// ---------------------------------------------------------------------------
// C shim linkage
// ---------------------------------------------------------------------------

#[cfg(not(netmap_disabled))]
extern "C" {
    /// Open a Netmap port (see `netmap_user.h` `nm_open(4)`).
    ///
    /// Returns a pointer to a `nm_desc` on success, or a null pointer on
    /// failure (with `errno` set).
    #[link_name = "ffi_nm_open"]
    pub fn nm_open(
        ifname: *const core::ffi::c_char,
        req: *const nmreq,
        flags: u64,
        arg: *const nm_desc,
    ) -> *mut nm_desc;

    /// Close a Netmap descriptor opened with [`nm_open`].
    #[link_name = "ffi_nm_close"]
    pub fn nm_close(d: *mut nm_desc) -> core::ffi::c_int;

    /// Map the Netmap memory region for a descriptor (see `nm_mmap` in
    /// `netmap_user.h`).
    #[link_name = "ffi_nm_mmap"]
    pub fn nm_mmap(d: *mut nm_desc, parent: *const nm_desc) -> core::ffi::c_int;

    /// Inject a single packet into the TX ring of a descriptor (see `nm_inject`
    /// in `netmap_user.h`).
    #[link_name = "ffi_nm_inject"]
    pub fn nm_inject(
        d: *mut nm_desc,
        buf: *const core::ffi::c_void,
        size: usize,
    ) -> core::ffi::c_int;
}

// ---------------------------------------------------------------------------
// ioctl numbers (bindgen cannot expand the _IO/_IOWR macros)
// ---------------------------------------------------------------------------

/// Synchronize the TX rings (`_IO('i', 148)` on Linux).
pub const NIOCTXSYNC: std::os::raw::c_ulong = 0x6994;
/// Synchronize the RX rings (`_IO('i', 149)` on Linux).
pub const NIOCRXSYNC: std::os::raw::c_ulong = 0x6995;
/// Register a Netmap interface with the kernel (`_IOWR('i', 146, struct nmreq)`).
pub const NIOCREGIF: std::os::raw::c_ulong = 0xc03c_6992;
/// Control-device request header (`_IOWR('i', 151, struct nmreq_header)`).
pub const NIOCCTRL: std::os::raw::c_ulong = 0xc058_6997;

// ---------------------------------------------------------------------------
// Registration-mode constants. The C header exposes the NR_REG_* values as an
// anonymous enum, which bindgen emits under a generated type name. The aliases
// below keep the older netmap-rs public names working and map onto the current
// API using `u32` values.
// ---------------------------------------------------------------------------

#[cfg(not(netmap_disabled))]
/// Attach only the host (SW) rings. Legacy name for `NR_REG_SW`.
pub const NR_REG_SW_ONLY: u32 = NR_REG_SW as u32;
#[cfg(not(netmap_disabled))]
/// Attach all NIC rings. Legacy name for `NR_REG_ALL_NIC`.
pub const NR_REG_NIC_ONLY: u32 = NR_REG_ALL_NIC as u32;
#[cfg(not(netmap_disabled))]
/// Attach both NIC and host rings. Legacy name for `NR_REG_NIC_SW`.
pub const NR_REG_NIC_AND_SW: u32 = NR_REG_NIC_SW as u32;

// ---------------------------------------------------------------------------
// Function-like macros from netmap_user.h (bindgen does not emit macros).
// ---------------------------------------------------------------------------

#[cfg(not(netmap_disabled))]
/// `NETMAP_TXRING(nifp, index)` — the TX ring at `index`, offset from the
/// `netmap_if` pointer.
///
/// # Safety
/// `nifp` must point to a valid `netmap_if`; `index` must be lower than
/// `nifp.ni_tx_rings`.
#[inline]
pub unsafe fn NETMAP_TXRING(nifp: *const netmap_if, index: u32) -> *mut netmap_ring {
    let offset = (*nifp).ring_ofs.as_ptr().add(index as usize).read();
    (nifp as *const u8).add(offset as usize) as *mut netmap_ring
}

#[cfg(not(netmap_disabled))]
/// `NETMAP_RXRING(nifp, index)` — the RX ring at `index`, offset from the
/// `netmap_if` pointer. RX rings start after all TX rings (HW + host).
///
/// # Safety
/// `nifp` must point to a valid `netmap_if`; `index` must be lower than
/// `nifp.ni_rx_rings`.
#[inline]
pub unsafe fn NETMAP_RXRING(nifp: *const netmap_if, index: u32) -> *mut netmap_ring {
    let base = (*nifp).ni_tx_rings + (*nifp).ni_host_tx_rings;
    let offset = (*nifp)
        .ring_ofs
        .as_ptr()
        .add((base + index) as usize)
        .read();
    (nifp as *const u8).add(offset as usize) as *mut netmap_ring
}

#[cfg(not(netmap_disabled))]
/// `NETMAP_IF(base, ofs)` — the `netmap_if` at `ofs` bytes into the mapped
/// memory region `base`.
///
/// # Safety
/// `base` must point into a valid Netmap shared-memory region.
#[inline]
pub unsafe fn NETMAP_IF(base: *const core::ffi::c_void, ofs: isize) -> *mut netmap_if {
    (base as *const u8).offset(ofs) as *mut netmap_if
}

#[cfg(not(netmap_disabled))]
/// `NETMAP_BUF(ring, index)` — pointer to buffer `index` inside `ring`'s buffer
/// pool.
///
/// # Safety
/// `ring` must point to a valid `netmap_ring`; `index` must be a valid buffer
/// index for that ring.
#[inline]
pub unsafe fn NETMAP_BUF(ring: *const netmap_ring, index: u32) -> *mut core::ffi::c_void {
    (ring as *const u8)
        .offset((*ring).buf_ofs as isize)
        .add(index as usize * (*ring).nr_buf_size as usize) as *mut core::ffi::c_void
}

#[cfg(netmap_disabled)]
mod disabled {
    /// Dummy stamp emitted only when the crate is built without Netmap (via
    /// `disable-netmap-kernel`, `DISABLE_NETMAP_KERNEL`, or missing headers) so
    /// the crate is not completely empty; use nothing from this module.
    pub const NETMAP_DISABLED: bool = true;
}

#[cfg(all(test, not(netmap_disabled)))]
mod tests {
    use super::*;
    use core::mem;

    #[test]
    fn struct_sizes() {
        // Values verified against `gcc -I <netmap include>` on x86-64 Linux.
        assert_eq!(mem::size_of::<netmap_slot>(), 16);
        assert_eq!(mem::align_of::<netmap_slot>(), 8);
        // netmap_ring has a flexible array member; its fixed part is 256 bytes.
        assert_eq!(mem::size_of::<netmap_ring>(), 256);
        assert_eq!(mem::size_of::<nmreq>(), 60);
        // A netmap_if has a fixed header of 56 bytes before the flexible
        // ring offset array.
        assert_eq!(mem::size_of::<netmap_if>(), 56);
        assert_eq!(mem::size_of::<nm_desc>(), 760);
        assert_eq!(mem::size_of::<nm_pkthdr>(), 56);
        assert_eq!(mem::size_of::<nm_stat>(), 12);
    }

    #[test]
    fn ioctl_constants() {
        assert_eq!(NIOCTXSYNC, 0x6994);
        assert_eq!(NIOCRXSYNC, 0x6995);
        assert_eq!(NETMAP_API, 14);
        assert_eq!(NR_REG_ALL_NIC as u32, 1);
        assert_eq!(NR_REG_SW as u32, 2);
    }

    #[test]
    fn reg_mode_constants_make_sense() {
        let all_nic = NR_REG_ALL_NIC as u32;
        let sw = NR_REG_SW as u32;
        let nic_sw = NR_REG_NIC_SW as u32;
        assert!(all_nic < sw);
        assert!(sw < nic_sw);
        assert_eq!(NR_REG_SW_ONLY, NR_REG_SW as u32);
        assert_eq!(NR_REG_NIC_ONLY, NR_REG_ALL_NIC as u32);
    }

    #[test]
    fn ring_macro_shims_are_unsafe_fn() {
        // Compile-time check that the macros are exposed as unsafe functions.
        fn _assert_fn_ptr(_f: unsafe fn(*const netmap_if, u32) -> *mut netmap_ring) {}
        _assert_fn_ptr(NETMAP_TXRING);
        _assert_fn_ptr(NETMAP_RXRING);
    }
}
