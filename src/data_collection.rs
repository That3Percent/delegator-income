use crate::diff::*;
use crate::grt::GRT;
use crate::queries::*;
use rayon::{join, prelude::*};
use std::sync::{Arc, Mutex};

fn _get_changes(
    start_task: CachedDelegation,
    end_task: CachedDelegation,
    changes: &Arc<Mutex<Changes>>,
) -> bool {
    let (start, delegations_start) = start_task.get();
    let (end, delegations_end) = end_task.get();
    assert!(end.block_number > start.block_number);

    // If there is no diff, this range is uninteresting.
    if delegations_start == delegations_end {
        return false;
    }

    // If the block range is reduced to 1,
    // we have found the exact block this change occurs at.
    if end.block_number == start.block_number + 1 {
        diff_delegators(
            delegations_start,
            delegations_end,
            end.block_number,
            changes,
        );
        return true;
    }

    // There is a diff, but the range is large need to binary search.
    let mid = (end.block_number + start.block_number) / 2;
    let mid = DelegationTask::create(mid, end.delegator_ids);
    let (left, right) = join(
        || _get_changes(start_task, mid.clone(), changes),
        || _get_changes(mid.clone(), end_task, changes),
    );
    assert!(left || right);
    return true;
}

pub fn get_changes(delegator_ids: &'static [&'static str]) -> Changes {
    let epochs = epochs();

    let changes = Changes::default();
    let changes = Arc::new(Mutex::new(changes));

    // Performance: This searches within each epoch separately so
    // that the binary search hits the same block numbers across runs.

    let mut tasks = Vec::new();
    for epoch in epochs.into_iter() {
        tasks.push(DelegationTask::create(epoch.start_block, delegator_ids));
    }

    tasks.windows(2).for_each(|chunk| {
        let prev = &chunk[0];
        let next = &chunk[1];

        _get_changes(prev.clone(), next.clone(), &changes);
    });

    let mut lock = changes.lock().unwrap();
    std::mem::replace(&mut *lock, Default::default())
}
