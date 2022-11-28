#![allow(dead_code)]
use std::{collections::BinaryHeap, num::NonZeroU64};

use crate::data::*;

pub const CACHE_SIZE: usize = 128;

pub struct CachedTimeline {
    tweets: [TweetIdx; CACHE_SIZE],
}

pub struct Timeline {
    pub tweets: Vec<Tweet>,
}

impl Timeline {
    pub fn for_user(data: &Datastore, user_idx: UserIdx, max_len: usize) -> Self {
        let mut tweets: Vec<Tweet> = Vec::with_capacity(max_len);

        let user = &data.graph.users[user_idx as usize];
        let mut heap: BinaryHeap<NextLink> = BinaryHeap::with_capacity(user.num_follows as usize);

        // seed heap
        for follow in &data.graph.follows[user.follows_idx..][..user.num_follows as usize] {
            if let Some(next_link) = data.feeds[*follow as usize] {
                heap.push(next_link)
            }
        }

        // compose timeline
        while let Some(NextLink { ts: _, tweet_idx }) = heap.pop() {
            let chain = &data.tweets[tweet_idx as usize];
            // tweets.push(Tweet::dummy(NonZeroU64::new(1).unwrap()));
            tweets.push(chain.tweet.clone());

            if let Some(next_link) = chain.prev_tweet {
                // data.prefetch_tweet(next_link.tweet_idx);
                heap.push(next_link)
            }
        }

        Timeline { tweets }
    }
}
