use std::cmp;

use blake2::Blake2b;
use digest::{Input, VariableOutput};

pub struct Matcher {
    req: Vec<u8>,
    mask: Vec<u8>,
    prefix_len: usize,
}

impl Matcher {
    pub fn new(mut req: Vec<u8>, mut mask: Vec<u8>) -> Matcher {
        for (r, m) in req.iter_mut().zip(mask.iter_mut()) {
            *r = *r & *m;
        }
        let prefix_len = mask.iter()
            .enumerate()
            .rev()
            .find(|&(_i, &m)| m != 0)
            .map(|(i, _m)| i + 1)
            .unwrap_or(0);
        assert!(prefix_len <= 37);
        req.truncate(prefix_len);
        mask.truncate(prefix_len);
        assert!(req.len() >= prefix_len);
        assert!(mask.len() >= prefix_len);
        Matcher {
            req: req,
            mask: mask,
            prefix_len,
        }
    }

    pub fn req(&self) -> &[u8] {
        &self.req
    }

    pub fn mask(&self) -> &[u8] {
        &self.mask
    }

    pub fn prefix_len(&self) -> usize {
        self.prefix_len
    }

    pub fn matches(&self, pubkey: &[u8; 32]) -> bool {
        for i in 0..cmp::min(self.prefix_len, 32) {
            if pubkey[i] & self.mask[i] != self.req[i] {
                return false;
            }
        }
        if self.prefix_len > 32 {
            let mut checksum = [0u8; 5];
            let mut hasher = Blake2b::new(checksum.len()).unwrap();
            hasher.process(pubkey as &[u8]);
            hasher.variable_result(&mut checksum as &mut [u8]).unwrap();
            for i in 32..self.prefix_len {
                if checksum[4 - (i - 32)] & self.mask[i] != self.req[i] {
                    return false;
                }
            }
        }
        true
    }

    pub fn estimated_attempts(&self) -> usize {
        let mut bits_in_mask = 0;
        for byte in &self.mask {
            bits_in_mask += byte.count_ones() as usize;
        }
        1 << bits_in_mask
    }
}
