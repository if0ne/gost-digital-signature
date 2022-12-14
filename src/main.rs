use crate::curve::Curve;
use crate::signature::Signature;
use bitvec::prelude::*;
use num_bigint_dig::BigInt;
use std::str::FromStr;

use crate::table::{A, C, PI, TAU};

mod curve;
mod point;
mod signature;
mod table;

const BLOCK_SIZE: usize = 64;

type Block = [u8; BLOCK_SIZE];

fn main() {
    let message = "Юра - балерина".bytes().collect::<Vec<u8>>();
    let key = BigInt::from_str(
        "55441196065363246126355624130324183196576709222340016572108097750006097525544",
    )
    .unwrap();
    let curve = Curve::new(
        BigInt::from_str("7").unwrap(),
        BigInt::from_str(
            "43308876546767276905765904595650931995942111794451039583252968842033849580414",
        )
        .unwrap(),
        BigInt::from_str(
            "57896044618658097711785492504343953926634992332820282019728792003956564821041",
        )
        .unwrap(),
        BigInt::from_str(
            "57896044618658097711785492504343953927082934583725450622380973592137631069619",
        )
        .unwrap(),
        BigInt::from_str(
            "57896044618658097711785492504343953927082934583725450622380973592137631069619",
        )
        .unwrap(),
    );
    let sign = Signature::sign(&message, key, curve.clone());
    println!("{}", sign.verify(&message, curve));
}

/*fn main() {
    /*let bytes = include_bytes!("text.txt");
    let time = std::time::Instant::now();
    let hash = hash_512(bytes);
    let elapsed = time.elapsed();
    println!("Hash: {:x?}", hash);
    println!("Time: {} ms", elapsed.as_millis());*/
}*/

fn hash_512(message: &[u8]) -> Block {
    hash([0u8; 64], message)
}

fn hash_256(message: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let hash = hash([1u8; 64], message);
    output[..32].copy_from_slice(&hash[..32]);

    output
}

fn hash(iv: Block, message: &[u8]) -> Block {
    let mut hash = iv;
    let mut n = [0u8; 64];
    let mut sigma = [0u8; 64];

    let mut len = message.len();
    let mut p = 0;

    let mut n_512 = [0u8; 64];
    n_512[62] = 0x02;

    while len >= 64 {
        let mut section = [0u8; 64];
        for i in 0..64 {
            section[i] = message[message.len() - (p + 1) * 64 + i];
        }
        hash = compression(n, hash, section);
        n = add(n, n_512);
        sigma = add(sigma, section);

        len -= 64;
        p += 1;
    }

    len *= 8;
    let rest = &message[..(message.len() - p * 64)];
    let section = padding(rest);

    let mut v = [0u8; 64];
    let v0 = [0u8; 64];
    v[63] = (len & 0xFF) as u8;
    v[62] = (len >> 8) as u8;

    hash = compression(n, hash, section);

    n = add(n, v);
    sigma = add(sigma, section);

    hash = compression(v0, hash, n);
    hash = compression(v0, hash, sigma);

    hash
}

fn padding(m: &[u8]) -> Block {
    let mut output = [0u8; BLOCK_SIZE];
    for i in 0..m.len() {
        output[BLOCK_SIZE - m.len() + i] = m[i]
    }
    if m.len() < BLOCK_SIZE {
        output[BLOCK_SIZE - m.len() - 1] = 0x01;
    }

    output
}

pub fn add(l: Block, r: Block) -> Block {
    let mut result = [0u8; 64];
    let mut t = 0i32;
    for i in (0..64).rev() {
        t = l[i] as i32 + r[i] as i32 + (t >> 8);
        result[i] = (t & 0xFF) as u8;
    }
    result
}

fn xor(k: Block, a: Block) -> Block {
    let mut output = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        output[i] = k[i] ^ a[i];
    }

    output
}

fn bijective(a: Block) -> Block {
    let mut output = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        output[i] = PI[a[i] as usize];
    }

    output
}

fn permutation(a: Block) -> Block {
    let mut output = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        output[i] = a[TAU[i]];
    }

    output
}

fn linear(a: Block) -> Block {
    let mut output = [0u8; 64];

    for i in 0..8 {
        let mut t = 0u64;
        let mut temp = [0u8; 8];

        for j in 0..8 {
            temp[j] = a[i * 8 + j];
        }
        let bits = temp.view_bits::<Msb0>();
        for j in 0..64 {
            if bits[j] {
                t ^= A[j];
            }
        }

        let mut t = t.to_ne_bytes();
        t.reverse();
        for j in 0..8 {
            output[i * 8 + j] = t[j];
        }
    }

    output
}

