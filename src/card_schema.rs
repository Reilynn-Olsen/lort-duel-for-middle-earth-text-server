//! Data schema for language-independent printed card faces.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardDefinition {
    pub id: String,
    pub chapter: u8,
    pub color: CardColor,
    pub cost: Cost,
    #[serde(default)]
    pub provides_chain: Option<String>,
    #[serde(default)]
    pub provides: Provides,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardColor {
    Grey,
    Yellow,
    Blue,
    Green,
    Red,
    Purple,
}

/// The mechanical role associated with a card's presentation color. Card
/// effects remain explicit data in `CardDefinition::effects`; callers should
/// not infer an individual card's effect from this family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardFamily {
    PersistentSkills,
    ImmediateCoins,
    QuestMovement,
    RaceAlliance,
    UnitDeployment,
    Maneuver,
}

impl CardColor {
    pub const fn family(self) -> CardFamily {
        match self {
            Self::Grey => CardFamily::PersistentSkills,
            Self::Yellow => CardFamily::ImmediateCoins,
            Self::Blue => CardFamily::QuestMovement,
            Self::Green => CardFamily::RaceAlliance,
            Self::Red => CardFamily::UnitDeployment,
            Self::Purple => CardFamily::Maneuver,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Cost {
    #[serde(default)]
    pub coins: u8,
    #[serde(default)]
    pub skills: BTreeMap<Skill, u8>,
    /// Possessing this symbol makes the entire card free.
    #[serde(default)]
    pub chain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Provides {
    #[serde(default)]
    pub skills: BTreeMap<Skill, u8>,
    /// Choose exactly one alternative when using this card's Skills that turn.
    #[serde(default)]
    pub skills_any_of: Vec<BTreeMap<Skill, u8>>,
    #[serde(default)]
    pub race: Option<Race>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Ruse,
    Strength,
    Courage,
    Knowledge,
    Leadership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    Elves,
    Ents,
    Hobbits,
    Humans,
    Dwarves,
    Wizards,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    GainCoins {
        amount: u8,
    },
    AdvanceQuest {
        spaces: u8,
    },
    PlaceUnits {
        amount: u8,
        regions: Vec<Region>,
        choose: u8,
    },
    MoveUnits {
        amount: u8,
    },
    RemoveEnemyUnits {
        amount: u8,
    },
    OpponentLosesCoins {
        amount: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Lindon,
    Arnor,
    Rhovanion,
    Enedwaith,
    Rohan,
    Gondor,
    Mordor,
}

impl CardDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=3).contains(&self.chapter) {
            return Err(format!("{} has an invalid Chapter", self.id));
        }
        if self.id != format!("chapter{}_card_{:02}", self.chapter, card_number(&self.id)?) {
            return Err(format!("{} is not a stable Chapter-card ID", self.id));
        }
        validate_skills(&self.cost.skills, &self.id)?;
        validate_skills(&self.provides.skills, &self.id)?;
        for choice in &self.provides.skills_any_of {
            if choice.is_empty() {
                return Err(format!("{} has an empty choice Skill", self.id));
            }
            validate_skills(choice, &self.id)?;
        }
        for effect in &self.effects {
            match effect {
                Effect::GainCoins { amount }
                | Effect::AdvanceQuest { spaces: amount }
                | Effect::MoveUnits { amount }
                | Effect::RemoveEnemyUnits { amount }
                | Effect::OpponentLosesCoins { amount }
                    if *amount == 0 =>
                {
                    return Err(format!("{} has a zero-value effect", self.id));
                }
                Effect::PlaceUnits {
                    amount,
                    regions,
                    choose,
                } if *amount == 0 || regions.len() != 2 || *choose != 1 => {
                    return Err(format!("{} has an invalid Unit choice", self.id));
                }
                _ => {}
            }
        }
        self.validate_family()
    }

    /// Reject catalogue data that would blur a card's family role. This is a
    /// data validation boundary, not the game-effect resolver: every concrete
    /// effect is still represented independently in `effects`.
    pub fn validate_family(&self) -> Result<(), String> {
        let has_skills =
            !self.provides.skills.is_empty() || !self.provides.skills_any_of.is_empty();
        let invalid = match self.color.family() {
            CardFamily::PersistentSkills => {
                !has_skills || self.provides.race.is_some() || !self.effects.is_empty()
            }
            CardFamily::ImmediateCoins => {
                has_skills
                    || self.provides.race.is_some()
                    || self.effects.is_empty()
                    || !self
                        .effects
                        .iter()
                        .all(|effect| matches!(effect, Effect::GainCoins { .. }))
            }
            CardFamily::QuestMovement => {
                has_skills
                    || self.provides.race.is_some()
                    || self.effects.is_empty()
                    || !self
                        .effects
                        .iter()
                        .all(|effect| matches!(effect, Effect::AdvanceQuest { .. }))
            }
            CardFamily::RaceAlliance => {
                has_skills || self.provides.race.is_none() || !self.effects.is_empty()
            }
            CardFamily::UnitDeployment => {
                has_skills
                    || self.provides.race.is_some()
                    || self.effects.is_empty()
                    || !self
                        .effects
                        .iter()
                        .all(|effect| matches!(effect, Effect::PlaceUnits { .. }))
            }
            CardFamily::Maneuver => {
                self.chapter != 3
                    || has_skills
                    || self.provides.race.is_some()
                    || self.effects.is_empty()
                    || !self.effects.iter().all(|effect| {
                        matches!(
                            effect,
                            Effect::MoveUnits { .. }
                                | Effect::RemoveEnemyUnits { .. }
                                | Effect::OpponentLosesCoins { .. }
                        )
                    })
            }
        };
        (!invalid).then_some(()).ok_or_else(|| {
            format!(
                "{} has data incompatible with its {:?} card family",
                self.id,
                self.color.family()
            )
        })
    }
}

fn card_number(id: &str) -> Result<u8, String> {
    id.rsplit_once("_card_")
        .and_then(|(_, number)| number.parse().ok())
        .ok_or_else(|| format!("{id} does not end in a card number"))
}

fn validate_skills(skills: &BTreeMap<Skill, u8>, id: &str) -> Result<(), String> {
    if skills.values().any(|amount| *amount == 0) {
        Err(format!("{id} has a zero-value Skill"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_catalogue_is_valid_json_and_preserves_printed_data() {
        let cards: Vec<CardDefinition> =
            serde_json::from_str(include_str!("../cards.json")).unwrap();
        assert_eq!(cards.len(), 69);
        assert_eq!(cards[0].provides.skills[&Skill::Ruse], 1);
        assert_eq!(cards[8].effects, vec![Effect::GainCoins { amount: 2 }]);
        assert!(cards.iter().all(|card| card.validate_family().is_ok()));
        assert_eq!(CardColor::Purple.family(), CardFamily::Maneuver);
    }
}
