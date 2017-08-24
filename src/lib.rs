#![allow(dead_code, non_camel_case_types)]

//! # oping library bindings
//!
//! This set of bindings allows the use of [liboping](http://noping.cc/) to
//! send pings via the ICMP protocol. (The library is included inside the
//! crate, so it is not necessary to install the library on your system
//! separately.)
//!
//! The central type is `Ping`, which wraps a set of options and one or more
//! ping sockets open to particular destinations. When `send()` is called, the
//! implementation sends a ping to each destination added, and listens for
//! replies from all, up to the specified timeout. It then provides an iterator
//! over response information from each destination.
//!
//! Sending a ping via a ping socket usually requires the program to run as
//! `root`. This set of bindings has only been tested on Linux.
//!
//! This crate contains a small command-line utility `rustping` which
//! demonstrates the use of these bindings.
//!
//! # Example
//!
//! ```
//! use oping::{Ping, PingResult};
//!
//! fn do_pings() -> PingResult<()> {
//!     let mut ping = Ping::new();
//!     try!(ping.set_timeout(5.0));  // timeout of 5.0 seconds
//!     try!(ping.add_host("localhost"));  // fails here if socket can't be created
//!     try!(ping.add_host("other_host"));
//!     try!(ping.add_host("::1"));  // IPv4 / IPv6 addresses OK
//!     try!(ping.add_host("1.2.3.4"));
//!     let responses = try!(ping.send());
//!     for resp in responses {
//!         if resp.dropped > 0 {
//!             println!("No response from host: {}", resp.hostname);
//!         } else {
//!             println!("Response from host {} (address {}): latency {} ms",
//!                 resp.hostname, resp.address, resp.latency_ms);
//!             println!("    all details: {:?}", resp);
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use std::mem::transmute;
use std::os::raw::c_char;
use std::ffi::{CStr, CString, NulError};
use std::default::Default;
use std::error;
use std::fmt;

extern crate libc;
use libc::{AF_INET, AF_INET6};
use libc::c_int;

use self::PingError::{
    LibOpingError,
    NulByteError,
    CustomError,
};

/// Address family (IPv4 or IPv6) used to send/receive a ping.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AddrFamily {
    IPV4,
    IPV6,
}

impl Default for AddrFamily {
    fn default() -> AddrFamily {
        AddrFamily::IPV4
    }
}

#[repr(C)]
enum PingOption {
    TIMEOUT = 1,
    TTL = 2,
    AF = 4,
    DATA = 8,
    SOURCE = 16,
    DEVICE = 32,
    QOS = 64,
}

#[repr(C)]
enum PingIterInfo {
    HOSTNAME = 1,
    ADDRESS = 2,
    FAMILY = 3,
    LATENCY = 4,
    SEQUENCE = 5,
    IDENT = 6,
    DATA = 7,
    USERNAME = 8,
    DROPPED = 9,
    RECV_TTL = 10,
    RECV_QOS = 11,
}

enum PingObj {}
enum PingObjIter {}

extern "C" {
    fn ping_construct() -> *mut PingObj;
    fn ping_destroy(obj: *mut PingObj);
    fn ping_setopt(obj: *mut PingObj, opt: PingOption, val: *mut u8) -> i32;
    fn ping_send(obj: *mut PingObj) -> i32;
    fn ping_host_add(obj: *mut PingObj, host: *const c_char) -> i32;
    fn ping_host_remove(obj: *mut PingObj, host: *const c_char) -> i32;
    fn ping_iterator_get(obj: *mut PingObj) -> *mut PingObjIter;
    fn ping_iterator_next(obj: *mut PingObjIter) -> *mut PingObjIter;
    fn ping_iterator_get_info(iter: *mut PingObjIter,
                              info: PingIterInfo,
                              buf: *mut u8,
                              size: *mut usize)
                              -> i32;
    fn ping_get_error(obj: *mut PingObj) -> *const c_char;
}

