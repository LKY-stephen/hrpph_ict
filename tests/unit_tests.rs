#[cfg(test)]
mod tests {
    use hrpph_ict::hashes::HRPPHICT;
    use num_bigint::{BigInt, BigUint, RandBigInt, ToBigInt};
    use rand::{rngs::ThreadRng, Rng};

    #[test]
    fn gen_hash_small_positive_input() {
        let mut bits = 128;
        while bits < 5000 {
            test_eval_small(bits, true, false);
            bits *= 2;
        }
    }

    #[test]
    fn gen_hash_small_negative_input() {
        let mut bits = 128;
        while bits < 5000 {
            test_eval_small(bits, false, false);
            bits *= 2;
        }
    }

    #[test]
    fn gen_hash_big_positive_input() {
        let mut bits = 128;
        while bits < 5000 {
            test_eval_small(bits, true, true);
            bits *= 2;
        }
    }

    #[test]
    fn gen_hash_big_negative_input() {
        let mut bits = 128;
        while bits < 5000 {
            test_eval_small(bits, false, true);
            bits *= 2;
        }
    }

    #[test]
    fn hash_homomorphic_add() {
        let mut bits = 128;
        let mut rng = rand::thread_rng();
        let t: u16 = rng.gen();
        while bits < 5000 {
            let generator = HRPPHICT::new(t.into(), bits);
            let pb = gen_input(&mut rng, t, true, true, bits);
            let ps = gen_input(&mut rng, t, false, true, bits);
            let nb = gen_input(&mut rng, t, true, false, bits);
            let ns = gen_input(&mut rng, t, false, false, bits);
            let array = &[pb, ps, nb, ns];
            for i in array {
                for j in array {
                    assert_eq!(
                        generator.hash(&(i + j)),
                        generator.hash(i) + generator.hash(j)
                    );
                }
            }
            bits *= 2;
        }
    }

    #[test]
    fn hash_homomorphic_sub() {
        let mut bits = 128;
        let mut rng = rand::thread_rng();
        let t: u16 = rng.gen();
        while bits < 5000 {
            let generator = HRPPHICT::new(t.into(), bits);
            let pb = gen_input(&mut rng, t, true, true, bits);
            let ps = gen_input(&mut rng, t, false, true, bits);
            let nb = gen_input(&mut rng, t, true, false, bits);
            let ns = gen_input(&mut rng, t, false, false, bits);
            let array = &[pb, ps, nb, ns];
            for i in array {
                for j in array {
                    assert_eq!(
                        generator.hash(&(i - j)),
                        generator.hash(i) - generator.hash(j)
                    );
                }
            }
            bits *= 2;
        }
    }
    /// Test cases with positive small inputs
    fn test_eval_small(lambda: u64, positive: bool, big: bool) {
        let mut rng = rand::thread_rng();
        let t: u16 = rng.gen();
        let generator = HRPPHICT::new(t.into(), lambda.into());
        let mut i = 0;
        while i < 10 {
            let input = gen_input(&mut rng, t, big, positive, lambda);
            let h = generator.hash(&input);
            let (y, result) = generator.eval(&h);
            if result == big {
                println!("inputs: {:?}", input);
                println!("hash: {:?}", h);
                println!("threshold: {:?}", t);
                panic!();
            }
            if !big {
                assert_eq!(BigInt::from(y.unwrap()), input);
            }
            i += 1;
        }
    }

    fn gen_input(rng: &mut ThreadRng, t: u16, big: bool, positive: bool, lambda: u64) -> BigInt {
        let positive_result = if big {
            let big_t = BigUint::from(t);
            let mut seed = rng.gen_biguint(lambda.into());
            if seed < big_t {
                seed += big_t
            }
            seed.to_bigint().unwrap()
        } else {
            let mut small_seed: u16 = rng.gen();
            small_seed = small_seed % t;
            BigInt::from(small_seed)
        };

        if positive {
            return positive_result;
        }
        return -positive_result;
    }
}
