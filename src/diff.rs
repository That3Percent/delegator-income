use crate::grt::*;
use crate::queries::*;
use bigdecimal::BigDecimal;
use single::Single as _;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct BlockDiff {
    pub block: u64,
    pub amount: GRT,
    pub dollars: BigDecimal,
}

fn push_block_diff(
    changes: &Arc<Mutex<Changes>>,
    property: impl Fn(&mut Changes) -> &mut Vec<BlockDiff>,
    block: &DelegationTask,
    amount: GRT,
) {
    // At the time of writing even at the all-time high value of GRT
    // this would be a fraction of a penny. Filtering here removes reporting
    // of some silly things like negative gains due to rounding when tokens
    // were burned, or collections of miniscule query fees.
    // This is 0.0001 GRT
    let min: GRT = "100000000000000".parse().unwrap();
    if amount < min {
        return;
    }

    let exchange_rate: BigDecimal = block.pair.get().token0Price.parse().unwrap();

    let diff = BlockDiff {
        block: block.block_number,
        amount: amount.clone(),
        dollars: amount.0 * exchange_rate,
    };
    let mut changes = changes.lock().unwrap();
    let collection = property(&mut changes);
    collection.push(diff);
}

#[derive(Debug, Default)]
pub struct Changes {
    pub rewards: Vec<BlockDiff>,
    pub burns: Vec<BlockDiff>,
}

pub fn diff_delegators(
    before: &[Delegator],
    after: &[Delegator],
    block: &DelegationTask,
    changes: &Arc<Mutex<Changes>>,
) {
    diff_set_by_id(
        before,
        after,
        |b, a| b.id == a.id,
        |d| add_delegator(d, block, changes),
        |d| remove_delegator(d, block, changes),
        |b, a| diff_delegator(b, a, block, changes),
    );
}

fn diff_delegator(
    before: &Delegator,
    after: &Delegator,
    block: &DelegationTask,
    changes: &Arc<Mutex<Changes>>,
) {
    assert_eq!(&before.id, &after.id);

    diff_set_by_id(
        &before.stakes,
        &after.stakes,
        |b, a| &b.id == &a.id,
        |s| add_stake(s, block, changes),
        |s| remove_stake(s, block, changes),
        |b, a| diff_stake(b, a, block, changes),
    )
}

fn add_delegator(delegator: &Delegator, block: &DelegationTask, changes: &Arc<Mutex<Changes>>) {
    for stake in delegator.stakes.iter() {
        add_stake(stake, block, changes)
    }
}

fn remove_delegator(
    _delegator: &Delegator,
    _block: &DelegationTask,
    _changes: &Arc<Mutex<Changes>>,
) {
    todo!("remove delegator")
}

fn add_stake(stake: &Stake, block: &DelegationTask, changes: &Arc<Mutex<Changes>>) {
    push_block_diff(changes, |c| &mut c.burns, block, stake.burned_grt());
    push_block_diff(changes, |c| &mut c.rewards, block, stake.gains());
}

fn remove_stake(stake: &Stake, block: &DelegationTask, changes: &Arc<Mutex<Changes>>) {
    todo!()
}

fn diff_stake(
    before: &Stake,
    after: &Stake,
    block: &DelegationTask,
    changes: &Arc<Mutex<Changes>>,
) {
    push_block_diff(
        changes,
        |c| &mut c.burns,
        block,
        after.burned_grt() - before.burned_grt(),
    );

    push_block_diff(
        changes,
        |c| &mut c.rewards,
        block,
        after.gains() - before.gains(),
    );
}

fn diff_set_by_id<FA: Fn(&T), FR: Fn(&T), FD: Fn(&T, &T), T>(
    before: &[T],
    after: &[T],
    delta: impl Fn(&T, &T) -> bool,
    fn_add: FA,
    fn_rem: FR,
    fn_d: FD,
) {
    for b in before.iter() {
        match after.iter().filter(|a| delta(b, a)).single() {
            Ok(a) => fn_d(b, a),
            Err(single::Error::NoElements) => fn_rem(b),
            Err(single::Error::MultipleElements) => panic!("Did not expect multiple matches"),
        }
    }

    for a in after.iter() {
        match before.iter().filter(|b| delta(b, a)).single() {
            Ok(_) => {} // Already handled above
            Err(single::Error::NoElements) => fn_add(a),
            Err(single::Error::MultipleElements) => panic!("Did not expect multiple matches"),
        }
    }
}
