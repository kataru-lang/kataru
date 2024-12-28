use crate::Result;

mod file;
mod text;

pub use file::{Load, LoadMessagePack, LoadYaml, Save, SaveMessagePack, SaveYaml};
pub use text::{FromMessagePack, FromStr, FromYaml, IntoStr};

/// Trait to merge two objects together. Used for maps.
pub trait Merge: Sized {
    fn merge(&mut self, other: &mut Self) -> Result<()>;
}

/// Trait to merge two objects together. Used for maps.
pub trait CopyMerge: Sized + Clone {
    fn copy_merge(&self, other: &Self) -> Result<Self>;
}

/// Trait to move values from one object objects together. Used for maps.
pub trait MoveValues: Sized {
    #[allow(dead_code)]
    fn move_values(other: &mut Self) -> Result<Self>;
}
