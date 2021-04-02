// Rewards Output:
// block, date, reward amount,

mod data_collection;
mod diff;
mod graphql;
mod grt;
mod queries;
mod task;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    delegator_ids: Vec<String>,
}

fn main() {
    // Read the delegators ids from the command-line
    let opt = Opt::from_args();
    let delegator_ids: Vec<&'static str> = opt
        .delegator_ids
        .iter()
        .map(|d| Box::leak(Box::new(d.clone().to_ascii_lowercase())).as_str())
        .collect();
    let delegator_ids = Box::leak(Box::new(delegator_ids));

    // Create the cache dir.
    // Future IO operations just assume this exists.
    let _ = std::fs::create_dir(format!("./cache"));

    let changes = data_collection::get_changes(&delegator_ids[..]);

    println!("Burns:");
    for burn in changes.burns.iter() {
        println!("  {}: {}", burn.block, burn.amount);
    }

    println!("Rewards: ");
    for reward in changes.rewards.iter() {
        println!("  {}: {}", reward.block, reward.amount);
    }
}