/// An error resulting from a ping option-setting or send/receive operation.
#[derive(Debug)]
pub enum PingError {
    /// A `liboping` internal error
    LibOpingError(String),
    /// A `std::ffi::NulError` that occurred while trying to convert a hostname string
    NulByteError(NulError),
    /// An error with a custom error message
    CustomError(String),
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LibOpingError(ref err) => write!(f, "oping::PingError::LibOpingError: {}", err),
            NulByteError(ref err) => write!(f, "oping::PingError::NulByteError: {}", err),
            CustomError(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for PingError {
    fn description(&self) -> &str {
        match *self {
            LibOpingError(_) => "a liboping internal error was encountered",
            NulByteError(ref e) => e.description(),
            CustomError(ref e) => &e,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            NulByteError(ref err) => Some(err),
            _ => None,
        }
    }
}

pub type PingResult<T> = Result<T, PingError>;

/// A `Ping` struct represents the state of one particular ping instance:
/// several instance-wide options (timeout, TTL, QoS setting, etc.), and
/// a list of hostnames/addresses to ping. It is consumed when a single set
/// of ping packets are sent to the listed destinations, resulting in an
/// iterator over the responses returned.
pub struct Ping {
    obj: *mut PingObj,
}

impl Drop for Ping {
    fn drop(&mut self) {
        unsafe { ping_destroy(self.obj) };
    }
}

macro_rules! try_c {
    ($obj: expr, $e: expr) => (
        if $e != 0 {
            let err = CStr::from_ptr(ping_get_error($obj));
            let s = String::from(err.to_str().unwrap());
            return Err(LibOpingError(s));
        }
    )
}


impl Ping {
    /// Create a new `Ping` context.
    pub fn new() -> Ping {
        let obj = unsafe { ping_construct() };
        assert!(!obj.is_null());
        Ping { obj: obj }
    }

    /// Set the timeout, in seconds, for which we will wait for replies from
    /// all listed destinations.
    pub fn set_timeout(&mut self, timeout: f64) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                   ping_setopt(self.obj, PingOption::TIMEOUT, transmute(&timeout)));
        }
        Ok(())
    }

    /// Set the TTL to set on the ping packets we send. Note that if a packet
    /// is sent with a TTL that is too low for the route, it may be dropped.
    pub fn set_ttl(&mut self, ttl: i32) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                   ping_setopt(self.obj, PingOption::TTL, transmute(&ttl)));
        }
        Ok(())
    }

    /// Set the preferred address family to use: IPv4 or IPv6.
    pub fn set_addr_family(&mut self, af: AddrFamily) -> PingResult<()> {
        let fam: c_int = match af {
            AddrFamily::IPV4 => AF_INET,
            AddrFamily::IPV6 => AF_INET6,
        };
        unsafe {
            try_c!(self.obj,
                   ping_setopt(self.obj, PingOption::AF, transmute(&fam)));
        }
        Ok(())
    }

    /// Set the value of the "quality of service" field to use on outgoing
    /// ping packets.
    pub fn set_qos(&mut self, qos: u8) -> PingResult<()> {
        unsafe {
            try_c!(self.obj,
                   ping_setopt(self.obj, PingOption::QOS, transmute(&qos)));
        }
        Ok(())
    }

    /// Add a ping destination. `hostname` may be a hostname to look up via
    /// the system's name resolution (DNS, etc), or a numeric IPv4 or IPv6
    /// address.
    ///
    /// Note that this method is the point at which a ping socket is actually
    /// created. Hence, if the program does not have the appropriate permission
    /// to send ping packets, this is usually where the error will occur.
    pub fn add_host(&mut self, hostname: &str) -> PingResult<()> {
        let cstr = match CString::new(hostname.as_bytes()) {
            Ok(s) => s,
            Err(e) => return Err(NulByteError(e)),
        };
        unsafe {
            try_c!(self.obj, ping_host_add(self.obj, cstr.as_ptr()));
        }
        Ok(())
    }

    /// Remove a destination that was previously added. If the hostname does
    /// not match one that was added previously, an error will be returned.
    pub fn remove_host(&mut self, hostname: &str) -> PingResult<()> {
        let cstr = match CString::new(hostname.as_bytes()) {
            Ok(s) => s,
            Err(e) => return Err(NulByteError(e)),
        };
        unsafe {
            try_c!(self.obj, ping_host_add(self.obj, cstr.as_ptr()));
        }
        Ok(())
    }

    /// Sends a single ping to all listed destinations, waiting until either
    /// replies are received from all destinations or the timeout is reached.
    /// Returns an iterator over all replies.
    ///
    /// A `Ping` context may only be used once; hence, this method consumes
    /// the context.
    pub fn send(self) -> PingResult<PingIter> {
        unsafe {
            let result = ping_send(self.obj);
            if result < 0 {
                try_c!(self.obj, result);  // should return error.
                unreachable!()
            } else {
                Ok(self.iter())
            }
        }
    }

    fn iter(self) -> PingIter {
        let ptr = unsafe { ping_iterator_get(self.obj) };
        PingIter {
            pingobj: self,
            iter: ptr,
        }
    }
}

