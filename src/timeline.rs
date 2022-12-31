#![allow(dead_code)]
use ringbuffer::ConstGenericRingBuffer;
use ringbuffer::RingBufferWrite;
use static_assertions::assert_eq_size;
use std::collections::BinaryHeap;

use crate::data::*;

pub struct Timeline<'a> {
    pub tweets: &'a [Tweet],
}

pub const CACHE_SIZE: usize = 123;

#[derive(Clone)]
#[repr(align(64))]
pub struct CachedTimeline {
    tweets: ConstGenericRingBuffer<TweetIdx, CACHE_SIZE>,
}
assert_eq_size!([u8; 512], CachedTimeline);

impl Default for CachedTimeline {
    fn default() -> Self {
        Self {
            tweets: ConstGenericRingBuffer::new(),
        }
    }
}

pub struct TimelineCache {
    pub timelines: Vec<CachedTimeline>,
}

impl TimelineCache {
    fn new(graph: &Graph) -> Self {
        Self {
            timelines: vec![CachedTimeline::default(); graph.users.len()],
        }
    }

    fn publish_tweet(&mut self, graph: &Graph, user_idx: UserIdx, tweet_idx: TweetIdx) {
        let user = &graph.users[user_idx as usize];
        for follow in graph.user_follows(user) {
            self.timelines[*follow as usize].tweets.push(tweet_idx);
        }
    }
}

#[derive(Default)]
pub struct TimelineFetcher {
    tweets: Vec<Tweet>,
    heap: BinaryHeap<NextLink>,
}

impl TimelineFetcher {
    #[inline]
    fn push_after(&mut self, link: Option<NextLink>, after: Timestamp) {
        link.filter(|l| l.ts >= after).map(|l| self.heap.push(l));
    }

    pub fn for_user<'a>(
        &'a mut self,
        data: &Datastore,
        user_idx: UserIdx,
        max_len: usize,
        after: Timestamp,
    ) -> Timeline<'a> {
        self.heap.clear();
        self.tweets.clear();
        let user = &data.graph.users[user_idx as usize];

        // seed heap
        for follow in data.graph.user_follows(user) {
            self.push_after(data.feeds[*follow as usize], after);
        }

        // compose timeline
        while let Some(NextLink { ts: _, tweet_idx }) = self.heap.pop() {
            let chain = &data.tweets[tweet_idx as usize];
            // tweets.push(Tweet::dummy(NonZeroU64::new(1).unwrap()));
            self.tweets.push(chain.tweet.clone());
            if self.tweets.len() >= max_len {
                break;
            }

            self.push_after(chain.prev_tweet, after);
        }

        Timeline {
            tweets: &self.tweets[..],
        }
    }
}
