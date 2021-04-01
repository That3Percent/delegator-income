// Rewards Output:
// block, date, reward amount,

mod data_collection;
mod diff;
mod graphql;
mod grt;
mod queries;
mod task;

fn main() {
    // Create the cache dir.
    let _ = std::fs::create_dir(format!("./cache"));

    // TODO: Get from cmd line
    let delegator_ids = Box::leak(Box::new([]));

    let changes = data_collection::get_changes(&delegator_ids[..]);
    dbg!(changes);
}
