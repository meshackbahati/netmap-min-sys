use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=c/netmap_shim.c");
    println!("cargo:rerun-if-env-changed=NETMAP_LOCATION");
    println!("cargo:rerun-if-env-changed=DISABLE_NETMAP_KERNEL");

    // Declare the custom cfg used to gate out Netmap-dependent code.
    println!("cargo:rustc-check-cfg=cfg(netmap_disabled)");

    // Allow disabling Netmap via env or feature flag.
    if cfg!(feature = "disable-netmap-kernel") || env::var("DISABLE_NETMAP_KERNEL").is_ok() {
        write_empty("Netmap disabled; skipping bindgen");
        return;
    }
    let install_dir = env::var("NETMAP_LOCATION").unwrap_or_else(|_| "/usr/local".into());
    let include_dir = PathBuf::from(&install_dir).join("include");
    let lib_dir = PathBuf::from(&install_dir).join("lib");

    // The published crate is built on hosts that never have the Netmap headers
    // installed (crates.io, docs.rs). Detect that up front and degrade to an
    // empty binding set instead of panicking inside bindgen, which would make
    // `cargo publish` fail. Consumers that build against a real Netmap install
    // set NETMAP_LOCATION (or install headers in /usr/local).
    let header = include_dir.join("net").join("netmap_user.h");
    if !header.exists() {
        println!(
            "cargo:warning=Netmap headers not found at {}; building empty bindings. \
             Set NETMAP_LOCATION to enable Netmap support.",
            header.display()
        );
        write_empty("Netmap headers not found; built empty bindings");
        return;
    }

    println!("cargo:warning=Linking against Netmap in: {}", install_dir);
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Generate Rust bindings for the netmap C headers.
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        // netmap_user.h gates struct nm_desc / nm_open / nm_close / ... behind
        // NETMAP_WITH_LIBS, so expose that section to bindgen.
        .clang_arg("-DNETMAP_WITH_LIBS")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Types: everything under netmap_* plus the nmreq_*/nm_desc family.
        .allowlist_type("netmap_.*")
        .allowlist_type("nmreq.*")
        .allowlist_type("nm_desc")
        .allowlist_type("nm_ifreq")
        .allowlist_type("nm_pkthdr")
        .allowlist_type("nm_stat")
        .allowlist_type("nm_csb_.*")
        .allowlist_type("nm_port.*")
        // These are emitted with the incorrect opaque layout by bindgen due to
        // the mutually-recursive nm_desc <-> nm_pkthdr cycle; they are defined
        // by hand in src/lib.rs instead.
        .blocklist_type("nm_desc")
        .blocklist_type("nm_pkthdr")
        .blocklist_type("nm_stat")
        // Constants: netmap/NR_/NS_/ioctl families and IFNAMSIZ.
        .allowlist_var("NETMAP_.*")
        .allowlist_var("NR_.*")
        .allowlist_var("NS_.*")
        .allowlist_var("NM_.*")
        .allowlist_var("IFNAMSIZ")
        .size_t_is_usize(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings with bindgen");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("binding.rs"))
        .expect("Couldn't write bindings to file");

    // Compile the C shim. The netmap_user.h helper functions (nm_open, nm_close,
    // ...) are `static` inside the header and are therefore not exported by any
    // netmap library; this shim compiles them into a static archive that is
    // linked directly into this crate.
    cc::Build::new()
        .file("c/netmap_shim.c")
        .include(&include_dir)
        .warnings(true)
        .compile("netmap_rs_shim");

    // Let downstream crates discover the netmap include/lib paths.
    println!("cargo:include={}", include_dir.display());
    println!(
        "cargo:rustc-env=NETMAP_INCLUDE_PATH={}",
        include_dir.display()
    );
    println!("cargo:rustc-env=NETMAP_LIB_PATH={}", lib_dir.display());
}

/// Write a minimal `binding.rs` (no Netmap symbols) and flip the
/// `netmap_disabled` cfg so the hand-written types in `src/lib.rs` are gated
/// out along with the generated bindings.
fn write_empty(reason: &str) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_path.join("binding.rs"), "// empty, Netmap disabled\n")
        .expect("Failed to write empty bindings.rs");
    println!("cargo:warning={}", reason);
    println!("cargo:rustc-cfg=netmap_disabled");
}
