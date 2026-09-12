//! Typed, language-independent data for Landmark tiles.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::card_schema::{Region, Skill};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LandmarkDefinition {
    pub id: String,
    pub name: String,
    pub cost: LandmarkCost,
    pub region: Region,
    pub effects: Vec<LandmarkEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LandmarkCost {
    pub coins_per_existing_fortress: u8,
    #[serde(default)]
    pub skills: BTreeMap<Skill, u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LandmarkEffect {
    PlaceFortress {
        location: Region,
    },
    PlaceUnits {
        amount: u8,
        regions: Vec<Region>,
        choose: u8,
    },
    GainCoins {
        amount: u8,
    },
    MoveUnits {
        amount: u8,
    },
    AdvanceQuest {
        spaces: u8,
    },
    TwoOfAnyOneRaceTokens,
    DiscardOneGreyCardFromOpponent,
    PlayOneCardFromDiscardForFree,
}
