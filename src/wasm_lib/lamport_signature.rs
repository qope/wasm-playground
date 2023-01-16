use anyhow::Result;

use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Sample},
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        config::{AlgebraicHasher, Hasher, PoseidonGoldilocksConfig},
    },
};
use rand::RngCore;
use wasm_bindgen::prelude::*;

pub fn verify<F: RichField, H: Hasher<F>>(
    c: &H::Hash,
    images: &Vec<H::Hash>,
    selector: &Vec<bool>,
    pre_images: &Vec<Vec<F>>,
) -> Result<(), String> {
    assert!(images.len() / 2 == selector.len());
    assert!(selector.len() == pre_images.len());
    if !c.eq(&commit::<F, H>(&images)) {
        return Err("commit is not correct".to_string());
    }
    let selected_images = select::<F, H>(&selector, &images);
    for i in 0..pre_images.len() {
        let hashed_pre_image = H::hash_no_pad(&pre_images[i]);
        if !hashed_pre_image.eq(&selected_images[i]) {
            return Err("pre-image is not correct".to_string());
        }
    }
    Ok(())
}

pub fn make_verifier<H, F, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    c: HashOutTarget,
    images: Vec<HashOutTarget>,
    selector: Vec<BoolTarget>,
    pre_images: Vec<Vec<Target>>,
) where
    F: RichField + Extendable<D>,
    H: Hasher<F> + AlgebraicHasher<F>,
{
    assert!(images.len() / 2 == selector.len());
    assert!(selector.len() == pre_images.len());
    let c_cal = commit_t::<H, F, D>(builder, images.clone());
    builder.connect_hashes(c, c_cal);
    let selected_images = select_t::<F, H, D>(builder, selector, images);
    for i in 0..pre_images.len() {
        let hashed_pre_image = builder.hash_n_to_hash_no_pad::<H>(pre_images[i].clone());
        builder.connect_hashes(hashed_pre_image, selected_images[i]);
    }
}

pub fn commit<F, H>(images: &Vec<H::Hash>) -> H::Hash
where
    F: RichField,
    H: Hasher<F>,
{
    let n = log2(images.len());
    let mut images = images.clone();
    for _ in 0..n {
        let mut new_images = vec![];
        for i in 0..images.len() / 2 {
            new_images.push(H::two_to_one(images[2 * i], images[2 * i + 1]));
        }
        images = new_images;
    }
    assert_eq!(images.len(), 1);
    images[0]
}

pub fn commit_t<H, F, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    images: Vec<HashOutTarget>,
) -> HashOutTarget
where
    F: RichField + Extendable<D>,
    H: Hasher<F> + AlgebraicHasher<F>,
{
    let n = log2(images.len());
    let mut images = images;
    for _ in 0..n {
        let mut new_images = vec![];
        for i in 0..images.len() / 2 {
            new_images.push(builder.hash_n_to_hash_no_pad::<H>(
                [images[2 * i].elements, images[2 * i + 1].elements].concat(),
            ));
        }
        images = new_images;
    }
    assert_eq!(images.len(), 1);
    images[0]
}

pub fn select<F: RichField, H: Hasher<F>>(
    selector: &Vec<bool>,
    images: &Vec<H::Hash>,
) -> Vec<H::Hash> {
    assert_eq!(images.len() / 2, selector.len());
    let mut selected_images = vec![];
    for i in 0..selector.len() {
        selected_images.push(if selector[i] {
            images[2 * i + 1]
        } else {
            images[2 * i]
        })
    }
    return selected_images;
}

pub fn select_pre_images<F: RichField, H: Hasher<F>>(
    selector: &Vec<bool>,
    pre_images: &Vec<Vec<F>>,
) -> Vec<Vec<F>> {
    assert_eq!(pre_images.len() / 2, selector.len());
    let mut selected_images = vec![];
    for i in 0..selector.len() {
        selected_images.push(if selector[i] {
            pre_images[2 * i + 1].clone()
        } else {
            pre_images[2 * i].clone()
        })
    }
    return selected_images;
}

pub fn select_t<F, H, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    selector: Vec<BoolTarget>,
    images: Vec<HashOutTarget>,
) -> Vec<HashOutTarget>
where
    F: RichField + Extendable<D>,
    H: Hasher<F> + AlgebraicHasher<F>,
{
    assert_eq!(images.len() / 2, selector.len());
    let mut selected_images = vec![];
    for i in 0..selector.len() {
        let t: Vec<Target> = (0..4)
            .map(|j| {
                builder.select(
                    selector[i],
                    images[2 * i + 1].elements[j],
                    images[2 * i].elements[j],
                )
            })
            .collect();
        let hash_t = HashOutTarget {
            elements: [t[0], t[1], t[2], t[3]],
        };
        selected_images.push(hash_t);
    }
    return selected_images;
}

fn log2(n: usize) -> usize {
    let mut n = n;
    let mut p = 0;
    while (n & 1) == 0 {
        n >>= 1;
        p += 1;
    }
    assert_eq!(n, 1, "is not pow of two");
    return p;
}

