use prost::Message;
use cosmwasm_std::{AnyMsg, Binary, CosmosMsg};


#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(string, tag = "2")]
    pub amount: String,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTokenizeShares {
    #[prost(string, tag = "1")]
    pub delegator_address: String,
    #[prost(string, tag = "2")]
    pub validator_address: String,
    #[prost(message, optional, tag = "3")]
    pub amount: Option<Coin>,
    #[prost(string, tag = "4")]
    pub tokenized_share_owner: String,
}

impl MsgTokenizeShares {
    pub fn to_cosmos_msg(&self) -> CosmosMsg {
        let mut buf = Vec::new();
        buf.reserve(self.encoded_len());
        self.encode(&mut buf).unwrap();
        CosmosMsg::Any(AnyMsg {
            type_url: "/cosmos.staking.v1beta1.MsgTokenizeShares".to_string(),
            value: Binary::from(buf),
        })
    }
}

