use ratelimit::Handle;
use std::{thread, time::Duration};

use crate::constants::*;

pub fn create_rate_limiter() -> Handle {
    let num_cpus = num_cpus::get();

    let mut limiter = ratelimit::Builder::new()
        .capacity(num_cpus as u32)
        .quantum(1)
        .interval(Duration::new(0, *RPC_DELAY_NS.read().unwrap()))
        .build();

    let handle = limiter.make_handle();
    thread::spawn(move || {
        limiter.run();
    });
    handle
}
