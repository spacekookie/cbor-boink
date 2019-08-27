use parity_scale_codec::Encode;
use parity_scale_codec_derive::{Encode, Decode};
use prost::Message;
use rand::prelude::*;

mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/protobuf.rs"));
}

/// Our example type we are going to serialise;
#[derive(Debug, Clone, Encode, Decode)]
pub struct Sample {
    field_a: u32,
    field_b: u64,
    field_c: String,
    field_d: SomeEnum,
    field_e: SomeOtherStruct,
    field_f: Vec<u8>
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum SomeEnum {
    One(u8),
    Two(u16),
    Three(u64),
    Four(String)
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SomeOtherStruct {
    field_a: Option<bool>,
    field_b: Vec<String>
}

/// Generate a random sample.
fn gen() -> Sample {
    fn gen_string(rng: &mut impl Rng) -> String {
        let n = rng.gen_range(0, 2048);
        rng.sample_iter(rand::distributions::Alphanumeric)
            .take(n)
            .collect()
    }

    let mut rng = rand::thread_rng();

    Sample {
        field_a: rng.gen(),
        field_b: rng.gen(),
        field_c: gen_string(&mut rng),
        field_d: match rng.gen_range(0, 4) {
            0 => SomeEnum::One(rng.gen()),
            1 => SomeEnum::Two(rng.gen()),
            2 => SomeEnum::Three(rng.gen()),
            _ => SomeEnum::Four(gen_string(&mut rng))
        },
        field_e: SomeOtherStruct {
            field_a: rng.gen(),
            field_b: {
                let n = rng.gen_range(0, 2048);
                let mut v = Vec::with_capacity(n);
                for _ in 0 .. n {
                    v.push(gen_string(&mut rng))
                }
                v
            }
        },
        field_f: {
            let n = rng.gen_range(0, 2048);
            rng.sample_iter(rand::distributions::Standard)
                .take(n)
                .collect()
        }
    }
}

#[derive(Debug)]
pub struct Lengths {
    scale: usize,
    cbor: usize,
    protobuf: usize
}

/// A single encoding round.
fn encode() -> Result<Lengths, Box<dyn std::error::Error>> {
    let x = gen();
    let s = x.encode();
    let c = {
        use miel::write::Encoder;
        let mut w = Vec::new();
        let mut e = Encoder::new(&mut w);
        e.object(6)?
            .u8(1)?.u32(x.field_a)?
            .u8(2)?.u64(x.field_b)?
            .u8(3)?.str(&x.field_c)?
            .u8(4)?;
        match &x.field_d {
            SomeEnum::One(v)   => e.u8(1)?.u8(*v)?,
            SomeEnum::Two(v)   => e.u8(2)?.u16(*v)?,
            SomeEnum::Three(v) => e.u8(3)?.u64(*v)?,
            SomeEnum::Four(v)  => e.u8(4)?.str(v)?
        };
        e.u8(5)?.object(2)?
            .u8(1)?.option(&x.field_e.field_a, |e, b| {
                e.bool(*b)?;
                Ok(())
            })?
            .u8(2)?.vector(&x.field_e.field_b, |e, s| {
                e.str(s)?;
                Ok(())
            })?
        .u8(6)?.bytes(&x.field_f)?;
        w
    };
    let p = {
        let s = protobuf::Sample {
            field_a: x.field_a,
            field_b: x.field_b,
            field_c: x.field_c,
            field_d: Some(protobuf::SomeEnum {
                some_enum: Some(match x.field_d {
                    SomeEnum::One(v) => protobuf::some_enum::SomeEnum::One(u32::from(v)),
                    SomeEnum::Two(v) => protobuf::some_enum::SomeEnum::Two(u32::from(v)),
                    SomeEnum::Three(v) => protobuf::some_enum::SomeEnum::Three(v),
                    SomeEnum::Four(v) => protobuf::some_enum::SomeEnum::Four(v)
                })
            }),
            field_e: Some(protobuf::SomeOtherStruct {
                field_a: x.field_e.field_a.unwrap_or(false),
                field_b: x.field_e.field_b
            }),
            field_f: x.field_f
        };
        let mut v = Vec::new();
        s.encode(&mut v)?;
        v
    };
    Ok(Lengths {
        scale: s.len(),
        cbor: c.len(),
        protobuf: p.len()
    })
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let x = encode()?;
    println!("SCALE = {} bytes\nCBOR  = {} bytes\nPROTO = {} bytes", x.scale, x.cbor, x.protobuf);
    Ok(())
}
