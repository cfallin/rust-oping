const MIN_LIBOPING_VERSION: &str = "1.10.0";

fn main() {
    // We used to bundle a copy of liboping and try to build it automatically,
    // but that turned out to be way too much support burden, so now we just
    // look for it via `pkg-config`.
    link_via_pkg_config();
}

fn link_via_pkg_config() {
    if let Err(err) = pkg_config::Config::new()
        .atleast_version(MIN_LIBOPING_VERSION)
        .cargo_metadata(true)
        .probe("liboping")
    {
        eprintln!(
            concat!("Could not find liboping on your system! This Rust crate\n",
                    "requires the C library liboping to be installed (it is\n",
                    "simply a wrapper around this library). Please install\n",
                    "liboping from https://noping.cc/ or your system's package\n",
                    "manager, and ensure that `pkg-config` can provide its build\n",
                    "flags. If build issues persist, please do not open an issue\n",
                    "without first ensuring that `pkg-config --libs liboping`\n",
                    "returns something reasonable.")
        );
        panic!(
            "Could not find liboping via pkg-config: {:?}\nPKG_CONFIG_SYSROOT_DIR={}",
            err,
            std::env::var("PKG_CONFIG_SYSROOT_DIR").unwrap_or_default()
        );
    }
}
