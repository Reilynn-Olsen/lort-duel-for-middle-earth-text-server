use crate::catalog::card_definition;
use crate::{Action, EntManeuver, Faction, GameError, PendingDecision, REGIONS};

use super::Game;

impl Game {
    pub(super) fn legal_actions(&self) -> Vec<Action> {
        if self.is_over() {
            Vec::new()
        } else if let Some(decision) = &self.pending_decision {
            match decision {
                PendingDecision::CardDisposition { card } => {
                    let mut actions = vec![Action::DiscardSelectedCard];
                    if self.uses_chain(self.active_player, *card) {
                        actions.insert(0, Action::PlaySelectedCard);
                        if self.normal_payment_for(*card).is_some() {
                            actions.insert(1, Action::PlaySelectedCardNormally);
                        }
                    } else if self.normal_payment_for(*card).is_some() {
                        actions.insert(0, Action::PlaySelectedCard);
                    }
                    if self.payment_for_using_any_skill(*card).is_some() {
                        actions.insert(0, Action::PlaySelectedCardUsingAnySkill);
                    }
                    actions
                }
                PendingDecision::QuestBonusUnitPlacement { player } => {
                    let mut actions = if self.player(*player).units_in_supply > 0 {
                        REGIONS
                            .into_iter()
                            .map(|region| Action::PlaceQuestBonusUnit { region })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    actions.push(Action::SkipQuestBonus);
                    actions
                }
                PendingDecision::CardUnitPlacement {
                    player,
                    amount: _,
                    regions,
                } => {
                    let permitted = self.permitted_red_destinations(*player, regions);
                    if self.player(*player).units_in_supply == 0 {
                        vec![Action::ResolveEmptyCardUnitPlacement]
                    } else {
                        permitted
                            .into_iter()
                            .map(|region| Action::PlaceCardUnits { region })
                            .collect()
                    }
                }
                PendingDecision::CardUnitMovement { player, .. } => {
                    let actions = self.legal_unit_moves(*player);
                    if actions.is_empty() {
                        vec![Action::ResolveUnavailableManeuver]
                    } else {
                        actions
                    }
                }
                PendingDecision::CardEnemyUnitRemoval { player, .. } => {
                    let actions = self
                        .map
                        .iter()
                        .filter(|state| self.region_unit_count(player.other(), state.region) > 0)
                        .map(|state| Action::RemoveCardEnemyUnit {
                            region: state.region,
                        })
                        .collect::<Vec<_>>();
                    if actions.is_empty() {
                        vec![Action::ResolveUnavailableManeuver]
                    } else {
                        actions
                    }
                }
                PendingDecision::QuestBonusFortressRemoval { player, optional } => {
                    let mut actions = self
                        .map
                        .iter()
                        .filter(|region| region.fortress == Some(player.other()))
                        .map(|region| Action::RemoveQuestBonusFortress {
                            region: region.region,
                        })
                        .collect::<Vec<_>>();
                    if *optional {
                        actions.push(Action::SkipQuestBonus);
                    }
                    actions
                }
                PendingDecision::QuestBonusChoice { .. } => {
                    vec![Action::AcceptQuestBonus, Action::SkipQuestBonus]
                }
                PendingDecision::OpponentGreyCardDiscard { player } => self
                    .player((*player).other())
                    .tableau
                    .iter()
                    .copied()
                    .filter(|card| {
                        card_definition(*card).color == crate::card_schema::CardColor::Grey
                    })
                    .map(|card| Action::DiscardOpponentGreyCard { card })
                    .collect(),
                PendingDecision::DiscardedCardFreePlay { .. } => self
                    .discarded_cards()
                    .into_iter()
                    .map(|card| Action::PlayDiscardedCardForFree { card })
                    .collect(),
                PendingDecision::AllianceTriggerChoice { triggers, .. } => triggers
                    .iter()
                    .copied()
                    .map(|trigger| Action::ResolveAllianceTrigger { trigger })
                    .collect(),
                PendingDecision::AllianceRaceChoice { .. } => self
                    .alliance_stacks
                    .iter()
                    .filter(|stack| !stack.token_order.is_empty())
                    .map(|stack| Action::ChooseAllianceRace { race: stack.race })
                    .collect(),
                PendingDecision::AllianceTokenChoice { revealed, .. } => revealed
                    .iter()
                    .copied()
                    .map(|token| Action::TakeAllianceToken { token })
                    .collect(),
                PendingDecision::EntManeuverChoice { .. } => vec![
                    Action::ChooseEntManeuver {
                        maneuver: EntManeuver::RemoveEnemyUnit,
                    },
                    Action::ChooseEntManeuver {
                        maneuver: EntManeuver::OpponentLosesCoin,
                    },
                    Action::ChooseEntManeuver {
                        maneuver: EntManeuver::MoveUnit,
                    },
                ],
                PendingDecision::AllianceUnitPlacement { player, .. } => {
                    if self.player(*player).units_in_supply == 0 {
                        vec![Action::ResolveEmptyCardUnitPlacement]
                    } else {
                        REGIONS
                            .into_iter()
                            .map(|region| Action::PlaceAllianceUnits { region })
                            .collect()
                    }
                }
            }
        } else {
            let mut actions = self
                .available_chapter_slot_ids()
                .into_iter()
                .map(|slot_id| Action::TakeChapterCard { slot_id })
                .collect::<Vec<_>>();
            actions.extend(
                self.landmarks
                    .available
                    .iter()
                    .filter(|id| {
                        self.player(self.active_player).fortresses_in_supply > 0
                            && self.landmark_payment(id).is_some()
                    })
                    .cloned()
                    .map(|id| Action::TakeLandmark { id }),
            );
            actions.extend(
                self.landmarks
                    .available
                    .iter()
                    .filter(|id| {
                        self.player(self.active_player).fortresses_in_supply > 0
                            && self.landmark_payment_using_any_skill(id).is_some()
                    })
                    .cloned()
                    .map(|id| Action::TakeLandmarkUsingAnySkill { id }),
            );
            actions
        }
    }
    pub fn legal_actions_for(&self, viewer: Faction) -> Vec<Action> {
        if self
            .decision_player()
            .is_some_and(|player| player != viewer)
            || (self.pending_decision.is_none() && self.active_player != viewer)
        {
            Vec::new()
        } else {
            self.legal_actions()
        }
    }
    pub fn apply_for(&mut self, viewer: Faction, action: Action) -> Result<(), GameError> {
        if !self.legal_actions_for(viewer).contains(&action) {
            return Err(GameError::IllegalAction(action));
        }
        self.apply(action)
    }
    pub(super) fn decision_player(&self) -> Option<Faction> {
        match self.pending_decision.as_ref()? {
            PendingDecision::CardDisposition { .. } => Some(self.active_player),
            PendingDecision::CardUnitPlacement { player, .. }
            | PendingDecision::CardUnitMovement { player, .. }
            | PendingDecision::CardEnemyUnitRemoval { player, .. }
            | PendingDecision::QuestBonusUnitPlacement { player }
            | PendingDecision::QuestBonusFortressRemoval { player, .. }
            | PendingDecision::QuestBonusChoice { player, .. }
            | PendingDecision::OpponentGreyCardDiscard { player }
            | PendingDecision::DiscardedCardFreePlay { player }
            | PendingDecision::AllianceTriggerChoice { player, .. }
            | PendingDecision::AllianceRaceChoice { player }
            | PendingDecision::AllianceTokenChoice { player, .. }
            | PendingDecision::EntManeuverChoice { player, .. }
            | PendingDecision::AllianceUnitPlacement { player, .. } => Some(*player),
        }
    }
    pub(super) fn apply(&mut self, action: Action) -> Result<(), GameError> {
        if self.is_over() {
            return Err(GameError::GameOver);
        }
        if !self.legal_actions().contains(&action) {
            return Err(GameError::IllegalAction(action));
        }
        match action {
            Action::TakeChapterCard { slot_id } => self.take_chapter_card(slot_id),
            Action::TakeLandmark { id } => self.take_landmark(id),
            Action::TakeLandmarkUsingAnySkill { id } => self.take_landmark_using_any_skill(id),
            Action::PlaySelectedCard => self.play_selected_card(),
            Action::PlaySelectedCardNormally => self.play_selected_card_normally(),
            Action::PlaySelectedCardUsingAnySkill => self.play_selected_card_using_any_skill(),
            Action::DiscardSelectedCard => self.discard_selected_card(),
            Action::PlaceCardUnits { region } => self.place_card_units(region),
            Action::ResolveEmptyCardUnitPlacement => self.resolve_empty_card_unit_placement(),
            Action::MoveCardUnit { from, to } => self.move_card_unit(from, to),
            Action::RemoveCardEnemyUnit { region } => self.remove_card_enemy_unit(region),
            Action::ResolveUnavailableManeuver => self.resolve_unavailable_maneuver(),
            Action::PlaceQuestBonusUnit { region } => self.place_quest_bonus_unit(region),
            Action::RemoveQuestBonusFortress { region } => self.remove_quest_bonus_fortress(region),
            Action::DiscardOpponentGreyCard { card } => self.discard_opponent_grey_card(card),
            Action::PlayDiscardedCardForFree { card } => self.play_discarded_card_for_free(card),
            Action::ResolveAllianceTrigger { trigger } => self.resolve_alliance_trigger(trigger),
            Action::ChooseAllianceRace { race } => self.choose_alliance_race(race),
            Action::TakeAllianceToken { token } => self.take_alliance_token(token),
            Action::ChooseEntManeuver { maneuver } => self.choose_ent_maneuver(maneuver),
            Action::PlaceAllianceUnits { region } => self.place_alliance_units(region),
            Action::AcceptQuestBonus => self.accept_quest_bonus(),
            Action::SkipQuestBonus => self.skip_quest_bonus(),
        }
        Ok(())
    }
}
