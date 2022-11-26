use std::num::NonZeroU64;

use bytemuck::{Pod, Zeroable};
use static_assertions::assert_eq_size;

pub const TWEET_BYTES: usize = 280;

#[derive(Clone)]
pub struct Tweet {
    pub content: [u8; TWEET_BYTES],
    pub ts: NonZeroU64,

    pub likes: u32,
    pub quotes: u32,
    pub retweets: u32,
}

// assert_eq_size!([u8; 304], Tweet);

pub type TweetIdx = u32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct NextLink {
    pub ts: NonZeroU64,
    pub tweet_idx: TweetIdx,
}

pub type FeedChain = Option<NextLink>;

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
    pub num_follows: usize,
}

pub struct Graph<'a> {
    pub users: &'a [User],
    pub follows: &'a [UserIdx],
}

pub struct Datastore<'a> {
    pub graph: Graph<'a>,
    pub tweets: Vec<ChainedTweet>,
    pub feeds: Vec<FeedChain>,
}
