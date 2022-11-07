use num_bigint_dig::{BigInt, ModInverse};
use num_traits::{Pow, Zero};

#[derive(Debug, Clone)]
pub struct Point {
    pub x: BigInt,
    pub y: BigInt,
}

impl Point {
    pub fn new(x: BigInt, y: BigInt) -> Self {
        Self { x, y }
    }

    pub fn identity() -> Self {
        Self {
            x: BigInt::zero(),
            y: BigInt::zero(),
        }
    }

    pub fn double(&self, p: &BigInt, a: &BigInt) -> Self {
        let temp: BigInt = (3 * self.x.pow(2u8) + a) * (2 * &self.y);
        let lambda = temp.mod_inverse(p).unwrap();
        let x = (lambda.pow(2u8) - (2 * &self.x)) % p;
        let y = (lambda * (&self.x - &x) - &self.y) % p;

        Point { x, y }
    }

    pub fn add(self, other: &Point, p: &BigInt, a: &BigInt) -> Self {
        if self.x == other.x && self.y == (&other.y * -1) {
            Point::identity()
        } else if self.x == other.x && self.y == other.y {
            self.double(p, a)
        } else if self.x == BigInt::zero() && self.y == BigInt::zero() {
            other.clone()
        } else if other.x == BigInt::zero() && other.y == BigInt::zero() {
            self
        } else {
            let lambda = ((&other.y - &self.y) * (&other.x - &self.x))
                .mod_inverse(p)
                .unwrap();
            let x = (lambda.pow(2u8) - &self.x - &other.x) % p;
            let y = (lambda * (&self.x - &x) - &self.y) % p;

            Point { x, y }
        }
    }

    pub fn multiply(mut self, mut n: BigInt, p: &BigInt, a: &BigInt) -> Self {
        let mut output = Point::identity();

        while n > BigInt::zero() {
            if &n % 2 != BigInt::zero() {
                output = output.add(&self, p, a);
            }

            self = self.double(p, a);

            n >>= 1;
        }

        output
    }
}
