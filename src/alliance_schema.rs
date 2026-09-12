//! Typed, data-driven definitions for Alliance-token effects and triggers.

use serde::Deserialize;

use crate::card_schema::{CardColor, Race};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AllianceTokenDefinition {
    pub race: Race,
    pub id: u8,
    pub name: String,
    #[serde(default)]
    pub immediate: Vec<AllianceEffect>,
    #[serde(default)]
    pub persistent: Vec<PersistentEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AllianceEffect {
    GainCoins { amount: u8 },
    AdvanceQuest { spaces: u8 },
    PlaceUnitsAnywhere { amount: u8 },
    MoveUnits { amount: u8 },
    RemoveEnemyUnits { amount: u8 },
    OpponentLosesCoins { amount: u8 },
    TakeExtraTurn,
    RemoveEnemyFortress,
    PlayDiscardedCardForFree,
    ChooseManeuvers { times: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PersistentEffect {
    OnCardPlayed {
        color: CardColor,
        effect: AllianceEffect,
    },
    OnChainedCardPlayed {
        effect: AllianceEffect,
    },
    OnLandmarkTaken {
        effect: AllianceEffect,
    },
    DiscardIncomeMultiplier {
        multiplier: u8,
    },
    WaiveLandmarkFortressCost,
    AnySkillOncePerTurn,
    RedDeploymentAnywhere,
    AdditionalRedUnits {
        amount: u8,
    },
    EagleRaceSymbol,
}
