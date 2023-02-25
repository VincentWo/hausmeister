#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]

use uuid::Uuid;

mod header;
mod layer;

pub mod backend;
pub mod data;

pub use layer::SessionLayer;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Copy, Clone, Debug)]
pub struct SessionId(Uuid);
