#![allow(unused)]
#![feature(bigint_helper_methods)]

use criterion::BenchmarkId;

pub const MODULUS: [u64; 4] = [
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

#[inline]
pub fn branching_sub(x: &[u64; 4], y: &[u64; 4]) -> [u64; 4] {
    let mut d0;
    let mut d1;
    let mut d2;
    let mut d3;
    let mut borrow;

    (d0, borrow) = x[0].overflowing_sub(y[0]);
    (d1, borrow) = x[1].borrowing_sub(y[1], borrow);
    (d2, borrow) = x[2].borrowing_sub(y[2], borrow);
    (d3, borrow) = x[3].borrowing_sub(y[3], borrow);

    // If underflow occurred on the final limb, add the modulus.
    if borrow {
        (d0, borrow) = d0.overflowing_add(MODULUS[0]);
        (d1, borrow) = d1.carrying_add(MODULUS[1], borrow);
        (d2, borrow) = d2.carrying_add(MODULUS[2], borrow);
        (d3, _) = d3.carrying_add(MODULUS[3], borrow);
    }
    [d0, d1, d2, d3]
}

pub fn sub(x: &[u64; 4], y: &[u64; 4]) -> [u64; 4] {
    /// Compute a - (b + borrow), returning the result and the new borrow.
    #[inline(always)]
    const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
        let ret = (a as u128).wrapping_sub((b as u128) + ((borrow >> 63) as u128));
        (ret as u64, (ret >> 64) as u64)
    }
    /// Compute a + b + carry, returning the result and the new carry over.
    #[inline(always)]
    const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
        let ret = (a as u128) + (b as u128) + (carry as u128);
        (ret as u64, (ret >> 64) as u64)
    }
    let (d0, borrow) = sbb(x[0], y[0], 0);
    let (d1, borrow) = sbb(x[1], y[1], borrow);
    let (d2, borrow) = sbb(x[2], y[2], borrow);
    let (d3, borrow) = sbb(x[3], y[3], borrow);

    // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
    // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
    let (d0, carry) = adc(d0, MODULUS[0] & borrow, 0);
    let (d1, carry) = adc(d1, MODULUS[1] & borrow, carry);
    let (d2, carry) = adc(d2, MODULUS[2] & borrow, carry);
    let (d3, _) = adc(d3, MODULUS[3] & borrow, carry);

    [d0, d1, d2, d3]
}

pub fn sub_nightly(x: &[u64; 4], y: &[u64; 4]) -> [u64; 4] {
    #[inline(always)]
    fn adc(a: u64, b: u64, carry: bool) -> (u64, bool) {
        a.carrying_add(b, carry)
    }

    /// Compute a - b - borrow, returning the result and the new borrow.
    #[inline(always)]
    fn sbb(a: u64, b: u64, borrow: bool) -> (u64, bool) {
        a.borrowing_sub(b, borrow)
    }

    let (d0, borrow) = x[0].overflowing_sub(y[0]);
    let (d1, borrow) = sbb(x[1], y[1], borrow);
    let (d2, borrow) = sbb(x[2], y[2], borrow);
    let (d3, borrow) = sbb(x[3], y[3], borrow);

    let borrow = 0u64.wrapping_sub(borrow as u64);
    // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
    // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
    let (d0, carry) = d0.overflowing_add(MODULUS[0] & borrow);
    let (d1, carry) = adc(d1, MODULUS[1] & borrow, carry);
    let (d2, carry) = adc(d2, MODULUS[2] & borrow, carry);
    let (d3, _) = adc(d3, MODULUS[3] & borrow, carry);

    [d0, d1, d2, d3]
}

use criterion::{criterion_group, criterion_main, Criterion};
use rand::distributions::uniform::SampleBorrow;

pub fn criterion_benchmark(c: &mut Criterion) {
    let x: [u64; 4] = [(); 4].map(|_| rand::random());
    let y: [u64; 4] = [(); 4].map(|_| rand::random());

    let mut group = c.benchmark_group("Bigint subtraction methods");

    group.bench_with_input(
        BenchmarkId::new("branching_sub", ""),
        &(x, y),
        |b, (x, y)| b.iter(|| branching_sub(x, y)),
    );

    group.bench_with_input(BenchmarkId::new("sub", ""), &(x, y), |b, (x, y)| {
        b.iter(|| sub(x, y))
    });

    group.bench_with_input(BenchmarkId::new("sub nightly", ""), &(x, y), |b, (x, y)| {
        b.iter(|| sub_nightly(x, y))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
