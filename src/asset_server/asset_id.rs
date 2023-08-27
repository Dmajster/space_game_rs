use std::{marker::PhantomData, fmt::Display};

use serde::{Serialize, Deserialize};

use crate::Id;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetId<T> {
    id: Id,
    pd: PhantomData<T>,
}

impl<T> AssetId<T> {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            pd: PhantomData,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub const EMPTY: AssetId<T> = AssetId {
        id: Id::EMPTY,
        pd: PhantomData,
    };
}

impl<T> Display for AssetId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl<T> PartialEq for AssetId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.pd == other.pd
    }
}

impl<T> Eq for AssetId<T> {}

impl<T> PartialOrd for AssetId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.pd.partial_cmp(&other.pd)
    }
}

impl<T> Ord for AssetId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.id.cmp(&other.id) {
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
        }
        self.pd.cmp(&other.pd)
    }
}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            pd: self.pd.clone(),
        }
    }
}

impl<T> Copy for AssetId<T> {}
