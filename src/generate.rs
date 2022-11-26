use crate::data::*;

use bytemuck::cast_slice;
use memmap2::Mmap;
use std::fs::File;
use std::ops::Deref;

pub struct TweetGenerator {
    pub seed: u64,
}

impl Default for TweetGenerator {
    fn default() -> TweetGenerator {
        TweetGenerator { seed: 123 }
    }
}

impl TweetGenerator {
    // pub fn generate(&self) -> Datastore {
    //     let mut tweets = vec!();
    //     let mut users = vec!();
    //     let mut follows = vec!();
    //     Datastore { tweets, users, follows }
    // }
}

pub struct LoadGraph {
    users: Mmap,
    follows: Mmap,
}

impl LoadGraph {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            users: unsafe { Mmap::map(&File::open("data/users.bin")?)? },
            follows: unsafe { Mmap::map(&File::open("data/follows.bin")?)? },
        })
    }

    pub fn graph<'a>(&'a self) -> Graph<'a> {
        Graph {
            users: cast_slice(self.users.deref()),
            follows: cast_slice(self.follows.deref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};

    pub fn n_eq(n: usize, ex: Expect) {
        ex.assert_eq(&n.to_string());
    }

    #[test]
    fn loading() {
        let loader = LoadGraph::new().unwrap();
        let graph = loader.graph();

        n_eq(graph.users.len(), expect!["41652230"]);
        n_eq(graph.follows.len(), expect!["1468365182"]);

        let non_trivial = graph.users.iter().filter(|u| u.num_follows > 15).count();
        n_eq(non_trivial, expect!["14050797"]);

        let max_follows = graph.users.iter().map(|u| u.num_follows).max().unwrap();
        n_eq(max_follows, expect!["770155"]);

        let mut follower_counts = vec![0u32; graph.users.len()];
        for f in graph.follows {
            follower_counts[*f as usize] += 1;
        }
        let max_followers = follower_counts.iter().cloned().max().unwrap();
        n_eq(max_followers as usize, expect!["2997469"]);
    }
}
