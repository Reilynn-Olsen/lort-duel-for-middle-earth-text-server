use crate::catalog::alliance_token_definition;
use crate::{
    Action, AllianceToken, AllianceTrigger, ChapterCard, EntManeuver, Faction, PendingDecision,
    QuestBonusEffect, Race, Region, alliance_schema,
};

use super::{Effect, Game};

impl Game {
    pub(super) fn discard_selected_card(&mut self) {
        let PendingDecision::CardDisposition { card } = self
            .pending_decision
            .take()
            .expect("validated pending choice")
        else {
            unreachable!("validated card disposition action")
        };
        self.discard_card(card);
        let income = self.current_chapter
            * self
                .player(self.active_player)
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
                .unwrap_or(1);
        self.gain_coins(self.active_player, income);
        self.queue_turn_completion();
        self.resolve_effects();
    }
    pub(super) fn discard_opponent_grey_card(&mut self, card: ChapterCard) {
        let PendingDecision::OpponentGreyCardDiscard { player } = self
            .pending_decision
            .take()
            .expect("validated Grey-card discard choice")
        else {
            unreachable!("validated Grey-card discard action")
        };
        let opponent = player.other();
        let position = self
            .player(opponent)
            .tableau
            .iter()
            .position(|owned| *owned == card)
            .expect("validated opponent Grey card");
        self.player_mut(opponent).tableau.remove(position);
        self.discard_card(card);
        self.resolve_effects();
    }
    pub(super) fn play_discarded_card_for_free(&mut self, card: ChapterCard) {
        let PendingDecision::DiscardedCardFreePlay { player } = self
            .pending_decision
            .take()
            .expect("validated discarded-card choice")
        else {
            unreachable!("validated free-play action")
        };
        assert!(self.remove_discarded_card(card));
        self.player_mut(player).tableau.push(card);
        self.effect_queue.push_front(Effect::ResolvePlayedCard {
            player,
            card,
            used_chain: false,
        });
        self.resolve_effects();
    }
    pub(super) fn resolve_alliance_trigger(&mut self, trigger: AllianceTrigger) {
        let PendingDecision::AllianceTriggerChoice { player, triggers } = self
            .pending_decision
            .take()
            .expect("validated Alliance trigger choice")
        else {
            unreachable!("validated Alliance trigger action")
        };
        debug_assert!(triggers.contains(&trigger));
        self.effect_queue
            .push_front(Effect::ResolveRaceAllianceTriggers { player });
        match trigger {
            AllianceTrigger::MatchingRace(race) => {
                self.player_mut(player).matched_races.push(race);
                self.reveal_alliance_tokens(player, &[race], 2);
            }
            AllianceTrigger::ThreeDifferentRaces => {
                self.player_mut(player).claimed_three_race_alliance = true;
                let races = self.player_races(player);
                self.reveal_alliance_tokens(player, &races[..3], 1);
            }
        }
    }
    pub(super) fn choose_alliance_race(&mut self, race: Race) {
        let PendingDecision::AllianceRaceChoice { player } = self
            .pending_decision
            .take()
            .expect("validated Alliance stack choice")
        else {
            unreachable!("validated Alliance stack action")
        };
        self.reveal_alliance_tokens(player, &[race], 2);
    }
    pub(super) fn reveal_alliance_tokens(
        &mut self,
        player: Faction,
        races: &[Race],
        per_race: usize,
    ) {
        let mut revealed = Vec::new();
        for race in races {
            let stack = self
                .alliance_stacks
                .iter_mut()
                .find(|stack| stack.race == *race)
                .expect("every Race has an Alliance stack");
            for _ in 0..per_race {
                let Some(id) = stack.token_order.pop() else {
                    break;
                };
                revealed.push(AllianceToken { race: *race, id });
            }
        }
        debug_assert!(!revealed.is_empty());
        self.pending_decision = Some(PendingDecision::AllianceTokenChoice { player, revealed });
    }
    pub(super) fn take_alliance_token(&mut self, token: AllianceToken) {
        let PendingDecision::AllianceTokenChoice { player, revealed } = self
            .pending_decision
            .take()
            .expect("validated Alliance-token choice")
        else {
            unreachable!("validated Alliance-token action")
        };
        debug_assert!(revealed.contains(&token));
        for unchosen in revealed.into_iter().filter(|candidate| *candidate != token) {
            self.alliance_stacks
                .iter_mut()
                .find(|stack| stack.race == unchosen.race)
                .expect("every Race has an Alliance stack")
                .token_order
                .push(unchosen.id);
        }
        self.player_mut(player).alliance_tokens.push(token);
        self.check_race_victory(player);
        if self.winner.is_none() {
            self.effect_queue
                .push_front(Effect::ResolveAllianceToken { player, token });
            self.resolve_effects();
        }
    }
    pub(super) fn choose_ent_maneuver(&mut self, maneuver: EntManeuver) {
        let PendingDecision::EntManeuverChoice { player, remaining } = self
            .pending_decision
            .take()
            .expect("validated Ent maneuver choice")
        else {
            unreachable!("validated Ent maneuver action")
        };
        if remaining > 1 {
            self.effect_queue.push_front(Effect::ResolveEntManeuvers {
                player,
                remaining: remaining - 1,
            });
        }
        match maneuver {
            EntManeuver::OpponentLosesCoin => {
                self.lose_coins(player.other(), 1);
                self.resolve_effects();
            }
            EntManeuver::MoveUnit => {
                self.pending_decision = Some(PendingDecision::CardUnitMovement {
                    player,
                    remaining: 1,
                });
            }
            EntManeuver::RemoveEnemyUnit => {
                self.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
                    player,
                    remaining: 1,
                });
            }
        }
    }
    pub(super) fn place_alliance_units(&mut self, region: Region) {
        let PendingDecision::AllianceUnitPlacement { player, remaining } = self
            .pending_decision
            .take()
            .expect("validated Alliance-token Unit placement")
        else {
            unreachable!("validated Alliance-token Unit action")
        };
        self.place_units_in_region(player, region, 1);
        if self.winner.is_none() && remaining > 1 && self.player(player).units_in_supply > 0 {
            self.pending_decision = Some(PendingDecision::AllianceUnitPlacement {
                player,
                remaining: remaining - 1,
            });
        } else {
            self.resolve_effects();
        }
    }
    pub(super) fn place_quest_bonus_unit(&mut self, region: Region) {
        let PendingDecision::QuestBonusUnitPlacement { player } = self
            .pending_decision
            .take()
            .expect("validated quest bonus choice")
        else {
            unreachable!("validated quest bonus unit action")
        };
        self.place_units_in_region(player, region, 1);
        self.resolve_effects();
    }
    pub(super) fn place_card_units(&mut self, region: Region) {
        let PendingDecision::CardUnitPlacement {
            player,
            amount,
            regions,
        } = self
            .pending_decision
            .take()
            .expect("validated red-card placement choice")
        else {
            unreachable!("validated red-card placement action")
        };
        debug_assert!(
            self.permitted_red_destinations(player, &regions)
                .contains(&region)
        );
        let placed = amount.min(self.player(player).units_in_supply);
        self.place_units_in_region(player, region, placed);
        self.resolve_effects();
    }
    pub(super) fn resolve_empty_card_unit_placement(&mut self) {
        let player = match self
            .pending_decision
            .take()
            .expect("validated empty Unit placement")
        {
            PendingDecision::CardUnitPlacement { player, .. }
            | PendingDecision::AllianceUnitPlacement { player, .. } => player,
            _ => unreachable!("validated empty Unit placement action"),
        };
        debug_assert_eq!(self.player(player).units_in_supply, 0);
        self.resolve_effects();
    }
    pub(super) fn move_card_unit(&mut self, from: Region, to: Region) {
        let PendingDecision::CardUnitMovement { player, remaining } = self
            .pending_decision
            .take()
            .expect("validated maneuver movement")
        else {
            unreachable!("validated maneuver movement action")
        };
        debug_assert!(
            self.legal_unit_moves(player)
                .contains(&Action::MoveCardUnit { from, to })
        );
        self.move_unit(player, from, to);
        if self.winner.is_none() && remaining > 1 {
            self.pending_decision = Some(PendingDecision::CardUnitMovement {
                player,
                remaining: remaining - 1,
            });
        } else {
            self.resolve_effects();
        }
    }
    pub(super) fn remove_card_enemy_unit(&mut self, region: Region) {
        let PendingDecision::CardEnemyUnitRemoval { player, remaining } = self
            .pending_decision
            .take()
            .expect("validated maneuver removal")
        else {
            unreachable!("validated maneuver removal action")
        };
        debug_assert!(self.region_unit_count(player.other(), region) > 0);
        let opponent = player.other();
        let target = self
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .expect("all regions are on the map");
        match opponent {
            Faction::Fellowship => target.fellowship_units -= 1,
            Faction::Sauron => target.sauron_units -= 1,
        }
        self.player_mut(opponent).units_in_supply += 1;
        self.check_map_domination();
        if self.winner.is_none() && remaining > 1 {
            self.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
                player,
                remaining: remaining - 1,
            });
        } else {
            self.resolve_effects();
        }
    }
    pub(super) fn resolve_unavailable_maneuver(&mut self) {
        match self
            .pending_decision
            .take()
            .expect("validated unavailable maneuver")
        {
            PendingDecision::CardUnitMovement { .. }
            | PendingDecision::CardEnemyUnitRemoval { .. } => self.resolve_effects(),
            _ => unreachable!("validated unavailable maneuver action"),
        }
    }
    pub(super) fn remove_quest_bonus_fortress(&mut self, region: Region) {
        let PendingDecision::QuestBonusFortressRemoval { player, .. } = self
            .pending_decision
            .take()
            .expect("validated quest bonus choice")
        else {
            unreachable!("validated quest bonus fortress action")
        };
        let fortress_owner = self
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .and_then(|state| state.fortress.take())
            .expect("validated enemy fortress action");
        debug_assert_eq!(fortress_owner, player.other());
        self.player_mut(fortress_owner).fortresses_in_supply += 1;
        self.resolve_effects();
    }
    pub(super) fn skip_quest_bonus(&mut self) {
        match self
            .pending_decision
            .take()
            .expect("validated quest bonus choice")
        {
            PendingDecision::CardUnitPlacement { .. }
            | PendingDecision::QuestBonusUnitPlacement { .. }
            | PendingDecision::QuestBonusFortressRemoval { .. }
            | PendingDecision::QuestBonusChoice { .. } => self.resolve_effects(),
            PendingDecision::CardDisposition { .. }
            | PendingDecision::CardUnitMovement { .. }
            | PendingDecision::CardEnemyUnitRemoval { .. }
            | PendingDecision::OpponentGreyCardDiscard { .. }
            | PendingDecision::DiscardedCardFreePlay { .. }
            | PendingDecision::AllianceTriggerChoice { .. }
            | PendingDecision::AllianceRaceChoice { .. }
            | PendingDecision::AllianceTokenChoice { .. }
            | PendingDecision::EntManeuverChoice { .. }
            | PendingDecision::AllianceUnitPlacement { .. } => {
                unreachable!("validated quest bonus action")
            }
        }
    }
    pub(super) fn accept_quest_bonus(&mut self) {
        let PendingDecision::QuestBonusChoice { player, effect } = self
            .pending_decision
            .take()
            .expect("validated quest bonus choice")
        else {
            unreachable!("validated quest bonus acceptance")
        };
        match effect {
            QuestBonusEffect::GainCoin => self.gain_coins(player, 1),
            QuestBonusEffect::TakeExtraTurn => self.extra_turns += 1,
            QuestBonusEffect::PlaceUnit | QuestBonusEffect::RemoveEnemyFortress => unreachable!(),
        }
        self.resolve_effects();
    }
    pub(super) fn queue_turn_completion(&mut self) {
        self.effect_queue
            .push_back(Effect::CheckImmediateVictoryConditions);
        self.effect_queue
            .push_back(Effect::RevealNewlyAvailableCards);
        self.effect_queue
            .push_back(Effect::TransitionChapterIfComplete);
        self.effect_queue.push_back(Effect::CheckPostRevealEffects);
        self.effect_queue.push_back(Effect::CompleteTurn {
            player: self.active_player,
        });
    }
}
