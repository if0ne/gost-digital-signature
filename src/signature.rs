use crate::curve::Curve;
use crate::hash_512;
use crate::point::Point;
use lazy_static::lazy_static;
use num_bigint_dig::{BigInt, ModInverse, RandBigInt, Sign};
use num_traits::{One, Zero};
use std::str::FromStr;

pub struct Signature {
    pub sign: BigInt,
    pub r: BigInt,
    pub s: BigInt,
}

lazy_static! {
    static ref P: Point = Point::new(
        BigInt::from_str("1928356944067022849399309401243137598997786635459507974357075491307766592685835441065557681003184874819658004903212332884252335830250729527632383493573274").unwrap(),
        BigInt::from_str(
            "2288728693371972859970012155529478416353562327329506180314497425931102860301572814141997072271708807066593850650334152381857347798885864807605098724013854",
        )
        .unwrap(),
    );
    static ref Q: Point = Point::new(
        BigInt::from_str(
            "909546853002536596556690768669830310006929272546556281596372965370312498563182320436892870052842808608262832456858223580713780290717986855863433431150561",
        )
        .unwrap(),
        BigInt::from_str(
            "2921457203374425620632449734248415455640700823559488705164895837509539134297327397380287741428246088626609329139441895016863758984106326600572476822372076",
        )
        .unwrap(),
    );
}

impl Signature {
    pub fn sign(message: &[u8], key: BigInt, curve: Curve) -> Self {
        let hash = hash_512(message);
        let hash = BigInt::from_bytes_le(Sign::Plus, &hash);
        let mut e = hash % &curve.p;
        if e == BigInt::zero() {
            e = BigInt::one();
        }

        let (r, s) = loop {
            let k = Self::rand_k(&curve.q);
            let big_c = P.clone().multiply(k.clone(), &curve.p, &curve.a);
            let r = big_c.x % &curve.q;
            if r == BigInt::zero() {
                println!("r zero");
                continue;
            }
            let s = (&r * &key + &k * &e) % &curve.q;
            if s == BigInt::zero() {
                println!("s zero");
                continue;
            }
            break (r, s);
        };

        let bytes = [r.to_bytes_le().1, s.to_bytes_le().1].concat();
        let sign = BigInt::from_bytes_le(Sign::Plus, &bytes);

        Self { sign, r, s }
    }

    pub fn verify(&self, message: &[u8], curve: Curve) -> bool {
        if !(self.r > BigInt::zero()
            && (self.r < curve.q)
            && self.s > BigInt::zero()
            && self.s < curve.q)
        {
            return false;
        }

        let hash = hash_512(message);
        let hash = BigInt::from_bytes_le(Sign::Plus, &hash);
        let mut e = hash % &curve.q;
        if e.is_zero() {
            e = BigInt::one();
        }

        let v = e.mod_inverse(&curve.p).unwrap();
        let z1 = (&self.s * &v) % &curve.q;
        let z2 = -&self.r * &v % &curve.q;
        let big_c = P.clone().multiply(z1, &curve.p, &curve.a).add(
            &Q.clone().multiply(z2, &curve.p, &curve.a),
            &curve.p,
            &curve.a,
        );
        let big_r = big_c.x % &curve.q;

        big_r == self.r
    }

    fn rand_k(upper: &BigInt) -> BigInt {
        rand::thread_rng().gen_bigint_range(&BigInt::zero(), upper)
    }
}
