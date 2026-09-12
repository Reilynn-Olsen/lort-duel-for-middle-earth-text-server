use std::collections::BTreeMap;

use crate::card_schema;
use crate::catalog::alliance_token_definition;
use crate::{AllianceToken, QuestBonus, QuestBonusEffect};

pub(crate) fn alliance_token_description(token: AllianceToken) -> &'static str {
    &alliance_token_definition(token).name
}

pub(crate) fn quest_bonuses() -> Vec<QuestBonus> {
    use QuestBonusEffect::*;
    [
        (4, GainCoin),
        (7, PlaceUnit),
        (10, TakeExtraTurn),
        (13, RemoveEnemyFortress),
    ]
    .into_iter()
    .map(|(position, effect)| QuestBonus {
        position,
        effect,
        claimed: false,
    })
    .collect()
}

/// Each choice symbol is used once per payment and produces exactly one of its
/// alternatives. There are only five Skills, but choosing greedily can still
/// be wrong when multiple symbols satisfy overlapping requirements; this
/// small exhaustive search therefore returns the exact least coin shortfall.
pub(crate) fn minimum_missing_skills(
    required: &BTreeMap<card_schema::Skill, u8>,
    fixed_production: BTreeMap<card_schema::Skill, u8>,
    choice_symbols: &[Vec<BTreeMap<card_schema::Skill, u8>>],
) -> u8 {
    fn missing(
        required: &BTreeMap<card_schema::Skill, u8>,
        produced: &BTreeMap<card_schema::Skill, u8>,
    ) -> u8 {
        required
            .iter()
            .map(|(skill, amount)| amount.saturating_sub(*produced.get(skill).unwrap_or(&0)))
            .sum()
    }
    fn choose(
        index: usize,
        required: &BTreeMap<card_schema::Skill, u8>,
        produced: BTreeMap<card_schema::Skill, u8>,
        choices: &[Vec<BTreeMap<card_schema::Skill, u8>>],
    ) -> u8 {
        if index == choices.len() {
            return missing(required, &produced);
        }
        choices[index]
            .iter()
            .map(|alternative| {
                let mut next = produced.clone();
                for (skill, amount) in alternative {
                    *next.entry(*skill).or_insert(0) += amount;
                }
                choose(index + 1, required, next, choices)
            })
            .min()
            .unwrap_or_else(|| choose(index + 1, required, produced, choices))
    }
    choose(0, required, fixed_production, choice_symbols)
}
