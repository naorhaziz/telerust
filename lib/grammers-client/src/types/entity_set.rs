// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::types::Entity;
use grammers_tl_types as tl;
use std::collections::HashMap;
use std::sync::Arc;

/// Hashable `Peer`.
#[derive(Hash, PartialEq, Eq)]
pub(crate) enum Peer {
    User(i32),
    Chat(i32),
    Channel(i32),
}

impl From<&tl::enums::Peer> for Peer {
    fn from(peer: &tl::enums::Peer) -> Self {
        use tl::enums::Peer::*;

        match peer {
            User(user) => Self::User(user.user_id),
            Chat(chat) => Self::Chat(chat.chat_id),
            Channel(channel) => Self::Channel(channel.channel_id),
        }
    }
}

/// Helper structure to efficiently retrieve entities via their peer.
///
/// A lot of responses include the entities related to them in the form of a list of users
/// and chats, making it annoying to extract a specific entity. This structure lets you
/// save those separate vectors in a single place and query them by using a `Peer`.
pub struct EntitySet {
    map: HashMap<Peer, Entity>,
}

/// In-memory entity cache, mapping peers to their respective access hashes.
pub(crate) struct EntityCache {
    users: HashMap<i32, i64>,
    channels: HashMap<i32, i64>,
    self_id: Option<i32>,
    self_bot: bool,
}

impl EntitySet {
    /// Create a new entity set.
    pub fn new(users: Vec<tl::enums::User>, chats: Vec<tl::enums::Chat>) -> Arc<Self> {
        use tl::enums::{Chat, User};

        Arc::new(Self {
            map: users
                .into_iter()
                .filter_map(|user| match user {
                    User::User(user) => Some(Entity::User(user)),
                    User::Empty(_) => None,
                })
                .chain(chats.into_iter().filter_map(|chat| match chat {
                    Chat::Empty(_) => None,
                    Chat::Chat(chat) => Some(Entity::Chat(chat)),
                    Chat::Forbidden(_) => None,
                    Chat::Channel(channel) => Some(Entity::Channel(channel)),
                    Chat::ChannelForbidden(_) => None,
                    // TODO *Forbidden have some info which may be relevant at times
                    // currently ignored for simplicity
                }))
                .map(|entity| ((&entity.peer()).into(), entity))
                .collect(),
        })
    }

    /// Create a new empty entity set.
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            map: HashMap::new(),
        })
    }

    /// Retrieve the full `Entity` object given its `Peer`.
    pub fn get<'a, 'b>(&'a self, peer: &'b tl::enums::Peer) -> Option<&'a Entity> {
        self.map.get(&peer.into())
    }
}

impl EntityCache {
    pub(crate) fn new() -> Self {
        Self {
            users: HashMap::new(),
            channels: HashMap::new(),
            self_id: None,
            self_bot: false,
        }
    }

    pub(crate) fn self_id(&self) -> i32 {
        self.self_id
            .expect("tried to query self_id before it's known")
    }

    pub(crate) fn is_self_bot(&self) -> bool {
        self.self_bot
    }

    pub(crate) fn contains(&self, peer: tl::enums::Peer) -> bool {
        match peer {
            tl::enums::Peer::User(u) => self.users.contains_key(&u.user_id),
            tl::enums::Peer::Chat(_) => true,
            tl::enums::Peer::Channel(c) => self.channels.contains_key(&c.channel_id),
        }
    }

    pub(crate) fn get_input(&self, peer: tl::enums::Peer) -> Option<tl::enums::InputPeer> {
        match peer {
            tl::enums::Peer::User(u) => self.users.get(&u.user_id).map(|&access_hash| {
                tl::types::InputPeerUser {
                    user_id: u.user_id,
                    access_hash,
                }
                .into()
            }),
            tl::enums::Peer::Chat(c) => {
                Some(tl::types::InputPeerChat { chat_id: c.chat_id }.into())
            }
            tl::enums::Peer::Channel(c) => self.channels.get(&c.channel_id).map(|&access_hash| {
                tl::types::InputPeerChannel {
                    channel_id: c.channel_id,
                    access_hash,
                }
                .into()
            }),
        }
    }
}
