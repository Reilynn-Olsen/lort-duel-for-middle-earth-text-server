//! Deterministic, text-first rules primitives for *The Lord of the Rings:
//! Duel for Middle-earth*.

pub mod alliance_schema;
pub mod card_schema;
mod catalog;
#[cfg(test)]
mod catalog_tests;
mod game;
mod helpers;
pub mod landmark_schema;
pub mod types;

pub use game::Game;
pub use types::*;

pub(crate) use helpers::{alliance_token_description, minimum_missing_skills, quest_bonuses};
