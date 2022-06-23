#![feature(test)]

extern crate test;

use num_bigint::BigInt;
use num_rational::{BigRational, Ratio};
use test::Bencher;

mod rng;
use rng::get_rng;

#[bench]
fn alloc_ratio_bigint_bench(b: &mut Bencher) {
    use rand::RngCore;
    let mut rng = get_rng();
    b.iter(|| {
        let a = BigInt::from(rng.next_u64());
        let b = BigInt::from(rng.next_u64());
        BigRational::new(a, b)
    });
}

#[bench]
fn alloc_ratio_u64_bench(b: &mut Bencher) {
    use rand::RngCore;
    let mut rng = get_rng();
    b.iter(|| {
        let a = rng.next_u64();
        let b = rng.next_u64();
        Ratio::new(a, b)
    });
}
