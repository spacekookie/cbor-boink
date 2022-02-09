use minicbor::{Encode, Decode};
use rand::{Rng, RngCore, rngs::ThreadRng};
use rand::distributions::{Alphanumeric, Standard};
use serde::{Serialize, Deserialize};
use std::iter;

/// Limits for sample size generation
const TINY: usize = 8;
const SMALL: usize = 32;
const MEDIUM: usize = 128;
const LARGE: usize = 512;

/// Our example type we are going to serialise;
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
struct Sample {
    #[n(0)] field_a: u32,
    #[n(1)] field_b: u64,
    #[n(2)] field_c: String,
    #[n(3)] field_d: SomeEnum,
    #[n(4)] field_e: SomeOtherStruct,
    #[n(5)] field_f: Vec<u16>,
    #[cbor(n(6), with = "minicbor::bytes")] field_g: Vec<u8>
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
enum SomeEnum {
    #[n(0)] One(#[n(0)] u8),
    #[n(1)] Two(#[n(0)] u16),
    #[n(2)] Three(#[n(0)] u64),
    #[n(3)] Four(#[n(0)] String),
    #[n(4)] Five(#[n(0)] Vec<Sample>)
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
struct SomeOtherStruct {
    #[n(0)] field_a: Option<bool>,
    #[n(1)] field_b: Vec<String>,
    #[n(2)] field_c: Vec<Sample>
}

/// Generate a random sample.
fn gen(rng: &mut ThreadRng, max: usize, rem: usize) -> Sample {
    fn gen_string(max: usize, rng: &mut impl Rng) -> String {
        let n = rng.gen_range(0 .. max);
        iter::repeat_with(|| char::from(rng.sample(Alphanumeric)))
            .take(n)
            .collect()
    }
    Sample {
        field_a: rng.gen(),
        field_b: rng.gen(),
        field_c: gen_string(max, rng),
        field_d: match rng.gen_range(0 .. 5) {
            0 => SomeEnum::One(rng.gen()),
            1 => SomeEnum::Two(rng.gen()),
            2 => SomeEnum::Three(rng.gen()),
            3 => SomeEnum::Four(gen_string(max, rng)),
            _ => SomeEnum::Five({
                let n = rng.gen_range(0 .. rem);
                iter::repeat_with(|| gen(rng, max, rem - 1)).take(n).collect()
            })
        },
        field_e: SomeOtherStruct {
            field_a: rng.gen(),
            field_b: {
                let n = rng.gen_range(0 .. max);
                iter::repeat_with(|| gen_string(max, rng)).take(n).collect()
            },
            field_c: {
                let n = rng.gen_range(0 .. rem);
                iter::repeat_with(|| gen(rng, max, rem - 1)).take(n).collect()
            }
        },
        field_f: {
            let n = rng.gen_range(0 .. max);
            rng.sample_iter(Standard).take(n).collect()
        },
        field_g: {
            let mut v = vec![0; rng.gen_range(0 .. max)];
            rng.fill_bytes(&mut v);
            v
        }
    }
}

/// A single encoding round.
fn encode(size: usize, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut g = rand::thread_rng();
    let samp = gen(&mut g, size, 7);
    let cbor = minicbor::to_vec(&samp)?.len();
    let bare = serde_bare::to_vec(&samp)?.len();
    let bare_to_cbor = (bare as f64 / cbor as f64) * 100.0;
    let cbor_to_bare = (cbor as f64 / bare as f64) * 100.0;
    println! {
        ">>> {label}\n\
         BARE = {bare} bytes ({bare_to_cbor:6.2}%)\n\
         CBOR = {cbor} bytes ({cbor_to_bare:6.2}%)"
    };
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    encode(TINY, "TINY")?;
    encode(SMALL, "SMALL")?;
    encode(MEDIUM, "MEDIUM")?;
    encode(LARGE, "LARGE")
}
