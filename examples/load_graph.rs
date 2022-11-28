// process the graph from https://snap.stanford.edu/data/twitter-2010.html
// time cat /Users/tristan/Downloads/twitter-2010.txt.gz | gunzip | cargo run --release --example load_graph

use bytemuck::cast_slice;
use std::{
    fs::File,
    io::{self, BufRead, Write},
};

use twitterperf::data::*;

const TEST: bool = true;

fn main() {
    let num_users = 41652230;
    let mut graph: Vec<Vec<UserIdx>> = (0..num_users).map(|_| vec![]).collect();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut total_follows = 0;
    while handle.read_line(&mut buffer).unwrap() != 0 {
        let mut split = buffer.split(" ");
        // An edge from i to j indicates that j is a follower of i
        let i: u32 = split.next().unwrap().trim().parse().unwrap();
        let j: u32 = split.next().unwrap().trim().parse().unwrap();
        graph[j as usize].push(i);
        buffer.clear();
        total_follows += 1;
    }

    eprintln!("Done phase 1");

    let mut users: Vec<User> = Vec::with_capacity(num_users);
    let mut follows: Vec<UserIdx> = Vec::with_capacity(total_follows);

    for ls in &graph {
        let user = User {
            follows_idx: follows.len(),
            num_follows: ls.len() as u32,
            num_followers: 0,
        };
        users.push(user);
        for x in ls {
            follows.push(*x);
        }
    }

    eprintln!("Done phase 2");

    for f in &follows {
        users[*f as usize].num_followers += 1;
    }

    eprintln!("Done phase 3");

    if TEST {
        return;
    }

    let mut users_f = File::create("data/users.bin").unwrap();
    let user_bytes = cast_slice(&users[..]);
    users_f.write_all(user_bytes).unwrap();

    let mut follows_f = File::create("data/follows.bin").unwrap();
    let follows_bytes = cast_slice(&follows[..]);
    follows_f.write_all(follows_bytes).unwrap();
}
