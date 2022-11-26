#![allow(dead_code)]
use std::collections::BinaryHeap;

use crate::data::*;

pub const CACHE_SIZE: usize = 128;

pub struct CachedTimeline {
    tweets: [TweetIdx; CACHE_SIZE],
}

pub struct Timeline {
    tweets: Vec<Tweet>,
}

impl Timeline {
    fn for_user(data: &Datastore, user_idx: UserIdx, max_len: usize) -> Self {
        let mut tweets: Vec<Tweet> = Vec::with_capacity(max_len);

        let user = &data.graph.users[user_idx as usize];
        let mut heap: BinaryHeap<NextLink> = BinaryHeap::with_capacity(user.num_follows);

        // seed heap
        for follow in &data.graph.follows[user.follows_idx..][..user.num_follows] {
            if let Some(next_link) = data.feeds[*follow as usize] {
                heap.push(next_link)
            }
        }

        // compose timeline
        while let Some(NextLink { ts: _, tweet_idx }) = heap.pop() {
            let chain = &data.tweets[tweet_idx as usize];
            tweets.push(chain.tweet.clone());

            if let Some(next_link) = chain.prev_tweet {
                heap.push(next_link)
            }
        }

        Timeline { tweets }
    }
}
