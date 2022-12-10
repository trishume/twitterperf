use std::time::Instant;

use twitterperf::data::{START_TIME};
use twitterperf::generate::{LoadGraph, TweetGenerator, TweetGeneratorConfig};
use twitterperf::timeline::{TimelineFetcher};

use signpost::{trace_function, AutoTrace};

fn main() {
    let loader = LoadGraph::new().unwrap();
    let graph = loader.graph();

    let n_tweets = 30_000_000;
    let mut config = TweetGeneratorConfig::default();
    config.capacity = n_tweets;
    let (mut gen, mut data) = TweetGenerator::new(config, graph);

    trace_function(1, &[0; 4], || gen.add_tweets(&mut data, n_tweets));

    let _x = AutoTrace::new(2, &[0usize; 4]);
    let n_views = 10_000;
    let mut total_viewed = 0usize;
    // let mut total_likes = 0u32;
    let start = Instant::now();
    let mut fetcher = TimelineFetcher::default();
    for _ in 0..n_views {
        let user_idx = gen.gen_view();
        let timeline = fetcher.for_user(&data, user_idx, 256, START_TIME);
        total_viewed += timeline.tweets.len();
        // total_likes += timeline.tweets.iter().map(|t| t.likes).sum::<u32>();
    }
    let dur = Instant::now() - start;
    let rate = total_viewed as f64 / dur.as_secs_f64();
    let avg_timeline_size = total_viewed as f64 / n_views as f64;
    let expansion = (avg_timeline_size * gen.viewing_users.len() as f64) / n_tweets as f64;
    eprintln!("Done {total_viewed} in {dur:?} at {rate:.3} tweets/s. Avg timeline size {avg_timeline_size:.2} -> expansion {expansion:.2}");
    // eprintln!("{total_likes}");
}
