extern crate oping;
use oping::{Ping, PingResult};

fn do_stuff() -> PingResult<()> {
    let mut ping = Ping::new();
    ping.set_timeout(5.0)?; // timeout of 5.0 seconds
    ping.add_host("localhost")?; // fails here if socket can't be created
    ping.add_host("::1")?; // IPv4 / IPv6 addresses OK
    ping.add_host("1.2.3.4")?;
    let responses = ping.send()?; // waits for responses from all, or timeout
    for resp in responses {
        if resp.dropped > 0 {
            println!("No response from host: {}", resp.hostname);
        } else {
            println!(
                "Response from host {} (address {}): latency {} ms",
                resp.hostname, resp.address, resp.latency_ms
            );
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
