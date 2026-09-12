use crate::card_schema::Skill;
use crate::catalog::{card_definition, chapter_cards, setup_chapter};
use crate::*;

use super::{Effect, Game};

/// Test-visible record of each automatic resolver step, in execution order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResolutionEvent {
    PlayedCard {
        player: Faction,
        card: ChapterCard,
    },
    Landmark {
        player: Faction,
        id: String,
    },
    LandmarkEffect {
        player: Faction,
    },
    RaceAllianceTriggers {
        player: Faction,
    },
    AllianceToken {
        player: Faction,
        token: AllianceToken,
    },
    AllianceEffect {
        player: Faction,
    },
    EntManeuvers {
        player: Faction,
        remaining: u8,
    },
    CardEffect {
        player: Faction,
    },
    QuestBonus {
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

impl From<&Effect> for ResolutionEvent {
    fn from(effect: &Effect) -> Self {
        match effect {
            Effect::ResolvePlayedCard { player, card, .. } => Self::PlayedCard {
                player: *player,
                card: *card,
            },
            Effect::ResolveLandmark { player, id } => Self::Landmark {
                player: *player,
                id: id.clone(),
            },
            Effect::ResolveLandmarkEffect { player, .. } => {
                Self::LandmarkEffect { player: *player }
            }
            Effect::ResolveRaceAllianceTriggers { player } => {
                Self::RaceAllianceTriggers { player: *player }
            }
            Effect::ResolveAllianceToken { player, token } => Self::AllianceToken {
                player: *player,
                token: *token,
            },
            Effect::ResolveAllianceEffect { player, .. } => {
                Self::AllianceEffect { player: *player }
            }
            Effect::ResolveEntManeuvers { player, remaining } => Self::EntManeuvers {
                player: *player,
                remaining: *remaining,
            },
            Effect::ResolveCardEffect { player, .. } => Self::CardEffect { player: *player },
            Effect::ResolveQuestBonus { player, effect } => Self::QuestBonus {
                player: *player,
                effect: *effect,
            },
            Effect::CheckImmediateVictoryConditions => Self::CheckImmediateVictoryConditions,
            Effect::RevealNewlyAvailableCards => Self::RevealNewlyAvailableCards,
            Effect::CheckPostRevealEffects => Self::CheckPostRevealEffects,
            Effect::TransitionChapterIfComplete => Self::TransitionChapterIfComplete,
            Effect::CompleteTurn { player } => Self::CompleteTurn { player: *player },
        }
    }
}

/// Builds a focused, internally consistent game state for one transition test.
pub(super) struct GameTest {
    game: Game,
}

impl GameTest {
    pub(super) fn new() -> Self {
        Self { game: Game::new(0) }
    }

    pub(super) fn active_player(mut self, player: Faction) -> Self {
        self.game.active_player = player;
        self
    }

    pub(super) fn chapter(mut self, chapter: u8) -> Self {
        assert!(
            (1..=3).contains(&chapter),
            "chapter must be between 1 and 3"
        );
        while self.game.chapters.len() < chapter as usize {
            let next = self.game.chapters.len() as u8 + 1;
            self.game
                .chapters
                .push(setup_chapter(next, &mut self.game.rng));
        }
        self.game.chapters.truncate(chapter as usize);
        self.game.current_chapter = chapter;
        self
    }

    pub(super) fn coins(mut self, player: Faction, coins: u8) -> Self {
        assert!(coins <= TOTAL_COINS - self.game.player(player.other()).coins);
        self.game.player_mut(player).coins = coins;
        self.game.coin_reserve = TOTAL_COINS - self.game.fellowship.coins - self.game.sauron.coins;
        self
    }

    pub(super) fn cards(
        mut self,
        player: Faction,
        cards: impl IntoIterator<Item = ChapterCard>,
    ) -> Self {
        self.game.player_mut(player).tableau = cards.into_iter().collect();
        self
    }

    pub(super) fn skills(
        mut self,
        player: Faction,
        skills: impl IntoIterator<Item = Skill>,
    ) -> Self {
        let cards = skills
            .into_iter()
            .map(|skill| card_for(|definition| definition.provides.skills.contains_key(&skill)))
            .collect::<Vec<_>>();
        self.game.player_mut(player).tableau.extend(cards);
        self
    }

    pub(super) fn races(mut self, player: Faction, races: impl IntoIterator<Item = Race>) -> Self {
        let cards = races
            .into_iter()
            .map(|race| card_for(|definition| definition.provides.race == Some(race.into())))
            .collect::<Vec<_>>();
        self.game.player_mut(player).tableau.extend(cards);
        self
    }

    pub(super) fn alliance_tokens(
        mut self,
        player: Faction,
        tokens: impl IntoIterator<Item = AllianceToken>,
    ) -> Self {
        self.game.player_mut(player).alliance_tokens = tokens.into_iter().collect();
        self
    }

    pub(super) fn units(mut self, player: Faction, region: Region, amount: u8) -> Self {
        let state = self
            .game
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap();
        match player {
            Faction::Fellowship => state.fellowship_units = amount,
            Faction::Sauron => state.sauron_units = amount,
        }
        let deployed = self
            .game
            .map
            .iter()
            .map(|state| match player {
                Faction::Fellowship => state.fellowship_units,
                Faction::Sauron => state.sauron_units,
            })
            .sum::<u8>();
        self.game.player_mut(player).units_in_supply = 13 - deployed;
        self
    }

    pub(super) fn fortress(mut self, player: Faction, region: Region) -> Self {
        self.game
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(player);
        let deployed = self
            .game
            .map
            .iter()
            .filter(|state| state.fortress == Some(player))
            .count() as u8;
        self.game.player_mut(player).fortresses_in_supply = 7 - deployed;
        self
    }

    pub(super) fn quest_positions(mut self, fellowship: u8, sauron: u8) -> Self {
        self.game.quest.fellowship_position = fellowship;
        self.game.quest.sauron_position = sauron;
        self
    }

    pub(super) fn claimed_bonus(mut self, position: u8) -> Self {
        self.game
            .quest
            .bonuses
            .iter_mut()
            .find(|bonus| bonus.position == position)
            .unwrap()
            .claimed = true;
        self
    }

    /// Replaces the chapter display with face-up, uncovered cards in slot order.
    pub(super) fn display(mut self, cards: impl IntoIterator<Item = ChapterCard>) -> Self {
        let chapter = self.game.current_chapter;
        self.game.chapters[(chapter - 1) as usize].layout = cards
            .into_iter()
            .enumerate()
            .map(|(id, card)| CardSlot {
                id,
                row: 0,
                column: id as u8,
                card,
                visibility: CardVisibility::FaceUp,
                covering_slots: Vec::new(),
            })
            .collect();
        self
    }

    pub(super) fn landmarks(mut self, ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.game.landmarks.available = ids.into_iter().map(Into::into).collect();
        self
    }

    pub(super) fn build(self) -> Game {
        self.game
    }
}

impl Game {
    /// Runs one legal action and starts a fresh resolution-event trace.
    pub(super) fn run_action(&mut self, action: Action) -> Result<(), GameError> {
        let previous_events = std::mem::take(&mut self.resolution_events);
        match self.apply(action) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.resolution_events = previous_events;
                Err(error)
            }
        }
    }

    /// Resolves the current pending choice with one legal choice action.
    pub(super) fn resolve_choice(&mut self, action: Action) -> Result<(), GameError> {
        assert!(
            self.pending_decision.is_some(),
            "no pending choice to resolve"
        );
        self.run_action(action)
    }

    pub(super) fn resolution_events(&self) -> &[ResolutionEvent] {
        &self.resolution_events
    }
}

