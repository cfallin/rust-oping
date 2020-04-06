use std::process::Command;

extern crate cc;

fn main() {
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
