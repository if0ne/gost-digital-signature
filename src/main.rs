use bitvec::prelude::*;

use crate::table::{A, C, PI, TAU};

mod table;

const BLOCK_SIZE: usize = 64;

type Block = [u8; BLOCK_SIZE];

fn main() {
    let bytes = "fbe2e5f0eee3c820fbeafaebef20fffbf0e1e0f0f520e0ed20e8ece0ebe5f0f2f120fff0eeec20f120faf2fee5e2202ce8f6f3ede220e8e6eee1e8f0f2d1202ce8f0f2e5e220e5d1".parse_bytes();

    println!("{:x?}", hash_512(&bytes));
    println!("{:x?}", hash_256(&bytes));
}

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

#[test]
fn byte_parser_test() {
    let message = "fbe2e5f0eee3c820fbeafaebef20fffbf0e1e0f0f520e0ed20e8ece0ebe5f0f2f120fff0eeec20f120faf2fee5e2202ce8f6f3ede220e8e6eee1e8f0f2d1202ce8f0f2e5e220e5d1";
    let bytes = message.parse_bytes();
    assert_eq!((bytes[0], bytes[1]), (0xfb, 0xe2));
}
