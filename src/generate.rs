use crate::data::*;
use crate::pool::SharedPool;

use bytemuck::cast_slice;
use memmap2::Mmap;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_wyrand::WyRand;
use std::fs::File;
use std::ops::Deref;

pub struct TweetGeneratorConfig {
    pub seed: u64,
    pub tweeter_follower_thresh: u32,
    pub viewer_follow_thresh: u32,
}

impl Default for TweetGeneratorConfig {
    fn default() -> TweetGeneratorConfig {
        TweetGeneratorConfig {
            seed: 123,
            tweeter_follower_thresh: 20,
            viewer_follow_thresh: 20,
        }
    }
}

pub struct TweetGenerator {
    // config: TweetGeneratorConfig,
    tweeting_users: Vec<UserIdx>,
    pub viewing_users: Vec<UserIdx>,
    rng: WyRand,
    ts: Timestamp,
}

impl TweetGenerator {
    pub fn new<'a>(config: TweetGeneratorConfig, graph: Graph<'a>) -> (Self, Datastore<'a>) {
        let feeds: Vec<AtomicChain> = (0..graph.users.len()).map(|_| AtomicChain::none()).collect();
        let tweets = SharedPool::new().unwrap();

        let mut rng = WyRand::from_seed(config.seed.to_le_bytes());
        let mut tweeting_users: Vec<u32> = graph
            .users
            .iter()
            .enumerate()
            .filter(|(_, u)| u.num_followers > config.tweeter_follower_thresh)
            .map(|(i, _)| i as u32)
            .collect();
        tweeting_users.shuffle(&mut rng);
        let mut viewing_users: Vec<u32> = graph
            .users
            .iter()
            .enumerate()
            .filter(|(_, u)| u.num_follows > config.viewer_follow_thresh)
            .map(|(i, _)| i as u32)
            .collect();
        viewing_users.shuffle(&mut rng);
        let this = Self {
            // config,
            tweeting_users,
            viewing_users,
            rng,
            ts: START_TIME,
        };

        let data = Datastore {
            graph,
            tweets,
            feeds,
        };

        (this, data)
    }

    pub fn gen_tweet(&mut self) -> (UserIdx, Tweet) {
        // TODO Zipf distribution or something
        let user_id: UserIdx = *self.tweeting_users.choose(&mut self.rng).unwrap();
        let tweet = Tweet::dummy(self.ts);
        self.ts = self.ts.saturating_add(1);
        (user_id, tweet)
    }

    pub fn add_tweets(&mut self, data: &mut Datastore, n: usize) {
        for _ in 0..n {
            let (user_id, tweet) = self.gen_tweet();
            data.add_tweet(tweet, user_id);
        }
    }

    pub fn gen_view(&mut self) -> UserIdx {
        // TODO Zipf distribution or something
        let user_id: UserIdx = *self.viewing_users.choose(&mut self.rng).unwrap();
        user_id
    }
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
    use crate::timeline::TimelineFetcher;

    use super::*;
    use expect_test::{expect, Expect};

    pub fn n_eq(n: usize, ex: Expect) {
        ex.assert_eq(&n.to_string());
    }

    pub fn f_eq(f: f64, ex: Expect) {
        ex.assert_eq(&format!("{f:.3}"));
    }

    #[test]
    fn loading() {
        let loader = LoadGraph::new().unwrap();
        let graph = loader.graph();

        n_eq(graph.users.len(), expect!["41652230"]);
        n_eq(graph.follows.len(), expect!["1468365182"]);

        let non_trivial = graph.users.iter().filter(|u| u.num_follows > 20).count();
        n_eq(non_trivial, expect!["9031061"]);

        let max_follows = graph.users.iter().map(|u| u.num_follows).max().unwrap();
        n_eq(max_follows as usize, expect!["770155"]);

        let max_followers = graph.users.iter().map(|u| u.num_followers).max().unwrap();
        n_eq(max_followers as usize, expect!["2997469"]);
    }

    #[test]
    fn generating() {
        let loader = LoadGraph::new().unwrap();
        let graph = loader.graph();

        let n_tweets = 4_000_000;
        let config = TweetGeneratorConfig::default();
        let (mut gen, mut data) = TweetGenerator::new(config, graph);

        n_eq(gen.viewing_users.len(), expect!["9031061"]);
        n_eq(gen.tweeting_users.len(), expect!["6746960"]);

        gen.add_tweets(&mut data, n_tweets);

        let n_views = 100_000;
        let mut total_viewed = 0usize;
        let mut fetcher = TimelineFetcher::default();
        for _ in 0..n_views {
            let user_idx = gen.gen_view();
            let timeline = fetcher.for_user(&data, user_idx, 200, START_TIME);
            total_viewed += timeline.tweets.len();
        }
        let avg_timeline_size = total_viewed as f64 / n_views as f64;
        f_eq(avg_timeline_size, expect!["41.480"]);
        let expansion = (avg_timeline_size * gen.viewing_users.len() as f64) / n_tweets as f64;
        f_eq(expansion, expect!["93.652"]);
    }
}
