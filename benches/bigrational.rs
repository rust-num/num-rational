#![feature(test)]

extern crate test;

use num_bigint::BigInt;
use num_rational::{BigRational, Ratio};
// use num_traits::{FromPrimitive, Num, One, Zero};
use test::Bencher;

mod rng;
use rng::get_rng;

// allocating 'a' and 'b' is about 70ns.
// allocating BigRational(a,b) is about 970ns.
fn alloc_bench(b: &mut Bencher) {
    use rand::RngCore;
    let mut rng = get_rng();
    b.iter(|| {
        let a = BigInt::from(rng.next_u64());
        let b = BigInt::from(rng.next_u64());
        BigRational::new(a, b)
        // (a, b)
    });
}

fn alloc_fast_bench(b: &mut Bencher) {
    use rand::RngCore;
    let mut rng = get_rng();
    b.iter(|| {
        let a = BigInt::from(rng.next_u64());
        let b = BigInt::from(rng.next_u64());
        BigRational::new_fast(a, b)
        // (a, b)
    });
}

fn alloc_u64_bench(b: &mut Bencher) {
    use rand::RngCore;
    let mut rng = get_rng();
    b.iter(|| {
        let a = rng.next_u64();
        let b = rng.next_u64();
        Ratio::new_fast(a, b)
        // (a, b)
    });
}

#[bench]
fn alloc_0(b: &mut Bencher) {
    alloc_bench(b);
}

#[bench]
fn alloc_fast_0(b: &mut Bencher) {
    alloc_fast_bench(b);
}

#[bench]
fn alloc_u64_0(b: &mut Bencher) {
    alloc_u64_bench(b);
}