pub(super) fn assert_illegal_action_unchanged(game: &mut Game, action: Action) {
    let before = game.clone();
    assert_eq!(
        game.run_action(action.clone()),
        Err(GameError::IllegalAction(action))
    );
    assert_game_state_eq(game, &before);
}

pub(super) fn assert_game_state_eq(actual: &Game, expected: &Game) {
    assert_eq!(
        actual,
        expected,
        "game states differ\nactual:\n{}\nexpected:\n{}",
        actual.render_state(),
        expected.render_state()
    );
}

fn card_for(predicate: impl Fn(&crate::card_schema::CardDefinition) -> bool) -> ChapterCard {
    (1..=3)
        .flat_map(chapter_cards)
        .find(|card| predicate(card_definition(*card)))
        .expect("catalogue contains requested test fixture card")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(chapter: u8, number: u8) -> ChapterCard {
        ChapterCard {
            chapter,
            number,
            copy: 0,
        }
    }

    #[test]
    fn fixture_builds_a_controlled_minimal_state() {
        let game = GameTest::new()
            .active_player(Faction::Fellowship)
            .chapter(2)
            .coins(Faction::Fellowship, 7)
            .cards(Faction::Sauron, [card(1, 9)])
            .skills(Faction::Fellowship, [Skill::Strength])
            .races(Faction::Fellowship, [Race::Elves])
            .alliance_tokens(
                Faction::Fellowship,
                [AllianceToken {
                    race: Race::Dwarves,
                    id: 1,
                }],
            )
            .units(Faction::Fellowship, Region::Lindon, 3)
            .fortress(Faction::Sauron, Region::Gondor)
            .quest_positions(8, 4)
            .claimed_bonus(7)
            .display([card(2, 1)])
            .landmarks(["helms_deep"])
            .build();

        assert_eq!(game.active_player(), Faction::Fellowship);
        assert_eq!(game.current_chapter, 2);
        assert_eq!(game.fellowship.coins, 7);
        assert_eq!(game.sauron.tableau, vec![card(1, 9)]);
        assert!(game.fellowship.tableau.iter().any(|card| {
            card_definition(*card)
                .provides
                .skills
                .contains_key(&Skill::Strength)
        }));
        assert_eq!(game.player_races(Faction::Fellowship), vec![Race::Elves]);
        assert_eq!(game.fellowship.alliance_tokens.len(), 1);
        assert_eq!(
            game.region_unit_count(Faction::Fellowship, Region::Lindon),
            3
        );
        assert_eq!(game.map[5].fortress, Some(Faction::Sauron));
        assert_eq!(
            (game.quest.fellowship_position, game.quest.sauron_position),
            (8, 4)
        );
        assert!(
            game.quest
                .bonuses
                .iter()
                .find(|bonus| bonus.position == 7)
                .unwrap()
                .claimed
        );
        assert_eq!(game.available_chapter_slot_ids(), vec![0]);
        assert_eq!(game.landmarks.available, vec!["helms_deep"]);
    }

    #[test]
    fn fixture_runs_actions_resolves_choices_and_records_events() {
        let mut game = GameTest::new()
            .active_player(Faction::Sauron)
            .display([card(1, 9)])
            .build();

        game.run_action(Action::TakeChapterCard { slot_id: 0 })
            .unwrap();
        assert!(matches!(
            game.pending_decision,
            Some(PendingDecision::CardDisposition { .. })
        ));
        game.resolve_choice(Action::PlaySelectedCard).unwrap();

        assert_eq!(game.sauron.coins, 4);
        assert_eq!(
            game.resolution_events(),
            [
                ResolutionEvent::PlayedCard {
                    player: Faction::Sauron,
                    card: card(1, 9),
                },
                ResolutionEvent::CardEffect {
                    player: Faction::Sauron,
                },
                ResolutionEvent::CheckImmediateVictoryConditions,
                ResolutionEvent::RevealNewlyAvailableCards,
                ResolutionEvent::TransitionChapterIfComplete,
                ResolutionEvent::CheckPostRevealEffects,
                ResolutionEvent::CompleteTurn {
                    player: Faction::Sauron,
                },
            ]
        );
    }

    #[test]
    fn fixture_asserts_illegal_actions_leave_state_unchanged() {
        let mut game = GameTest::new().display([card(1, 9)]).build();
        assert_illegal_action_unchanged(&mut game, Action::TakeChapterCard { slot_id: 99 });
    }
}
