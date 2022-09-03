extern crate modinverse;
extern crate num_bigint;
extern crate rsa;
use num_bigint::{BigInt, BigUint, RandBigInt, ToBigInt};
use num_traits::{One, ToPrimitive, Zero};
use rsa::{PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use std::ops::{Add, Sub};

/**
Struct for Integer Close to HRPPH generator.

t: threshold used for judging if the input of hash is within [-t,t]
d: small modulus for enumerating potential items
s: half of enumerating test cases, it is less than 100
a: random number for randomizing the collision resistant hash value
n: big modulus for collision resistant hash
*/
#[derive(Debug)]
pub struct HRPPHICT {
    t: u16,
    d: u16,
    s: u16,
    a: BigUint,
    n: BigUint,
}

/**
Struct for hash value of Integer Close To HRPPH.

r: small remains for enumerating potential items
d: small modulus for enumerating potential items
d: collision resistant hash value
n: big modulus for collision resistant hash
*/
#[derive(Debug, PartialEq, Eq)]
pub struct Hash {
    r: u16,
    g: BigUint,
    d: u16,
    n: BigUint,
}

impl HRPPHICT {
    /**
    Given a threshold and the bits of key, return a HRPPHICT generator.

    Collision resistant hash is generated from RSA library.
    The enumerating process is limited to no more than 200 rounds.
    */
    pub fn new(threshold: u16, lambda: u64) -> HRPPHICT {
        let d = if threshold <= 100 {
            threshold
        } else {
            threshold / 100
        };

        let mut rng = rand::thread_rng();
        let priv_key =
            RsaPrivateKey::new(&mut rng, lambda as usize).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        let mut a = rng.gen_biguint(lambda as u64);
        let module = BigUint::from_bytes_le(&(pub_key.n().to_bytes_le()));
        a = a % &module;

        HRPPHICT {
            t: threshold,
            d: d,
            s: threshold / d,
            a: a.clone(),
            n: module.clone(),
        }
    }

    pub fn hash(&self, x: &BigInt) -> Hash {
        let c = if *x >= BigInt::zero() {
            self.a.modpow(&(x.to_biguint().unwrap()), &(self.n))
        } else {
            let i = self.a.modpow(&((-x).to_biguint().unwrap()), &(self.n));
            modinverse(&i, &(self.n)).unwrap()
        };
        let new_r = x % self.d;
        let positive_new_r = if new_r < BigInt::zero() {
            new_r + self.d
        } else {
            new_r
        };
        let u16_r = positive_new_r.to_u16().unwrap();
        assert_eq!(BigInt::from(u16_r), positive_new_r);
        Hash {
            g: c.clone(),
            r: u16_r,
            d: self.d,
            n: self.n.clone(),
        }
    }

    pub fn eval(&self, h: &Hash) -> (Option<i32>, bool) {
        let step: i32 = self.d.into();
        let top: i32 = self.t.into();
        let bottom: i32 = -top;

        let mut c: i32 = (self.s * self.d) as i32 + h.r as i32;
        if c > top {
            c -= step;
        };
        while c > bottom {
            if self.eqcheck(c, &h.g) {
                return (Some(c), true);
            }
            c -= step;
        }
        return (None, false);
    }

    pub fn n(&self) -> BigUint {
        self.n.clone()
    }

    // Check if the candidate match the input
    fn eqcheck(&self, x: i32, y: &BigUint) -> bool {
        let h = if x >= 0 {
            self.a.modpow(&BigUint::from(x as u32), &(self.n))
        } else {
            let i = self.a.modpow(&BigUint::from((-x) as u32), &self.n);
            modinverse(&i, &self.n).unwrap()
        };
        return h == *y;
    }
}

impl Add for Hash {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        assert_eq!(&self.d, &other.d);
        assert_eq!(&self.n, &other.n);

        Self {
            r: (self.r + other.r) % self.d,
            g: (self.g * other.g) % &(self.n),
            d: self.d,
            n: self.n.clone(),
        }
    }
}

impl Sub for Hash {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self + other.inverse()
    }
}

impl Hash {
    fn inverse(&self) -> Hash {
        Self {
            r: self.d - self.r,
            g: modinverse(&(self.g), &(self.n)).unwrap(),
            d: self.d,
            n: self.n.clone(),
        }
    }
}

fn egcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if *a == BigInt::zero() {
        ((*b).clone(), BigInt::zero(), BigInt::one())
    } else {
        let (g, x, y) = egcd(&(b % a), &a);
        (g, y - (b / a) * (x.clone()), x)
    }
}

fn modinverse(a: &BigUint, m: &BigUint) -> Option<BigUint> {
    let (g, x, _) = egcd(&((*a).to_bigint().unwrap()), &((*m).to_bigint().unwrap()));
    if g != BigInt::one() {
        None
    } else {
        let im = &(m.to_bigint().unwrap());
        ((x % im + im) % im).to_biguint()
    }
}

#[test]
fn mod_inverse_test() {
    let mut bits = 128;
    let mut rng = rand::thread_rng();
    while bits < 5000 {
        let a = rng.gen_biguint(bits as u64);
        let generator = HRPPHICT::new(128, bits);
        let n = generator.n();
        let y = modinverse(&a, &n).unwrap();
        assert_eq!((y * a) % n, BigUint::one());
        bits *= 2;
    }
}
