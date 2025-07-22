use std::collections::HashMap;

use chrono::NaiveDateTime;

/// Struct to manage currently active and historical inodes
#[derive(Debug, Default)]
pub struct Inodes {
    /// Vector to hold all historical inodes and, if drain_active is called, all active inodes
    pub all: Vec<Inode>,
    /// HashMap to hold all currently active inodes
    active: HashMap<u64, Inode>,
}

impl Inodes {
    /// Create a new Inodes struct with an empty all vector and an empty active hashmap
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an inode to the active hashmap
    pub fn append(&mut self, inode: u64, timestamp: Option<chrono::NaiveDateTime>) {
        self.active.entry(inode).or_insert(Inode {
            inode,
            created: timestamp,
            ..Default::default()
        });
    }

    /// Remove an inode from the active hashmap and append it to the all vector
    pub fn delete(&mut self, inode: u64, timestamp: Option<chrono::NaiveDateTime>) {
        if let Some(deleted_inode) = self.active.remove(&inode) {
            self.all.push(deleted_inode);
        } else {
            self.all.push(Inode {
                inode,
                deleted: timestamp,
                ..Default::default()
            })
        }
    }

    /// Update the length of an inode and the amount of data written to it.
    pub fn update_length(&mut self, inode: u64, length: u64) {
        if let Some(i) = self.active.get_mut(&inode) {
            if i.last_known_length == 0 {
                i.written += length;
            } else if length > i.last_known_length {
                i.written += length - i.last_known_length;
            }
            // We do not count truncuations for now
            i.last_known_length = length
        } else {
            self.active.insert(
                inode,
                Inode {
                    inode,
                    last_known_length: length,
                    ..Default::default()
                },
            );
        }
    }

    /// Drain the active hashmap and append all inodes to the all vector
    pub fn drain_active(&mut self) {
        for (_, i) in self.active.drain() {
            self.all.push(i);
        }
    }
}

/// Struct to hold inode information
/// Note that the inode is not unique, as it may be reused after deletion, so we can't use a
/// hashmap for storing information for each inode. The `Inodes` struct is used to manage this.
#[derive(Debug, Default)]
pub struct Inode {
    /// The inode number
    pub inode: u64,
    /// The timestamp the inode, if known, was created
    pub created: Option<NaiveDateTime>,
    /// The timestamp the inode, if known, was deleted
    pub deleted: Option<NaiveDateTime>,
    /// The last known length of the inode
    pub last_known_length: u64,
    /// The amount of data written to the inode
    /// Note that truncations are not counted
    pub written: u64,
}
