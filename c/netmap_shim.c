/*
 * netmap C shim.
 *
 * The helper functions exposed by <net/netmap_user.h> (nm_open(), nm_close(),
 * nm_mmap(), nm_inject(), ...) are declared `static`, meaning they are not
 * exported by any netmap library. This file includes the header and re-exports
 * thin, non-static wrappers so that `netmap-min-sys` can link against them.
 *
 * The wrappers are self-contained: they only rely on libc (open/ioctl/mmap/...)
 * and the netmap ioctl interface, so no libnetmap is required at build or
 * runtime.
 *
 * `NETMAP_WITH_LIBS` must be defined so that netmap_user.h provides the
 * "simple I/O libraries" section (`struct nm_desc`, nm_open(), nm_close(), ...).
 */
#define NETMAP_WITH_LIBS
#include <net/netmap_user.h>

struct nm_desc *ffi_nm_open(const char *ifname, const struct nmreq *req,
                            uint64_t flags, const struct nm_desc *arg) {
    return nm_open(ifname, req, flags, arg);
}

int ffi_nm_close(struct nm_desc *d) {
    return nm_close(d);
}

int ffi_nm_mmap(struct nm_desc *d, const struct nm_desc *parent) {
    return nm_mmap(d, parent);
}

int ffi_nm_inject(struct nm_desc *d, const void *buf, size_t size) {
    return nm_inject(d, buf, size);
}