/// An iterator over ping responses. Will return one `PingItem` for each
/// destination that was added to the `Ping` context.
pub struct PingIter {
    pingobj: Ping,
    iter: *mut PingObjIter,
}

/// One ping response from a destination that was added to the `Ping` context.
#[derive(Clone, Debug, Default)]
pub struct PingItem {
    /// The hostname as resolved by the library, possibly resolved to a more
    /// canonical name.
    pub hostname: String,
    /// The address as resolved by the library, either IPv4 or IPv6, in textual
    /// form.
    pub address: String,
    /// The address family (IPv4 or IPv6) used to ping the destination.
    pub family: AddrFamily,
    /// The latency of the response, if any, in milliseconds.
    pub latency_ms: f64,
    /// The dropped-packet count: either 0 or 1.
    pub dropped: u32,
    /// The sequence number of the ping.
    pub seq: i32,
    /// The TTL on the received response.
    pub recv_ttl: i32,
    /// The QoS (quality of service) field on the received response.
    pub recv_qos: u8,
}

macro_rules! get_str_field {
    ($iter: expr, $field: expr, $vec: expr) => (
        unsafe {
            let ptr = $vec.as_mut_ptr();
            let mut size: usize = $vec.capacity();
            if ping_iterator_get_info($iter, $field, ptr, &mut size as *mut usize) != 0 {
                return None;
            }
            CStr::from_ptr(ptr as *const _).to_str().unwrap().to_string()
        }
    )
}

macro_rules! get_num_field {
    ($iter: expr, $field: expr, $vec: expr,$t: ty) => (
        unsafe {
            let ptr = $vec.as_mut_ptr();
            let mut size: usize = $vec.capacity();
            if ping_iterator_get_info($iter, $field, ptr, &mut size as *mut usize) != 0 {
                return None;
            }
            let cast_ptr: *const $t = transmute(ptr);
            *cast_ptr
        }
    )
}

impl Iterator for PingIter {
    type Item = PingItem;
    fn next(&mut self) -> Option<PingItem> {
        if self.iter == (0 as *mut PingObjIter) {
            return None;
        }

        let mut ret: PingItem = Default::default();
        let mut buf = Vec::<u8>::with_capacity(1024);

        ret.hostname = get_str_field!(self.iter, PingIterInfo::HOSTNAME, buf);
        ret.address = get_str_field!(self.iter, PingIterInfo::ADDRESS, buf);
        ret.family = match get_num_field!(self.iter, PingIterInfo::FAMILY, buf, i32) {
            libc::AF_INET => AddrFamily::IPV4,
            libc::AF_INET6 => AddrFamily::IPV6,
            _ => AddrFamily::IPV4,
        };
        ret.latency_ms = get_num_field!(self.iter, PingIterInfo::LATENCY, buf, f64);
        ret.dropped = get_num_field!(self.iter, PingIterInfo::DROPPED, buf, u32);
        ret.seq = get_num_field!(self.iter, PingIterInfo::SEQUENCE, buf, i32);
        ret.recv_ttl = get_num_field!(self.iter, PingIterInfo::RECV_TTL, buf, i32);
        ret.recv_qos = get_num_field!(self.iter, PingIterInfo::RECV_QOS, buf, u8);

        self.iter = unsafe { ping_iterator_next(self.iter) };

        Some(ret)
    }
}

mod test {
    // N.B.: this test does not actually add any hosts or send pings, because
    // these actions usually require `root` privileges, and we want unit tests
    // to run as an ordinary user. As such we'll have to be content not to test
    // the host-add/remove, packet send/receive, or iterator functionality.

    #[test]
    fn test_basic_opts() {
        let mut p = ::Ping::new();
        assert!(p.set_timeout(5.0).is_ok());
        assert!(p.set_ttl(42).is_ok());
        assert!(p.set_addr_family(::AddrFamily::IPV4).is_ok());
        assert!(p.set_qos(42).is_ok());
    }
}
