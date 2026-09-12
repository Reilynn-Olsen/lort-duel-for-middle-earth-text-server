use crate::*;

use super::*;

#[test]
fn final_chapter_card_advances_to_the_next_stored_layout_and_refills_landmarks() {
    let mut game = Game::new(0);
    game.chapters[0].layout.clear();
    game.landmarks.available.clear();
    game.queue_turn_completion();
    game.resolve_effects();

    assert_eq!(game.current_chapter, 2);
    assert_eq!(game.chapters[1].layout.len(), CARDS_IN_LAYOUT);
    assert_eq!(game.landmarks.available.len(), 3);
    assert_eq!(game.landmarks.facedown_deck.len(), 1);
}
#[test]
fn chapter_transition_preserves_normal_alternation_and_extra_turns() {
    let mut normal_turn = Game::new(0);
    normal_turn.chapters[0].layout.clear();
    normal_turn.queue_turn_completion();
    normal_turn.resolve_effects();
    assert_eq!(normal_turn.current_chapter, 2);
    assert_eq!(normal_turn.active_player(), Faction::Fellowship);

    let mut extra_turn = Game::new(0);
    extra_turn.chapters[0].layout.clear();
    extra_turn.extra_turns = 1;
    extra_turn.queue_turn_completion();
    extra_turn.resolve_effects();
    assert_eq!(extra_turn.current_chapter, 2);
    assert_eq!(extra_turn.active_player(), Faction::Sauron);
    assert_eq!(extra_turn.extra_turns, 0);
}
#[test]
fn chapter_does_not_transition_while_any_card_remains_in_its_layout() {
    let mut game = Game::new(0);
    for slot in &mut game.chapters[0].layout {
        slot.visibility = CardVisibility::FaceDown;
    }
    game.effect_queue
        .push_back(Effect::TransitionChapterIfComplete);
    game.resolve_effects();

    assert_eq!(game.current_chapter, 1);
}
#[test]
fn final_chapter_uses_territorial_comparison_and_can_share_the_victory() {
    let mut shared = Game::new(0);
    shared.chapters[0].layout.clear();
    shared.transition_chapter_if_complete();
    shared.chapters[1].layout.clear();
    shared.transition_chapter_if_complete();
    shared.chapters[2].layout.clear();
    shared
        .effect_queue
        .push_back(Effect::TransitionChapterIfComplete);
    shared.resolve_effects();
    assert!(shared.is_over());
    assert!(shared.is_shared_victory());
    assert_eq!(
        shared.victory_type(),
        Some(VictoryType::SharedTerritorialComparison)
    );

    let mut fellowship = Game::new(0);
    fellowship.chapters[0].layout.clear();
    fellowship.transition_chapter_if_complete();
    fellowship.chapters[1].layout.clear();
    fellowship.transition_chapter_if_complete();
    fellowship.chapters[2].layout.clear();
    fellowship
        .map
        .iter_mut()
        .find(|region| region.region == Region::Lindon)
        .unwrap()
        .fellowship_units = 1;
    fellowship
        .effect_queue
        .push_back(Effect::TransitionChapterIfComplete);
    fellowship.resolve_effects();
    assert_eq!(fellowship.winner(), Some(Faction::Fellowship));
    assert_eq!(
        fellowship.victory_type(),
        Some(VictoryType::TerritorialComparison)
    );
}
#[test]
fn simultaneous_matching_and_three_race_rewards_require_a_resolution_choice() {
    let mut game = Game::new(0);
    let humans = ChapterCard {
        chapter: 1,
        number: 17,
        copy: 0,
    };
    game.sauron.tableau.extend([
        humans,
        humans,
        ChapterCard {
            chapter: 1,
            number: 18,
            copy: 0,
        },
        ChapterCard {
            chapter: 1,
            number: 19,
            copy: 0,
        },
    ]);
    game.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    game.resolve_effects();

    assert_eq!(
        game.legal_actions(),
        vec![
            Action::ResolveAllianceTrigger {
                trigger: AllianceTrigger::MatchingRace(Race::Humans),
            },
            Action::ResolveAllianceTrigger {
                trigger: AllianceTrigger::ThreeDifferentRaces,
            },
        ]
    );
}
#[test]
fn landmark_victory_stops_later_landmark_effects_immediately() {
    let mut game = Game::new(0);
    game.active_player = Faction::Fellowship;
    for region in REGIONS {
        if region != Region::Rohan {
            game.map
                .iter_mut()
                .find(|state| state.region == region)
                .unwrap()
                .fortress = Some(Faction::Fellowship);
        }
    }
    game.effect_queue.push_back(Effect::ResolveLandmark {
        player: Faction::Fellowship,
        id: "helms_deep".into(),
    });
    game.resolve_effects();

    assert_eq!(game.winner(), Some(Faction::Fellowship));
    assert_eq!(game.victory_type(), Some(VictoryType::Conquest));
    assert!(game.pending_decision.is_none());
    assert_eq!(
        game.region_unit_count(Faction::Fellowship, Region::Rohan),
        0
    );
}
#[test]
fn a_short_alliance_stack_reveals_only_its_remaining_token() {
    let mut game = Game::new(0);
    game.alliance_stacks
        .iter_mut()
        .find(|stack| stack.race == Race::Elves)
        .unwrap()
        .token_order = vec![3];
    game.pending_decision = Some(PendingDecision::AllianceRaceChoice {
        player: Faction::Sauron,
    });
    game.apply(Action::ChooseAllianceRace { race: Race::Elves })
        .unwrap();

    assert_eq!(
        game.legal_actions(),
        vec![Action::TakeAllianceToken {
            token: AllianceToken {
                race: Race::Elves,
                id: 3,
            },
        }]
    );
}
#[test]
fn quest_bonus_can_be_explicitly_declined() {
    let mut game = Game::new(0);
    game.quest.fellowship_position = 20;
    game.quest.sauron_position = 5;
    game.advance_quest(Faction::Fellowship, 1);
    game.resolve_effects();

    assert!(game.legal_actions().contains(&Action::SkipQuestBonus));
    while game.legal_actions().contains(&Action::SkipQuestBonus) {
        game.apply(Action::SkipQuestBonus).unwrap();
    }
    assert!(game.pending_decision.is_none());
}
#[test]
fn ent_fortress_removal_is_mandatory_when_an_enemy_fortress_exists() {
    let mut game = Game::new(0);
    game.map
        .iter_mut()
        .find(|region| region.region == Region::Arnor)
        .unwrap()
        .fortress = Some(Faction::Fellowship);
    game.effect_queue.push_back(Effect::ResolveAllianceEffect {
        player: Faction::Sauron,
        effect: alliance_schema::AllianceEffect::RemoveEnemyFortress,
    });
    game.resolve_effects();

    assert!(!game.legal_actions().contains(&Action::SkipQuestBonus));
    assert_eq!(
        game.legal_actions(),
        vec![Action::RemoveQuestBonusFortress {
            region: Region::Arnor,
        }]
    );
}
