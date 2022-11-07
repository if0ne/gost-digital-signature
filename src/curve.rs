use num_bigint_dig::BigInt;

#[derive(Clone)]
pub struct Curve {
    pub a: BigInt,
    pub b: BigInt,
    pub p: BigInt,
    pub m: BigInt,
    pub q: BigInt,
}

impl Curve {
    pub fn new(a: BigInt, b: BigInt, p: BigInt, m: BigInt, q: BigInt) -> Self {
        Self { a, b, p, m, q }
    }
}
