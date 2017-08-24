extern crate oping;
use oping::{Ping, PingResult, PingError};

fn do_stuff() -> PingResult<()> {
    let mut ping = Ping::new();
    try!(ping.set_timeout(5.0));  // timeout of 5.0 seconds
    try!(ping.add_host("localhost"));  // fails here if socket can't be created
    try!(ping.add_host("::1"));  // IPv4 / IPv6 addresses OK
    
    /* Replacing the following 'try' macro with an example of how to use
     * my PingError::CustomError. I suggest adding a broken IP Address like
     * 1.300.1.1 to see it in action.
     *      - Timmonfette1
     *
     * try!(ping.add_host("1.2.3.4"));
     */
    let ip = "1.2.3.4";
    match ping.add_host(ip) {
        Ok(_)  => (),
        Err(e) =>
        {
            let error_message = format!("Failure from {} - {}", ip, e);
            return Err(PingError::CustomError(error_message));
        }
    }
    
    let responses = try!(ping.send());  // waits for responses from all, or timeout
    for resp in responses {
        if resp.dropped > 0 {
            println!("No response from host: {}", resp.hostname);
        } else {
            println!("Response from host {} (address {}): latency {} ms",
                     resp.hostname,
                     resp.address,
                     resp.latency_ms);
            println!("  all details: {:?}", resp);
        }
    }
    Ok(())
}

fn main() {
    match do_stuff() {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            ()
        }
    }
}
