use std::collections::BTreeMap;

use crate::catalog::{alliance_token_definition, card_definition, landmark_definition};
use crate::minimum_missing_skills;
use crate::{ChapterCard, Faction, alliance_schema, card_schema};

use super::Game;

impl Game {
    pub(super) fn discard_income_for(&self, player: Faction) -> u8 {
        self.current_chapter
            * self
                .player(player)
                .alliance_tokens
                .iter()
                .flat_map(|token| alliance_token_definition(*token).persistent.iter())
                .filter_map(|effect| match effect {
                    alliance_schema::PersistentEffect::DiscardIncomeMultiplier { multiplier } => {
                        Some(*multiplier)
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(1)
    }
    pub(super) fn can_use_any_skill(&self, player: Faction) -> bool {
        self.has_persistent(player, |effect| {
            matches!(
                effect,
                alliance_schema::PersistentEffect::AnySkillOncePerTurn
            )
        }) && !self.player(player).used_any_skill_this_turn
    }
    pub(super) fn has_persistent(
        &self,
        player: Faction,
        predicate: impl Fn(&alliance_schema::PersistentEffect) -> bool,
    ) -> bool {
        self.player(player).alliance_tokens.iter().any(|token| {
            alliance_token_definition(*token)
                .persistent
                .iter()
                .any(&predicate)
        })
    }
    pub(super) fn triggered_card_effects(
        &self,
        player: Faction,
        color: card_schema::CardColor,
        chained: bool,
    ) -> Vec<alliance_schema::AllianceEffect> {
        self.player(player)
            .alliance_tokens
            .iter()
            .flat_map(|token| alliance_token_definition(*token).persistent.iter())
            .filter_map(|persistent| match persistent {
                alliance_schema::PersistentEffect::OnCardPlayed {
                    color: trigger,
                    effect,
                } if *trigger == color => Some(effect.clone()),
                alliance_schema::PersistentEffect::OnChainedCardPlayed { effect } if chained => {
                    Some(effect.clone())
                }
                _ => None,
            })
            .collect()
    }
    pub(super) fn landmark_triggered_effects(
        &self,
        player: Faction,
    ) -> Vec<alliance_schema::AllianceEffect> {
        self.player(player)
            .alliance_tokens
            .iter()
            .flat_map(|token| alliance_token_definition(*token).persistent.iter())
            .filter_map(|persistent| match persistent {
                alliance_schema::PersistentEffect::OnLandmarkTaken { effect } => {
                    Some(effect.clone())
                }
                _ => None,
            })
            .collect()
    }
    pub(super) fn additional_red_units(&self, player: Faction) -> u8 {
        self.player(player)
            .alliance_tokens
            .iter()
            .flat_map(|token| alliance_token_definition(*token).persistent.iter())
            .filter_map(|effect| match effect {
                alliance_schema::PersistentEffect::AdditionalRedUnits { amount } => Some(*amount),
                _ => None,
            })
            .sum()
    }
    pub(super) fn uses_chain(&self, player: Faction, card: ChapterCard) -> bool {
        card_definition(card)
            .cost
            .chain
            .as_ref()
            .is_some_and(|chain| {
                self.player(player)
                    .tableau
                    .iter()
                    .filter(|owned| **owned != card)
                    .any(|owned| card_definition(*owned).provides_chain.as_ref() == Some(chain))
            })
    }
    /// Returns the minimum coin payment, or `None` when the player cannot pay.
    pub(super) fn payment_for(&self, card: ChapterCard) -> Option<u8> {
        self.payment_for_with_any_skill(card, false)
    }
    pub(super) fn payment_for_using_any_skill(&self, card: ChapterCard) -> Option<u8> {
        self.payment_for_with_any_skill(card, true)
    }
    pub(super) fn payment_for_with_any_skill(
        &self,
        card: ChapterCard,
        using_any_skill: bool,
    ) -> Option<u8> {
        let definition = card_definition(card);
        let player = self.player(self.active_player);
        if definition.cost.chain.as_ref().is_some_and(|chain| {
            player
                .tableau
                .iter()
                .any(|owned| card_definition(*owned).provides_chain.as_ref() == Some(chain))
        }) {
            return (!using_any_skill).then_some(0);
        }
        let mut produced = BTreeMap::new();
        let mut choice_symbols = Vec::new();
        for owned in &player.tableau {
            let provides = &card_definition(*owned).provides;
            for (skill, amount) in &provides.skills {
                *produced.entry(*skill).or_insert(0u8) += amount;
            }
            if !provides.skills_any_of.is_empty() {
                choice_symbols.push(provides.skills_any_of.clone());
            }
        }
        let mut missing =
            minimum_missing_skills(&definition.cost.skills, produced, &choice_symbols);
        if using_any_skill {
            if !self.can_use_any_skill(self.active_player) || missing == 0 {
                return None;
            }
            missing -= 1;
        }
        let total = definition.cost.coins + missing;
        (player.coins >= total).then_some(total)
    }
    pub(super) fn normal_payment_for(&self, card: ChapterCard) -> Option<u8> {
        let definition = card_definition(card);
        let player = self.player(self.active_player);
        let mut produced = BTreeMap::new();
        let mut choices = Vec::new();
        for owned in &player.tableau {
            let provides = &card_definition(*owned).provides;
            for (skill, amount) in &provides.skills {
                *produced.entry(*skill).or_insert(0) += amount;
            }
            if !provides.skills_any_of.is_empty() {
                choices.push(provides.skills_any_of.clone());
            }
        }
        let total = definition.cost.coins
            + minimum_missing_skills(&definition.cost.skills, produced, &choices);
        (player.coins >= total).then_some(total)
    }
    pub(super) fn landmark_payment(&self, id: &str) -> Option<u8> {
        self.landmark_payment_with_any_skill(id, false)
    }
    pub(super) fn landmark_payment_using_any_skill(&self, id: &str) -> Option<u8> {
        self.landmark_payment_with_any_skill(id, true)
    }
    pub(super) fn landmark_payment_with_any_skill(
        &self,
        id: &str,
        using_any_skill: bool,
    ) -> Option<u8> {
        let definition = landmark_definition(id);
        let player = self.player(self.active_player);
        let mut produced = BTreeMap::new();
        let mut choice_symbols = Vec::new();
        for card in &player.tableau {
            let provides = &card_definition(*card).provides;
            for (skill, amount) in &provides.skills {
                *produced.entry(*skill).or_insert(0u8) += amount;
            }
            if !provides.skills_any_of.is_empty() {
                choice_symbols.push(provides.skills_any_of.clone());
            }
        }
        let mut missing =
            minimum_missing_skills(&definition.cost.skills, produced, &choice_symbols);
        if using_any_skill {
            if !self.can_use_any_skill(self.active_player) || missing == 0 {
                return None;
            }
            missing -= 1;
        }
        let fortresses = self
            .map
            .iter()
            .filter(|region| region.fortress == Some(self.active_player))
            .count() as u8;
        let extra = if self.has_persistent(self.active_player, |effect| {
            matches!(
                effect,
                alliance_schema::PersistentEffect::WaiveLandmarkFortressCost
            )
        }) || player.waives_landmark_fortress_cost
        {
            0
        } else {
            fortresses * definition.cost.coins_per_existing_fortress
        };
        let total = missing + extra;
        (player.coins >= total).then_some(total)
    }
}
