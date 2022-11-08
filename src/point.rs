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
        let dy = make_positive((BigInt::from(2) * &self.y).mod_inverse(p).unwrap(), p);
        let dx = make_positive(BigInt::from(3) * self.x.pow(2u8) + a, p);
        let lamda = (dx * dy) % p;
        let x = make_positive((lamda.pow(2u8) - (BigInt::from(2) * &self.x)) % p, p);
        let y = make_positive((lamda * (&self.x - &x) - &self.y) % p, p);

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
            let dx = make_positive(&other.y - &self.y, p);
            let dy = make_positive(&other.x - &self.x, p);

            if dy == BigInt::zero() {
                return Point::new(BigInt::zero(), BigInt::zero());
            }

            let lambda = make_positive(dx * dy.mod_inverse(p).unwrap() % p, p);
            let x = make_positive((lambda.pow(2u8) - &self.x - &other.x) % p, p);
            let y = make_positive((lambda * (&self.x - &x) - &self.y) % p, p);

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

pub(crate) fn make_positive(q: BigInt, p: &BigInt) -> BigInt {
    if q < BigInt::zero() {
        q + p
    } else {
        q
    }
}
