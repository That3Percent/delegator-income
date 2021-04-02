use crate::task::*;
use crate::{graphql::*, grt::GRT};
use bigdecimal::BigDecimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Epoch {
    pub start_block: u64,
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpochData {
    epoches: Vec<Epoch>,
}

pub fn epochs() -> Vec<Epoch> {
    let query = format!(
        r#"{{
            epoches(first: 1000) {{
                startBlock
                id
            }}
        }}"#,
    );

    let mut epoches = graphql_query::<EpochData>(&query).epoches;
    epoches.sort_by_key(|e| {
        let v: u32 = e.id.parse().unwrap();
        v
    });
    epoches
}

pub fn delegators(ids: &[&str], block_number: u64) -> Vec<Delegator> {
    let query = format!(
        r#"{{
            delegators(where: {{id_in: {:?}}}, block: {{ number: {} }}) {{
                id
                stakes {{
                    id
                    shareAmount
                    stakedTokens
                    personalExchangeRate
                    indexer {{
                        id
                        delegatorShares
                        delegatedTokens
                        delegationExchangeRate
                    }}
                }}
                totalRealizedRewards
            }}
        }}"#,
        ids, block_number
    );

    cached_graphql_query::<DelegatorsData>(&query).delegators
}

#[derive(Debug, Deserialize)]
struct DelegatorData {
    delegators: Option<Delegator>,
}

#[derive(Debug, Deserialize)]
struct DelegatorsData {
    delegators: Vec<Delegator>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Delegator {
    pub stakes: Vec<Stake>,
    pub id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stake {
    pub share_amount: String,
    pub personal_exchange_rate: String,
    pub staked_tokens: String,
    pub indexer: Indexer,
    pub id: String,
}

impl Stake {
    pub fn staked_grt(&self) -> GRT {
        self.staked_tokens.parse().unwrap()
    }
    pub fn burned_grt(&self) -> GRT {
        // total * 0.995 = stake.staked_tokens
        let staked = self.staked_grt();
        let non_burned: BigDecimal = "0.995".parse().unwrap();
        let total = staked.clone() / non_burned;
        total - staked
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Indexer {
    pub id: String,
    pub delegator_shares: String,
    pub delegated_tokens: String,
    pub delegation_exchange_rate: String,
}

#[derive(Copy, Clone)]
pub struct DelegationTask {
    pub block_number: u64,
    pub delegator_ids: &'static [&'static str],
}

impl TaskSource for DelegationTask {
    type Output = (Self, Vec<Delegator>);

    fn execute(&self) -> Self::Output {
        (
            self.clone(),
            delegators(&self.delegator_ids, self.block_number),
        )
    }
}

impl DelegationTask {
    pub fn create(block_number: u64, delegator_ids: &'static [&'static str]) -> CachedDelegation {
        let task = DelegationTask {
            block_number,
            delegator_ids,
        };
        Task::new(task)
    }
}

pub type CachedDelegation = Task<(DelegationTask, Vec<Delegator>)>;
