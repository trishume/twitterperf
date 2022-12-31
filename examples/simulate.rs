use std::thread;
use std::time::Instant;

use twitterperf::data::START_TIME;
use twitterperf::generate::{LoadGraph, TweetGenerator, TweetGeneratorConfig, ViewGenerator};
use twitterperf::timeline::TimelineFetcher;

use signpost::{trace_function, AutoTrace};

fn main() {
    let loader = LoadGraph::new().unwrap();
    let graph = loader.graph();

    let n_test_add = 15_000_000;
    let n_tweets = 30_000_000 - n_test_add;
    let config = TweetGeneratorConfig::default();
    let (mut gen, viewing_users, mut data) = TweetGenerator::new(config, graph);

    let add_start = Instant::now();
    trace_function(1, &[0; 4], || gen.add_tweets(&mut data, n_tweets));
    let add_dur = Instant::now() - add_start;
    let add_rate = n_tweets as f64 / add_dur.as_secs_f64();
    eprintln!("Initially added {n_tweets} tweets in {add_dur:?}: {add_rate:.3} tweets/s.");

    let add_start = Instant::now();
    trace_function(1, &[0; 4], || gen.add_tweets(&mut data, n_tweets));
    let add_dur = Instant::now() - add_start;
    let add_rate = n_test_add as f64 / add_dur.as_secs_f64();
    eprintln!("Benchmarked adding {n_test_add} tweets in {add_dur:?}: {add_rate:.3} tweets/s.");

    let _x = AutoTrace::new(2, &[0usize; 4]);
    let n_views = 100_000;
    // let mut total_likes = 0u32;
    let n_threads = 8;
    eprintln!("Starting fetches from {n_threads} threads");
    let viewing_users = &viewing_users[..];
    let data = &data;
    thread::scope(|s| {
        for _ in 0..n_threads {
            let seed: u64 = gen.fork_seed();
            s.spawn(move || {
            let mut view_gen = ViewGenerator::new(seed, viewing_users);
            let mut total_viewed = 0usize;
            let start = Instant::now();
            let mut fetcher = TimelineFetcher::default();
            for _ in 0..n_views {
                let user_idx = view_gen.gen_view();
                let timeline = fetcher.for_user(data, user_idx, 256, START_TIME);
                total_viewed += timeline.tweets.len();
                // total_likes += timeline.tweets.iter().map(|t| t.likes).sum::<u32>();
            }
            let dur = Instant::now() - start;
            let rate = total_viewed as f64 / dur.as_secs_f64();
            let avg_timeline_size = total_viewed as f64 / n_views as f64;
            let expansion = (avg_timeline_size * view_gen.viewing_users.len() as f64) / n_tweets as f64;
            eprintln!("Done {total_viewed} in {dur:?} at {rate:.3} tweets/s. Avg timeline size {avg_timeline_size:.2} -> expansion {expansion:.2}");
            });
        }
    });
    // eprintln!("{total_likes}");
}