#[wasm_bindgen]
pub async fn make_lamport_sig() -> String {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;
    type H = PoseidonHash;
    let mut rng = rand::thread_rng();
    let n = 9;
    let all_pre_images: Vec<Vec<F>> = (0..1 << n).map(|_| F::rand_vec(4)).collect();
    let images: Vec<HashOut<F>> = all_pre_images.iter().map(|x| H::hash_no_pad(x)).collect();
    let selector: Vec<bool> = (0..1 << n - 1).map(|_| rng.next_u32() % 2 == 0).collect();
    let pre_images = select_pre_images::<F, H>(&selector, &all_pre_images);
    let c = commit::<F, H>(&images);

    verify::<F, H>(&c, &images, &selector, &pre_images).unwrap();
    "success".to_string()
}

#[cfg(test)]
mod tests {

    use super::*;

    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Sample},
        hash::{hash_types::HashOut, poseidon::PoseidonHash},
        iop::{
            target::{BoolTarget, Target},
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{Hasher, PoseidonGoldilocksConfig},
        },
    };
    use rand::RngCore;
    use std::time::Instant;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;
    type H = PoseidonHash;

    #[test]
    fn test_commit() {
        let n = 9;
        let image: Vec<HashOut<F>> = (0..1 << n).map(|_| HashOut::ZERO).collect();
        dbg!(commit::<F, H>(&image));
    }

    #[test]
    fn test_pow2() {
        let mut n = 8;
        let mut p = 0;
        while (n & 1) == 0 {
            n >>= 1;
            p += 1;
        }
        assert_eq!(n, 1, "is not pow of two");
        dbg!(p);
    }

    #[test]
    fn test_commit_circuit() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config.clone());
        let mut pw = PartialWitness::<F>::new();

        let n = 9;
        let image: Vec<HashOut<F>> = (0..1 << n).map(|_| HashOut::ZERO).collect();
        let c = commit::<F, H>(&image);
        let image_t = builder.add_virtual_hashes(1 << n);
        for i in 0..1 << n {
            pw.set_hash_target(image_t[i], image[i]);
        }
        let c_t = commit_t::<H, F, D>(&mut builder, image_t);
        pw.set_hash_target(c_t, c);
        let data = builder.build::<C>();
        let _proof = data.prove(pw);
    }

    #[test]
    fn test_select() {
        let mut rng = rand::thread_rng();
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config.clone());
        let mut pw = PartialWitness::<F>::new();
        let n = 9;
        let images: Vec<HashOut<F>> = (0..1 << n).map(|_| HashOut::ZERO).collect();
        let selector: Vec<bool> = (0..1 << n - 1).map(|_| rng.next_u32() % 2 == 0).collect();
        let selected_images = select::<F, H>(&selector, &images);

        let selector_t: Vec<BoolTarget> = (0..1 << n - 1)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect();
        let images_t = builder.add_virtual_hashes(1 << n);
        let selected_images_t =
            select_t::<F, H, D>(&mut builder, selector_t.clone(), images_t.clone());

        for i in 0..images.len() {
            pw.set_hash_target(images_t[i], images[i]);
        }
        for i in 0..selector.len() {
            pw.set_bool_target(selector_t[i], selector[i]);
        }
        for i in 0..selected_images.len() {
            pw.set_hash_target(selected_images_t[i], selected_images[i]);
        }
        let data = builder.build::<C>();
        let _proof = data.prove(pw);
    }

    #[test]
    fn test_verify() -> Result<(), String> {
        let mut rng = rand::thread_rng();
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config.clone());
        let mut pw = PartialWitness::<F>::new();

        let n = 9;
        let all_pre_images: Vec<Vec<F>> = (0..1 << n).map(|_| F::rand_vec(4)).collect();
        let images: Vec<HashOut<F>> = all_pre_images.iter().map(|x| H::hash_no_pad(x)).collect();
        let selector: Vec<bool> = (0..1 << n - 1).map(|_| rng.next_u32() % 2 == 0).collect();
        let pre_images = select_pre_images::<F, H>(&selector, &all_pre_images);
        let c = commit::<F, H>(&images);

        verify::<F, H>(&c, &images, &selector, &pre_images)?;

        let c_t = builder.add_virtual_hash();
        let images_t = builder.add_virtual_hashes(1 << n);
        let selector_t: Vec<BoolTarget> = (0..1 << n - 1)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect();
        let mut pre_images_t: Vec<Vec<Target>> = vec![];
        for _ in 0..pre_images.len() {
            let pre_image: Vec<Target> = (0..4).map(|_| builder.add_virtual_target()).collect();
            pre_images_t.push(pre_image);
        }

        make_verifier::<H, F, D>(
            &mut builder,
            c_t,
            images_t.clone(),
            selector_t.clone(),
            pre_images_t.clone(),
        );

        pw.set_hash_target(c_t, c);

        for i in 0..images.len() {
            pw.set_hash_target(images_t[i], images[i]);
        }
        for i in 0..selector.len() {
            pw.set_bool_target(selector_t[i], selector[i]);
        }
        for i in 0..pre_images.len() {
            for j in 0..4 {
                pw.set_target(pre_images_t[i][j], pre_images[i][j]);
            }
        }

        let data = builder.build::<C>();

        let now = Instant::now();
        match data.prove(pw) {
            Ok(_) => Ok(()),
            _ => Err("prove failed"),
        }?;
        dbg!(now.elapsed().as_millis());
        dbg!(data.common.degree_bits());
        Ok(())
    }
}
