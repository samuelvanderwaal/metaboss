use ratelimit::Handle;
use std::{thread, time::Duration};

use crate::constants::*;

pub fn create_rate_limiter() -> Handle {
    let num_cpus = num_cpus::get();

    let mut limiter = ratelimit::Builder::new()
        .capacity(num_cpus as u32)
        .quantum(1)
        .interval(Duration::new(
            0,
            (TIME_PER_MAX_REQUESTS_NS / MAX_REQUESTS) as u32 + TIME_BUFFER_NS,
        ))
        .build();

    let handle = limiter.make_handle();
    thread::spawn(move || {
        limiter.run();
    });
    handle
}
