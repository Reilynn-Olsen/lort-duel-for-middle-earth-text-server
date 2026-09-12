use std::collections::VecDeque;

use crate::catalog::{DeterministicRng, landmark_ids, setup_chapter};
use crate::{
    AllianceStack, AllianceToken, CardSlot, CardVisibility, ChapterCard, ChapterState, Faction,
    LandmarkState, PendingDecision, PlayerState, QuestBonusEffect, QuestCharacter, QuestTrack,
    RACES, REGIONS, Region, RegionState, TOTAL_COINS, VictoryType, alliance_schema, quest_bonuses,
};

mod actions;
#[cfg(test)]
mod alliance_tests;
mod board;
#[cfg(test)]
mod board_tests;
#[cfg(test)]
mod card_tests;
#[cfg(test)]
mod chapter_tests;
mod choices;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod display_tests;
mod effects;
#[cfg(test)]
mod invariant_tests;
#[cfg(test)]
mod landmark_tests;
#[cfg(test)]
mod payment_tests;
mod payments;
#[cfg(test)]
mod quest_alliance_tests;
mod render;
#[cfg(test)]
mod setup_tests;
#[cfg(test)]
mod test_support;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Effect {
    ResolvePlayedCard {
        player: Faction,
        card: ChapterCard,
        used_chain: bool,
    },
    ResolveLandmark {
        player: Faction,
        id: String,
    },
    ResolveLandmarkEffect {
        player: Faction,
        effect: crate::landmark_schema::LandmarkEffect,
    },
    ResolveRaceAllianceTriggers {
        player: Faction,
    },
    ResolveAllianceToken {
        player: Faction,
        token: AllianceToken,
    },
    ResolveAllianceEffect {
        player: Faction,
        effect: alliance_schema::AllianceEffect,
    },
    ResolveEntManeuvers {
        player: Faction,
        remaining: u8,
    },
    ResolveCardEffect {
        player: Faction,
        effect: crate::card_schema::Effect,
    },
    ResolveQuestBonus {
        player: Faction,
        effect: QuestBonusEffect,
    },
    CheckImmediateVictoryConditions,
    RevealNewlyAvailableCards,
    CheckPostRevealEffects,
    TransitionChapterIfComplete,
    CompleteTurn {
        player: Faction,
    },
}

/// Complete authoritative game state. `new(seed)` constructs every random setup
/// element deterministically, including each independently shuffled stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    seed: u64,
    rng: DeterministicRng,
    active_player: Faction,
    turn: u32,
    winner: Option<Faction>,
    victory_type: Option<VictoryType>,
    shared_territory_victory: bool,
    fellowship: PlayerState,
    sauron: PlayerState,
    coin_reserve: u8,
    map: Vec<RegionState>,
    quest: QuestTrack,
    alliance_stacks: Vec<AllianceStack>,
    landmarks: LandmarkState,
    chapters: Vec<ChapterState>,
    current_chapter: u8,
    pending_decision: Option<PendingDecision>,
    effect_queue: VecDeque<Effect>,
    extra_turns: u8,
    #[cfg(test)]
    resolution_events: Vec<test_support::ResolutionEvent>,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut rng = DeterministicRng::new(seed);
        let mut alliance_stacks = RACES
            .into_iter()
            .map(|race| {
                let mut token_order = vec![1, 2, 3];
                rng.shuffle(&mut token_order);
                AllianceStack { race, token_order }
            })
            .collect::<Vec<_>>();
        // Preserve a canonical race order in rendered state; only contents are shuffled.
        alliance_stacks.sort_by_key(|stack| RACES.iter().position(|race| *race == stack.race));
        let mut landmark_order = landmark_ids();
        rng.shuffle(&mut landmark_order);
        let chapters = vec![setup_chapter(1, &mut rng)];
        Self {
            seed,
            rng,
            active_player: Faction::Sauron,
            turn: 0,
            winner: None,
            victory_type: None,
            shared_territory_victory: false,
            fellowship: PlayerState {
                faction: Faction::Fellowship,
                quest_character: QuestCharacter::FrodoAndSam,
                coins: 3,
                units_in_supply: 13,
                fortresses_in_supply: 7,
                tableau: Vec::new(),
                landmarks: Vec::new(),
                alliance_tokens: Vec::new(),
                matched_races: Vec::new(),
                claimed_three_race_alliance: false,
                waives_landmark_fortress_cost: false,
                landmark_grants_extra_turn: false,
                used_any_skill_this_turn: false,
            },
            sauron: PlayerState {
                faction: Faction::Sauron,
                quest_character: QuestCharacter::Nazgul,
                coins: 2,
                units_in_supply: 13,
                fortresses_in_supply: 7,
                tableau: Vec::new(),
                landmarks: Vec::new(),
                alliance_tokens: Vec::new(),
                matched_races: Vec::new(),
                claimed_three_race_alliance: false,
                waives_landmark_fortress_cost: false,
                landmark_grants_extra_turn: false,
                used_any_skill_this_turn: false,
            },
            coin_reserve: TOTAL_COINS - 5,
            map: REGIONS
                .into_iter()
                .map(|region| RegionState {
                    region,
                    fellowship_units: u8::from(region == Region::Arnor) * 2,
                    sauron_units: u8::from(region == Region::Mordor) * 2,
                    fortress: None,
                })
                .collect(),
            quest: QuestTrack {
                fellowship_position: 0,
                sauron_position: 0,
                bonuses: quest_bonuses(),
            },
            alliance_stacks,
            landmarks: LandmarkState {
                available: landmark_order[..3].to_vec(),
                facedown_deck: landmark_order[3..].to_vec(),
            },
            chapters,
            current_chapter: 1,
            pending_decision: None,
            effect_queue: VecDeque::new(),
            extra_turns: 0,
            #[cfg(test)]
            resolution_events: Vec::new(),
        }
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn active_player(&self) -> Faction {
        self.active_player
    }
    pub fn turn(&self) -> u32 {
        self.turn
    }
    pub fn winner(&self) -> Option<Faction> {
        self.winner
    }
    pub fn victory_type(&self) -> Option<VictoryType> {
        self.victory_type
    }
    pub fn is_shared_victory(&self) -> bool {
        self.shared_territory_victory
    }
    pub fn is_over(&self) -> bool {
        self.winner.is_some() || self.shared_territory_victory
    }
}