fn linear_permutation_bijective(a: Block) -> Block {
    linear(permutation(bijective(a)))
}

fn key_schedule(k: Block, i: usize) -> Block {
    linear_permutation_bijective(xor(k, C[i]))
}

fn e_transformation(k: Block, m: Block) -> Block {
    let mut s = xor(k, m);
    let mut k = k;
    for i in 0..12 {
        s = linear_permutation_bijective(s);
        k = key_schedule(k, i);
        s = xor(k, s);
    }

    s
}

fn compression(n: Block, h: Block, m: Block) -> Block {
    let k = xor(h, n);
    let k = linear_permutation_bijective(k);
    let t = e_transformation(k, m);
    let t = xor(t, h);
    xor(t, m)
}

trait ByteParse {
    fn parse_bytes(self) -> Vec<u8>;
}

impl ByteParse for &str {
    fn parse_bytes(self) -> Vec<u8> {
        let mut vec = vec![];
        for i in (0..self.len()).step_by(2) {
            vec.push(u8::from_str_radix(&self[i..(i + 2)], 16).unwrap())
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use crate::{hash_512, sign, verify, ByteParse};
    use num_bigint_dig::BigInt;
    use std::str::FromStr;

    const MSG: [u8; 63] = [
        0x32u8, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38,
        0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33,
        0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38,
        0x37, 0x36, 0x35, 0x34, 0x33, 0x32, 0x31, 0x30, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33,
        0x32, 0x31, 0x30,
    ];

    #[test]
    fn byte_parser_test() {
        let message = "fbe2e5f0eee3c820fbeafaebef20fffbf0e1e0f0f520e0ed20e8ece0ebe5f0f2f120fff0eeec20f120faf2fee5e2202ce8f6f3ede220e8e6eee1e8f0f2d1202ce8f0f2e5e220e5d1";
        let bytes = message.parse_bytes();
        assert_eq!((bytes[0], bytes[1]), (0xfb, 0xe2));
    }

    #[test]
    fn hasher() {
        let hash_actual = hash_512(&MSG);

        let hash_expected = [
            0x48u8, 0x6F, 0x64, 0xC1, 0x91, 0x78, 0x79, 0x41, 0x7F, 0xEF, 0x08, 0x2B, 0x33, 0x81,
            0xA4, 0xE2, 0x11, 0xC3, 0x24, 0xF0, 0x74, 0x65, 0x4C, 0x38, 0x82, 0x3A, 0x7B, 0x76,
            0xF8, 0x30, 0xAD, 0x00, 0xFA, 0x1F, 0xBA, 0xE4, 0x2B, 0x12, 0x85, 0xC0, 0x35, 0x2F,
            0x22, 0x75, 0x24, 0xBC, 0x9A, 0xB1, 0x62, 0x54, 0x28, 0x8D, 0xD6, 0x86, 0x3D, 0xCC,
            0xD5, 0xB9, 0xF5, 0x4A, 0x1A, 0xD0, 0x54, 0x1B,
        ];

        assert_eq!(hash_actual, hash_expected);
    }

    #[test]
    fn signer() {
        let hash = hash_512(&MSG);

        let p = BigInt::from_str(
            "57896044618658097711785492504343953926634992332820282019728792003956564821041",
        )
        .unwrap();
        let a = BigInt::from_str("7").unwrap();
        let b = BigInt::from_str(
            "43308876546767276905765904595650931995942111794451039583252968842033849580414",
        )
        .unwrap();
        let m = BigInt::from_str(
            "57896044618658097711785492504343953927082934583725450622380973592137631069619",
        )
        .unwrap();
        let q = BigInt::from_str(
            "57896044618658097711785492504343953927082934583725450622380973592137631069619",
        )
        .unwrap();

        let point = (
            BigInt::from_str("2").unwrap(),
            BigInt::from_str(
                "4018974056539037503335449422937059775635739389905545080690979365213431566280",
            )
            .unwrap(),
        );

        let d = BigInt::from_str(
            "55441196065363246126355624130324183196576709222340016572108097750006097525544",
        )
        .unwrap();

        let check_point = (
            BigInt::from_str(
                "57520216126176808443631405023338071176630104906313632182896741342206604859403",
            )
            .unwrap(),
            BigInt::from_str(
                "17614944419213781543809391949654080031942662045363639260709847859438286763994",
            )
            .unwrap(),
        );

        let (sign, r, s) = sign(&p, &a, &q, &point, &d, hash);
        let is_verified = verify(&sign, &r, &s, &q, &p, &point, &check_point, &a, hash);

        assert!(is_verified);
    }
}
