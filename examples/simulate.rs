use std::time::Instant;

use twitterperf::generate::{LoadGraph, TweetGenerator, TweetGeneratorConfig};
use twitterperf::timeline::Timeline;

use signpost::{trace_function, AutoTrace};

fn main() {
    let loader = LoadGraph::new().unwrap();
    let graph = loader.graph();

    let n_tweets = 10_000_000;
    let mut config = TweetGeneratorConfig::default();
    config.capacity = n_tweets;
    let (mut gen, mut data) = TweetGenerator::new(config, graph);

    trace_function(1, &[0; 4], || gen.add_tweets(&mut data, n_tweets));

    let _x = AutoTrace::new(2, &[0usize; 4]);
    let n_views = 100_000;
    let mut total_viewed = 0usize;
    let start = Instant::now();
    for _ in 0..n_views {
        let user_idx = gen.gen_view();
        let timeline = Timeline::for_user(&data, user_idx, 200);
        total_viewed += timeline.tweets.len();
    }
    let dur = Instant::now() - start;
    let rate = total_viewed as f64 / dur.as_secs_f64();
    eprintln!("Done {total_viewed} in {dur:?} at {rate:.3} tweets/s");
}