impl Game {
    pub(super) fn current_chapter_state(&self) -> &ChapterState {
        &self.chapters[(self.current_chapter - 1) as usize]
    }
    /// The physical discard stacks are stored with their chapters, but all are
    /// one shared discard pile for effects that select a discarded card.
    pub(super) fn discarded_cards(&self) -> Vec<ChapterCard> {
        self.chapters
            .iter()
            .flat_map(|chapter| chapter.discarded.iter().copied())
            .collect()
    }
    pub(super) fn remove_discarded_card(&mut self, card: ChapterCard) -> bool {
        self.chapters.iter_mut().any(|chapter| {
            let Some(position) = chapter
                .discarded
                .iter()
                .position(|discarded| *discarded == card)
            else {
                return false;
            };
            chapter.discarded.remove(position);
            true
        })
    }
    pub(super) fn discard_card(&mut self, card: ChapterCard) {
        self.chapters[(card.chapter - 1) as usize]
            .discarded
            .push(card);
    }
    /// Slot IDs for the only chapter cards the active player may take.
    pub fn available_chapter_slot_ids(&self) -> Vec<usize> {
        self.current_chapter_state()
            .layout
            .iter()
            .filter(|slot| self.card_is_available(slot))
            .map(|slot| slot.id)
            .collect()
    }
    pub(super) fn card_is_available(&self, slot: &CardSlot) -> bool {
        slot.visibility == CardVisibility::FaceUp
            && slot.covering_slots.iter().all(|covering_id| {
                !self
                    .current_chapter_state()
                    .layout
                    .iter()
                    .any(|candidate| candidate.id == *covering_id)
            })
    }
    pub(super) fn take_chapter_card(&mut self, slot_id: usize) {
        let chapter = &mut self.chapters[(self.current_chapter - 1) as usize];
        let position = chapter
            .layout
            .iter()
            .position(|slot| slot.id == slot_id)
            .expect("validated legal slot");
        let card = chapter.layout.remove(position).card;
        self.pending_decision = Some(PendingDecision::CardDisposition { card });
    }
    pub(super) fn take_landmark(&mut self, id: String) {
        self.take_landmark_with_payment(id, false);
    }
    pub(super) fn take_landmark_using_any_skill(&mut self, id: String) {
        self.take_landmark_with_payment(id, true);
    }
    pub(super) fn take_landmark_with_payment(&mut self, id: String, using_any_skill: bool) {
        let payment = if using_any_skill {
            self.landmark_payment_using_any_skill(&id)
        } else {
            self.landmark_payment(&id)
        }
        .expect("validated affordable Landmark");
        let index = self
            .landmarks
            .available
            .iter()
            .position(|available| available == &id)
            .expect("validated Landmark");
        self.landmarks.available.remove(index);
        let player = self.active_player;
        if using_any_skill {
            self.player_mut(player).used_any_skill_this_turn = true;
        }
        self.player_mut(player).coins -= payment;
        self.coin_reserve += payment;
        self.player_mut(player).landmarks.push(id.clone());
        self.effect_queue
            .push_back(Effect::ResolveLandmark { player, id });
        self.queue_turn_completion();
        self.resolve_effects();
    }
    pub(super) fn play_selected_card(&mut self) {
        let card = match self.pending_decision {
            Some(PendingDecision::CardDisposition { card }) => card,
            _ => unreachable!("validated card disposition action"),
        };
        self.play_selected_card_with_payment(false, self.uses_chain(self.active_player, card));
    }
    pub(super) fn play_selected_card_normally(&mut self) {
        self.play_selected_card_with_payment(false, false);
    }
    pub(super) fn play_selected_card_using_any_skill(&mut self) {
        self.play_selected_card_with_payment(true, false);
    }
    pub(super) fn play_selected_card_with_payment(
        &mut self,
        using_any_skill: bool,
        use_chain: bool,
    ) {
        let PendingDecision::CardDisposition { card } = self
            .pending_decision
            .take()
            .expect("validated pending choice")
        else {
            unreachable!("validated card disposition action")
        };
        let payment = if use_chain {
            Some(0)
        } else if using_any_skill {
            self.payment_for_using_any_skill(card)
        } else {
            self.normal_payment_for(card)
        }
        .expect("validated affordable card");
        let active_player = self.active_player;
        {
            let player = self.player_mut(active_player);
            player.coins -= payment;
            player.tableau.push(card);
            if using_any_skill {
                player.used_any_skill_this_turn = true;
            }
        }
        self.coin_reserve += payment;
        self.effect_queue.push_back(Effect::ResolvePlayedCard {
            player: active_player,
            card,
            used_chain: use_chain,
        });
        self.queue_turn_completion();
        self.resolve_effects();
    }
    pub(super) fn player(&self, faction: Faction) -> &PlayerState {
        match faction {
            Faction::Fellowship => &self.fellowship,
            Faction::Sauron => &self.sauron,
        }
    }
    pub(super) fn player_mut(&mut self, faction: Faction) -> &mut PlayerState {
        match faction {
            Faction::Fellowship => &mut self.fellowship,
            Faction::Sauron => &mut self.sauron,
        }
    }
}
