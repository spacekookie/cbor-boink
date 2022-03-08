use minicbor::{bytes::ByteVec, Decode, Encode};
use rand::distributions::{Alphanumeric, Standard};
use rand::{rngs::ThreadRng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Debug, Clone, Encode, Decode)]
struct TransportMessageCbor {
    #[n(0)]
    version: u8,
    #[n(1)]
    onward_route: Vec<minicbor::bytes::ByteVec>,
    #[n(2)]
    return_route: Vec<minicbor::bytes::ByteVec>,
    #[n(3)]
    #[cbor(with = "minicbor::bytes")]
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransportMessageSerde {
    version: u8,
    onward_route: Vec<serde_bytes::ByteBuf>,
    return_route: Vec<serde_bytes::ByteBuf>,
    #[serde(with = "serde_bytes")]
    payload: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inner_bare = serde_bare::to_vec(&TransportMessageSerde {
        version: 0,
        // `serde_bytes::ByteBuf` doesn't implement `From`, and just has a
        // constructor function named `from` :/
        onward_route: vec![serde_bytes::ByteBuf::from("0#app".as_bytes())],
        return_route: vec![],
        payload: serde_bare::to_vec(&String::from("Hello ockam"))?,
    })?;

    let inner_cbor = minicbor::to_vec(&TransportMessageCbor {
        version: 0,
        onward_route: vec!["0#app".as_bytes().to_vec().into()],
        return_route: vec![],
        payload: minicbor::to_vec(&String::from("Hello ockam"))?,
    })?;

    let len_inner_bare = inner_bare.len();
    let len_inner_cbor = inner_cbor.len();

    let outer_bare = serde_bare::to_vec(&TransportMessageSerde {
        version: 0,
        onward_route: vec![
            serde_bytes::ByteBuf::from("1#hub.ockam.network".as_bytes().to_vec()),
            serde_bytes::ByteBuf::from("0#my-pipe-friend".as_bytes().to_vec()),
        ],
        return_route: vec![],
        payload: inner_bare,
    })?;

    let outer_cbor = minicbor::to_vec(&TransportMessageCbor {
        version: 0,
        onward_route: vec![
            "1#hub.ockam.network".as_bytes().to_vec().into(),
            "0#my-pipe-friend".as_bytes().to_vec().into(),
        ],
        return_route: vec![],
        payload: inner_cbor,
    })?;

    let len_outer_bare = outer_bare.len();
    let len_outer_cbor = outer_cbor.len();

    let outest_bare = serde_bare::to_vec(&TransportMessageSerde {
        version: 0,
        onward_route: vec![
            serde_bytes::ByteBuf::from("1#hub.ockam.network".as_bytes().to_vec()),
            serde_bytes::ByteBuf::from("0#my-pipe-friend".as_bytes().to_vec()),
        ],
        return_route: vec![],
        payload: outer_bare,
    })?;

    let outest_cbor = minicbor::to_vec(&TransportMessageCbor {
        version: 0,
        onward_route: vec![
            "1#hub.ockam.network".as_bytes().to_vec().into(),
            "0#my-pipe-friend".as_bytes().to_vec().into(),
        ],
        return_route: vec![],
        payload: outer_cbor,
    })?;

    let len_outest_bare = outest_bare.len();
    let len_outest_cbor = outest_cbor.len();

    let cbor_inner_adds = (len_inner_cbor as f64 / len_inner_bare as f64) * 100.0;
    let cbor_outer_adds = (len_outer_cbor as f64 / len_outer_bare as f64) * 100.0;
    let cbor_outest_adds = (len_outest_cbor as f64 / len_outest_bare as f64) * 100.0;

    println! {
        ">>> Inner message\n\
         BARE = {len_inner_bare:4} bytes (100.00%)\n\
         CBOR = {len_inner_cbor:4} bytes ({cbor_inner_adds:6.2}%)\
    \n\n\
         >>> Outer message\n\
         BARE = {len_outer_bare:4} bytes (100.00%)\n\
         CBOR = {len_outer_cbor:4} bytes ({cbor_outer_adds:6.2}%)\
    \n\n\
         >>> Outest message\n\
         BARE = {len_outest_bare:4} bytes (100.00%)\n\
         CBOR = {len_outest_cbor:4} bytes ({cbor_outest_adds:6.2}%)"
    };

    Ok(())
}
