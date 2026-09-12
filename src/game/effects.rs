use crate::catalog::{alliance_token_definition, card_definition, landmark_definition};
use crate::{AllianceTrigger, PendingDecision, QuestBonusEffect, Region, alliance_schema};

use super::{Effect, Game};

impl Game {
    /// Runs automatic effects until either a choice is needed or the turn ends.
    pub(super) fn resolve_effects(&mut self) {
        while self.pending_decision.is_none() && !self.is_over() {
            let Some(effect) = self.effect_queue.pop_front() else {
                break;
            };
            #[cfg(test)]
            self.resolution_events.push((&effect).into());
            match effect {
                Effect::ResolvePlayedCard {
                    player,
                    card,
                    used_chain,
                } => {
                    let definition = card_definition(card);
                    for effect in self
                        .triggered_card_effects(player, definition.color, used_chain)
                        .into_iter()
                        .rev()
                    {
                        self.effect_queue
                            .push_front(Effect::ResolveAllianceEffect { player, effect });
                    }
                    if definition.color == crate::card_schema::CardColor::Green {
                        self.effect_queue
                            .push_front(Effect::ResolveRaceAllianceTriggers { player });
                    }
                    for effect in card_definition(card).effects.iter().rev() {
                        self.effect_queue.push_front(Effect::ResolveCardEffect {
                            player,
                            effect: effect.clone(),
                        });
                    }
                }
                Effect::ResolveLandmark { player, id } => {
                    for effect in self.landmark_triggered_effects(player).into_iter().rev() {
                        self.effect_queue
                            .push_front(Effect::ResolveAllianceEffect { player, effect });
                    }
                    for effect in landmark_definition(&id).effects.iter().rev() {
                        self.effect_queue.push_front(Effect::ResolveLandmarkEffect {
                            player,
                            effect: effect.clone(),
                        });
                    }
                    if self.player(player).landmark_grants_extra_turn {
                        self.extra_turns += 1;
                    }
                }
                Effect::ResolveLandmarkEffect { player, effect } => match effect {
                    crate::landmark_schema::LandmarkEffect::PlaceFortress { location } => {
                        self.place_fortress(player, location.into())
                    }
                    crate::landmark_schema::LandmarkEffect::GainCoins { amount } => {
                        self.gain_coins(player, amount)
                    }
                    crate::landmark_schema::LandmarkEffect::AdvanceQuest { spaces } => {
                        self.advance_quest(player, spaces)
                    }
                    crate::landmark_schema::LandmarkEffect::PlaceUnits {
                        amount, regions, ..
                    } => {
                        self.pending_decision = Some(PendingDecision::CardUnitPlacement {
                            player,
                            amount,
                            regions: regions.into_iter().map(Into::into).collect(),
                        });
                    }
                    crate::landmark_schema::LandmarkEffect::MoveUnits { amount } => {
                        self.pending_decision = Some(PendingDecision::CardUnitMovement {
                            player,
                            remaining: amount,
                        });
                    }
                    crate::landmark_schema::LandmarkEffect::DiscardOneGreyCardFromOpponent => {
                        if self.player(player.other()).tableau.iter().any(|card| {
                            card_definition(*card).color == crate::card_schema::CardColor::Grey
                        }) {
                            self.pending_decision =
                                Some(PendingDecision::OpponentGreyCardDiscard { player });
                        }
                    }
                    crate::landmark_schema::LandmarkEffect::PlayOneCardFromDiscardForFree => {
                        self.pending_decision =
                            Some(PendingDecision::DiscardedCardFreePlay { player });
                    }
                    crate::landmark_schema::LandmarkEffect::TwoOfAnyOneRaceTokens => {
                        self.pending_decision =
                            Some(PendingDecision::AllianceRaceChoice { player });
                    }
                },
                Effect::ResolveCardEffect { player, effect } => match effect {
                    crate::card_schema::Effect::GainCoins { amount } => {
                        self.gain_coins(player, amount);
                    }
                    crate::card_schema::Effect::AdvanceQuest { spaces } => {
                        self.advance_quest(player, spaces);
                    }
                    crate::card_schema::Effect::PlaceUnits {
                        amount, regions, ..
                    } => {
                        self.pending_decision = Some(PendingDecision::CardUnitPlacement {
                            player,
                            amount: amount + self.additional_red_units(player),
                            regions: regions.into_iter().map(Region::from).collect(),
                        });
                    }
                    crate::card_schema::Effect::MoveUnits { amount } => {
                        self.pending_decision = Some(PendingDecision::CardUnitMovement {
                            player,
                            remaining: amount,
                        });
                    }
                    crate::card_schema::Effect::RemoveEnemyUnits { amount } => {
                        self.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
                            player,
                            remaining: amount,
                        });
                    }
                    crate::card_schema::Effect::OpponentLosesCoins { amount } => {
                        self.lose_coins(player.other(), amount);
                    }
                },
                Effect::ResolveRaceAllianceTriggers { player } => {
                    self.check_race_victory(player);
                    if self.winner.is_some() {
                        continue;
                    }
                    let mut triggers = self.pending_alliance_triggers(player);
                    if triggers.len() == 1 {
                        let trigger = triggers.pop().expect("one trigger");
                        self.player_mut(player).matched_races.extend(match trigger {
                            AllianceTrigger::MatchingRace(race) => vec![race],
                            AllianceTrigger::ThreeDifferentRaces => Vec::new(),
                        });
                        match trigger {
                            AllianceTrigger::MatchingRace(race) => {
                                self.reveal_alliance_tokens(player, &[race], 2)
                            }
                            AllianceTrigger::ThreeDifferentRaces => {
                                self.player_mut(player).claimed_three_race_alliance = true;
                                let races = self.player_races(player);
                                self.reveal_alliance_tokens(player, &races[..3], 1);
                            }
                        }
                    } else if !triggers.is_empty() {
                        self.pending_decision =
                            Some(PendingDecision::AllianceTriggerChoice { player, triggers });
                    }
                }
                Effect::ResolveAllianceToken { player, token } => {
                    for effect in alliance_token_definition(token).immediate.iter().rev() {
                        self.effect_queue.push_front(Effect::ResolveAllianceEffect {
                            player,
                            effect: effect.clone(),
                        });
                    }
                }
                Effect::ResolveAllianceEffect { player, effect } => match effect {
                    alliance_schema::AllianceEffect::GainCoins { amount } => {
                        self.gain_coins(player, amount)
                    }
                    alliance_schema::AllianceEffect::AdvanceQuest { spaces } => {
                        self.advance_quest(player, spaces)
                    }
                    alliance_schema::AllianceEffect::PlaceUnitsAnywhere { amount } => {
                        self.pending_decision = Some(PendingDecision::AllianceUnitPlacement {
                            player,
                            remaining: amount,
                        });
                    }
                    alliance_schema::AllianceEffect::MoveUnits { amount } => {
                        self.pending_decision = Some(PendingDecision::CardUnitMovement {
                            player,
                            remaining: amount,
                        });
                    }
                    alliance_schema::AllianceEffect::RemoveEnemyUnits { amount } => {
                        self.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
                            player,
                            remaining: amount,
                        });
                    }
                    alliance_schema::AllianceEffect::OpponentLosesCoins { amount } => {
                        self.lose_coins(player.other(), amount)
                    }
                    alliance_schema::AllianceEffect::TakeExtraTurn => self.extra_turns += 1,
                    alliance_schema::AllianceEffect::RemoveEnemyFortress => {
                        self.pending_decision = Some(PendingDecision::QuestBonusFortressRemoval {
                            player,
                            optional: false,
                        });
                    }
                    alliance_schema::AllianceEffect::PlayDiscardedCardForFree => {
                        self.pending_decision =
                            Some(PendingDecision::DiscardedCardFreePlay { player });
                    }
                    alliance_schema::AllianceEffect::ChooseManeuvers { times } => {
                        self.effect_queue.push_front(Effect::ResolveEntManeuvers {
                            player,
                            remaining: times,
                        });
                    }
                },
                Effect::ResolveEntManeuvers { player, remaining } => {
                    self.pending_decision =
                        Some(PendingDecision::EntManeuverChoice { player, remaining });
                }
                Effect::ResolveQuestBonus { player, effect } => match effect {
                    QuestBonusEffect::GainCoin | QuestBonusEffect::TakeExtraTurn => {
                        self.pending_decision =
                            Some(PendingDecision::QuestBonusChoice { player, effect });
                    }
                    QuestBonusEffect::PlaceUnit => {
                        self.pending_decision =
                            Some(PendingDecision::QuestBonusUnitPlacement { player });
                    }
                    QuestBonusEffect::RemoveEnemyFortress => {
                        self.pending_decision = Some(PendingDecision::QuestBonusFortressRemoval {
                            player,
                            optional: true,
                        });
                    }
                },
                Effect::TransitionChapterIfComplete => self.transition_chapter_if_complete(),
                Effect::CheckImmediateVictoryConditions | Effect::CheckPostRevealEffects => {
                    // Victory rules are introduced with their corresponding effects.
                }
                Effect::RevealNewlyAvailableCards => self.reveal_newly_available_cards(),
                Effect::CompleteTurn { player } => {
                    debug_assert_eq!(player, self.active_player);
                    self.turn += 1;
                    self.player_mut(player).used_any_skill_this_turn = false;
                    if self.extra_turns > 0 {
                        self.extra_turns -= 1;
                    } else {
                        self.active_player = self.active_player.other();
                    }
                }
            }
        }
    }
}
