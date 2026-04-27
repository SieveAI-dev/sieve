fn main() {
    afl::fuzz!(|data: &[u8]| {
        sieve_core::fuzz_helpers::fuzz_one_sse(data);
    });
}
