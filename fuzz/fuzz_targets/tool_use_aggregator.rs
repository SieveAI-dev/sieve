#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    sieve_core::fuzz_helpers::fuzz_one_tool_use(data);
});
