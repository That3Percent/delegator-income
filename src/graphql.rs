use lazy_static::lazy_static;
use reqwest::blocking::Client;
use semaphore::Semaphore;
use serde::{de::DeserializeOwned, Deserialize};
use std::{collections::HashMap, thread::sleep};
use std::{error::Error, fs, time::Duration};

const REQUEST_CONCURRENCY: usize = 12;

lazy_static! {
    static ref CLIENT: Semaphore<Client> = Semaphore::new(REQUEST_CONCURRENCY, Client::new());
}

#[derive(Debug, Deserialize)]
struct Response<T> {
    data: T,
}

fn with_client<F: FnOnce(&Client) -> O, O>(f: F) -> O {
    loop {
        match CLIENT.try_access() {
            Ok(guard) => return f(&guard),
            Err(_) => sleep(Duration::from_millis(100)),
        }
    }
}

fn retry<F: Fn() -> Result<O, E>, E: std::fmt::Display + std::fmt::Debug, O>(f: F) -> O {
    for i in 0..10 {
        let v = f();
        if let Ok(v) = v {
            return v;
        }
        print!(".");
        sleep(Duration::from_secs(i));
    }
    f().unwrap()
}

fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, serde_json::Error> {
    serde_json::from_slice::<Response<T>>(&bytes).map(|r| r.data)
}

fn graphql_query_bytes(graphql: &str, subgraph: &str) -> Vec<u8> {
    let mut json = HashMap::new();
    json.insert("query", graphql);
    let url = format!("https://api.thegraph.com/subgraphs/name/{}", subgraph);
    retry::<_, Box<dyn Error>, _>(|| {
        with_client(|client| {
            let response = client.post(&url).json(&json).send()?;
            let bytes = response.bytes()?;
            Ok(bytes.to_vec())
        })
    })
}

pub fn cached_graphql_query<T>(graphql: &str, subgraph_name: &str) -> T
where
    T: DeserializeOwned,
{
    let mut hasher = blake3::Hasher::new();
    hasher.update(subgraph_name.as_bytes());
    hasher.update(graphql.as_bytes());
    let hash = hasher.finalize();
    let hash = hash.to_hex();
    let hash = &hash[0..24];
    let path = format!("./cache/{}.json", hash);

    match fs::read(&path) {
        Ok(s) => deserialize(&s).unwrap(),
        Err(_) => {
            let (response, deser) = retry(|| {
                let response = graphql_query_bytes(graphql, subgraph_name);
                // Validate before caching.
                // This prevents intermittent issues from
                // being cached.
                deserialize(&response).map(|d| (response, d))
            });
            fs::write(&path, &response).unwrap();
            deser
        }
    }
}

pub fn graphql_query<T>(graphql: &str, subgraph: &str) -> T
where
    T: DeserializeOwned,
{
    retry(|| deserialize(&graphql_query_bytes(graphql, subgraph)))
}
