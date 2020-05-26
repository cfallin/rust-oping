use std::process::Command;

extern crate cc;

const LIBOPING_VERSION: &str = "1.10.0";

fn main() {
    match std::env::var("RUST_OPING_USE_PKG_CONFIG") {
        Ok(_) => link_via_pkg_config(),
        Err(_) => build_and_statically_link()
    }
}

fn link_via_pkg_config() {
    if let Err(err) = pkg_config::Config::new()
        .exactly_version(LIBOPING_VERSION)
        .cargo_metadata(true)
        .probe("liboping")
    {
        panic!(
            "Could not find liboping via pkg-config: {:?}\nPKG_CONFIG_SYSROOT_DIR={}",
            err,
            std::env::var("PKG_CONFIG_SYSROOT_DIR").unwrap_or_default()
        );
    }
}

fn build_and_statically_link() {
    Command::new("sh")
        .current_dir("liboping/")
        .arg("autogen.sh")
        .status()
        .unwrap();
    Command::new("./configure")
        .current_dir("liboping/")
        .arg("--with-perl-bindings=no")
        .status()
        .unwrap();

    cc::Build::new()
        .define("HAVE_CONFIG_H", None)
        .file("liboping/src/liboping.c")
        .include("liboping/src/")
        .compile("liboping.a");
}
