use std::{num::NonZeroU32, sync::atomic::AtomicU64};

use bytemuck::{NoUninit, Pod, Zeroable};
use static_assertions::assert_eq_size;

use crate::pool::SharedPool;

/// Leave room for a full 280 character plus some accents or emoji.
/// A real implementation would have an escape hatch for longer tweets.
pub const TWEET_BYTES: usize = 284;

pub type Timestamp = NonZeroU32;
pub const START_TIME: Timestamp = unsafe { NonZeroU32::new_unchecked(1) };

#[derive(Clone)]
pub struct Tweet {
    pub content: [u8; TWEET_BYTES],
    pub ts: Timestamp,

    pub likes: u32,
    pub quotes: u32,
    pub retweets: u32,
}

impl Tweet {
    pub fn dummy(ts: Timestamp) -> Self {
        Tweet {
            content: [0; TWEET_BYTES],
            ts,
            likes: 0,
            quotes: 0,
            retweets: 0,
        }
    }
}

// assert_eq_size!([u8; 304], Tweet);

pub type TweetIdx = u32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct NextLink {
    pub ts: Timestamp,
    pub tweet_idx: TweetIdx,
}
assert_eq_size!(AtomicU64, NextLink);

pub type FeedChain = Option<NextLink>;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct PodNextLink(u32, u32);
assert_eq_size!(AtomicU64, PodNextLink);

/// linked list of tweets to make appending fast and avoid space overhead
/// a linked list of chunks of tweets would probably be faster because of
/// cache locality of fetches, but I haven't implemented that
pub struct AtomicChain(AtomicU64);

impl AtomicChain {
    pub fn none() -> Self {
        AtomicChain(AtomicU64::new(0))
    }

    pub fn set(&self, next: NextLink) {
        let as_u64: u64 = bytemuck::cast(next);
        self.0.store(as_u64, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn fetch(&self) -> Option<NextLink> {
        let as_u64 = self.0.load(std::sync::atomic::Ordering::SeqCst);
        // we hope LLVM optimizes this into a no-op
        let pod: PodNextLink = bytemuck::cast(as_u64);
        match Timestamp::new(pod.0) {
            Some(ts) => Some(NextLink {
                ts,
                tweet_idx: pod.1,
            }),
            None => None,
        }
    }
}

#[repr(align(64))]
pub struct ChainedTweet {
    pub tweet: Tweet,
    pub prev_tweet: FeedChain,
}
assert_eq_size!([u8; 320], ChainedTweet);

pub type UserIdx = u32;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct User {
    // This would be better as a Vec for mutation but for fast data loading we use one giant slice
    pub follows_idx: usize,
    pub num_follows: u32,
    pub num_followers: u32,
}

/// We store the Graph in a format we can mmap from a pre-baked file
/// so that our tests can load a real graph faster
pub struct Graph<'a> {
    pub users: &'a [User],
    pub follows: &'a [UserIdx],
}

impl<'a> Graph<'a> {
    #[inline]
    pub fn user_follows(&'a self, user: &User) -> &'a [UserIdx] {
        &self.follows[user.follows_idx..][..user.num_follows as usize]
    }
}

pub struct Datastore<'a> {
    pub graph: Graph<'a>,
    pub tweets: SharedPool<ChainedTweet>,
    pub feeds: Vec<AtomicChain>,
}

impl<'a> Datastore<'a> {
    /// This will clobber writes (in a safe way) if called concurrently
    /// from multiple threads. Ideally we'd have a separate &mut handle for this
    pub fn add_tweet(&self, tweet: Tweet, user_id: UserIdx) {
        let prev_tweet = self.feeds[user_id as usize].fetch();
        let ts = tweet.ts;
        let chained = ChainedTweet { tweet, prev_tweet };
        let tweet_idx = self.tweets.push(chained) as TweetIdx;
        self.feeds[user_id as usize].set(NextLink { ts, tweet_idx });
    }

    pub fn prefetch_tweet(&self, tweet_idx: TweetIdx) {
        let tweet_ptr = &self.tweets[tweet_idx as usize] as *const ChainedTweet;
        unsafe {
            for cache_line in 0..3 {
                let line_ptr = (tweet_ptr as *const i8).offset(64 * cache_line);
                core::arch::x86_64::_mm_prefetch(line_ptr, core::arch::x86_64::_MM_HINT_T0)
            }
        }
    }
}
