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

    let mut epoches =
        graphql_query::<EpochData>(&query, "graphprotocol/graph-network-mainnet").epoches;
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

    cached_graphql_query::<DelegatorsData>(&query, "graphprotocol/graph-network-mainnet").delegators
}

pub fn pair(block_number: u64) -> Pair {
    let query = format!(
        r#"{{
            pair(id: "0xdfa42ba0130425b21a1568507b084cc246fb0c8f", block: {{ number: {} }}) {{
                token0Price
            }}
        }}"#,
        block_number
    );

    cached_graphql_query::<PairData>(&query, "uniswap/uniswap-v2").pair
}

#[derive(Debug, Deserialize)]
pub struct Pair {
    pub token0Price: String,
}

#[derive(Debug, Deserialize)]
struct DelegatorData {
    delegators: Option<Delegator>,
}

#[derive(Debug, Deserialize)]
struct DelegatorsData {
    delegators: Vec<Delegator>,
}

#[derive(Debug, Deserialize)]
struct PairData {
    pair: Pair,
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
    pub fn current_value(&self) -> GRT {
        let exchange_rate: BigDecimal = self.indexer.delegation_exchange_rate.parse().unwrap();
        let share_amount: GRT = self.share_amount.parse().unwrap();
        share_amount * exchange_rate
    }
    pub fn gains(&self) -> GRT {
        let value = self.current_value();
        let staked = self.staked_grt();
        value - staked
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

#[derive(Clone)]
pub struct DelegationTask {
    pub block_number: u64,
    pub delegator_ids: &'static [&'static str],
    pub pair: Task<Pair>,
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
            pair: PairsTask::create(block_number),
        };
        Task::new(task)
    }
}

pub type CachedDelegation = Task<(DelegationTask, Vec<Delegator>)>;

#[derive(Clone, Copy)]
pub struct PairsTask {
    pub block_number: u64,
}

impl TaskSource for PairsTask {
    type Output = Pair;

    fn execute(&self) -> Self::Output {
        pair(self.block_number)
    }
}

impl PairsTask {
    pub fn create(block_number: u64) -> Task<Pair> {
        let task = PairsTask { block_number };
        Task::new(task)
    }
}
