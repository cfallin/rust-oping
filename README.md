liboping bindings for Rust: simple ICMP pings
=============================================

This crate is a simple Rust binding for [liboping](http://noping.cc/), a
library that implements basic ICMP ping functionality. These bindings allow a
Rust program to send ping packets (possibly to multiple hosts in parallel) and
enumerate the responses.

This crate also includes a very simple program `rustping` that uses the
bindings to implement a barebones command-line ping utility.

This crate requires `liboping` to be installed on the system. If you do not
have `liboping`, you can install it either from the above link, or using your
system package manager. The crate looks for the appropriate linker flags using
`pkg-config`, so for this crate to build correctly, `pkg-config --libs
liboping` must return a meaningful result.

This crate was written by Chris Fallin &lt;cfallin@c1f.net&gt; and is released
under the MIT license.

Documentation is available [here](https://cfallin.github.io/rust-oping/oping/),
and the crate is available as `oping`
[on crates.io here](https://crates.io/crates/oping/).

*NOTE*: sending ping packets requires either running as `root` or setting a
capability on your binary, at least on Linux. This is a restriction enforced by
the system, not by this crate. To set the capability, run the following as
root:

    $ setcap cap_net_raw+ep $MY_BINARY    # allow binary to send ping packets
