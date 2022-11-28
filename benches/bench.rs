use criterion::{
    black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
// use twitterperf::data::Datastore;
use twitterperf::generate::{LoadGraph, TweetGenerator, TweetGeneratorConfig};
use twitterperf::timeline::Timeline;

// fn bench_merge<'a>(b: &mut Bencher, input: &'a mut (&'a mut TweetGenerator, &'a mut Datastore<'a>)) {
//     let (gen, data) = input;
//     b.iter(|| {
//         let user_idx = black_box(gen.gen_view());
//         Timeline::for_user(&data, user_idx, 200)
//     });
// }

fn criterion_benchmark(c: &mut Criterion) {
    let loader = LoadGraph::new().unwrap();
    let graph = loader.graph();

    let n_tweets = 4_000_000;
    let mut config = TweetGeneratorConfig::default();
    config.capacity = n_tweets;
    let (mut gen, mut data) = TweetGenerator::new(config, graph);

    gen.add_tweets(&mut data, n_tweets);

    // c.bench_with_input(BenchmarkId::new("timeline_merge", "default"), &mut (&mut gen, &mut data), bench_merge);
    let mut group = c.benchmark_group("timeline");
    group.throughput(Throughput::Elements(69));
    group.bench_function("merge", |b| {
        b.iter(|| {
            let user_idx = black_box(gen.gen_view());
            Timeline::for_user(&data, user_idx, 200)
        })
    });
    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
