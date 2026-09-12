use crate::*;

use super::*;

const CHAPTER_ONE_COVERS: &[&[usize]] = &[
    &[2, 3],
    &[3, 4],
    &[5, 6],
    &[6, 7],
    &[7, 8],
    &[9, 10],
    &[10, 11],
    &[11, 12],
    &[12, 13],
    &[14, 15],
    &[15, 16],
    &[16, 17],
    &[17, 18],
    &[18, 19],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
];

#[test]
fn every_issued_action_has_a_self_contained_description() {
    let mut game = Game::new(42);
    for _ in 0..150 {
        let player = game.active_player();
        let actions = game.legal_actions_for(player);
        if actions.is_empty() {
            break;
        }
        for action in &actions {
            let description = game.describe_action(action);
            assert!(description.contains("Immediate result:"));
            assert!(description.len() > action.to_string().len());
        }
        game.apply_for(player, actions[0].clone()).unwrap();
        if game.is_over() {
            break;
        }
    }
}
const CHAPTER_TWO_COVERS: &[&[usize]] = &[
    &[6],
    &[6, 7],
    &[7, 8],
    &[8, 9],
    &[9, 10],
    &[10],
    &[11],
    &[11, 12],
    &[12, 13],
    &[13, 14],
    &[14],
    &[15],
    &[15, 16],
    &[16, 17],
    &[17],
    &[18],
    &[18, 19],
    &[19],
    &[],
    &[],
];
const CHAPTER_THREE_COVERS: &[&[usize]] = &[
    &[2, 3],
    &[3, 4],
    &[5, 6],
    &[6, 7],
    &[7, 8],
    &[],
    &[9],
    &[10],
    &[],
    &[12],
    &[13],
    &[15],
    &[15, 16],
    &[16, 17],
    &[17],
    &[18],
    &[18, 19],
    &[19],
    &[],
    &[],
];

#[test]
fn official_layouts_have_exact_rows_visibility_and_covering_relationships() {
    for (chapter, rows, visibility, covers) in [
        (
            1,
            &[2, 3, 4, 5, 6][..],
            &[
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
            ][..],
            CHAPTER_ONE_COVERS,
        ),
        (
            2,
            &[6, 5, 4, 3, 2][..],
            &[
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
            ][..],
            CHAPTER_TWO_COVERS,
        ),
        (
            3,
            &[2, 3, 4, 2, 4, 3, 2][..],
            &[
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
                CardVisibility::FaceDown,
                CardVisibility::FaceUp,
            ][..],
            CHAPTER_THREE_COVERS,
        ),
    ] {
        let mut rng = crate::catalog::DeterministicRng::new(1);
        let state = crate::catalog::setup_chapter(chapter, &mut rng);
        assert_eq!(state.layout.len(), CARDS_IN_LAYOUT);
        assert_eq!(state.discarded.len(), 3);
        assert_eq!(
            (0..rows.len())
                .map(|row| state
                    .layout
                    .iter()
                    .filter(|slot| slot.row == row as u8)
                    .count())
                .collect::<Vec<_>>(),
            rows
        );
        assert_eq!(
            state
                .layout
                .iter()
                .map(|slot| slot.covering_slots.as_slice())
                .collect::<Vec<_>>(),
            covers
        );
        for slot in &state.layout {
            assert_eq!(
                slot.visibility, visibility[slot.row as usize],
                "Chapter {chapter}, slot {}",
                slot.id
            );
        }
    }
}

#[test]
fn availability_is_exactly_face_up_and_uncovered() {
    let game = Game::new(3);
    let chapter = game.current_chapter_state();
    for slot in &chapter.layout {
        assert_eq!(
            game.available_chapter_slot_ids().contains(&slot.id),
            slot.visibility == CardVisibility::FaceUp && slot.covering_slots.is_empty(),
            "slot {} has incorrect availability",
            slot.id
        );
    }
}

#[test]
fn reveal_flips_only_cards_whose_last_cover_is_removed_and_runs_before_turn_completion() {
    let mut zero = Game::new(0);
    zero.chapters[0].layout.retain(|slot| slot.id != 14);
    zero.reveal_newly_available_cards();
    assert_eq!(
        zero.chapters[0]
            .layout
            .iter()
            .find(|slot| slot.id == 9)
            .unwrap()
            .visibility,
        CardVisibility::FaceDown
    );

    let mut one = Game::new(0);
    one.chapters[0]
        .layout
        .retain(|slot| slot.id != 14 && slot.id != 15);
    one.reveal_newly_available_cards();
    assert_eq!(
        one.chapters[0]
            .layout
            .iter()
            .find(|slot| slot.id == 9)
            .unwrap()
            .visibility,
        CardVisibility::FaceUp
    );
    assert_eq!(
        one.chapters[0]
            .layout
            .iter()
            .find(|slot| slot.id == 10)
            .unwrap()
            .visibility,
        CardVisibility::FaceDown
    );

    let mut multiple = Game::new(0);
    multiple.chapters[0]
        .layout
        .retain(|slot| ![14, 15, 16].contains(&slot.id));
    multiple.reveal_newly_available_cards();
    assert!(
        multiple.chapters[0]
            .layout
            .iter()
            .filter(|slot| [9, 10].contains(&slot.id))
            .all(|slot| slot.visibility == CardVisibility::FaceUp)
    );

    let mut resolved = Game::new(0);
    resolved
        .apply(Action::TakeChapterCard { slot_id: 14 })
        .unwrap();
    resolved.run_action(Action::DiscardSelectedCard).unwrap();
    resolved
        .apply(Action::TakeChapterCard { slot_id: 15 })
        .unwrap();
    resolved.run_action(Action::DiscardSelectedCard).unwrap();
    assert_eq!(
        resolved.chapters[0]
            .layout
            .iter()
            .find(|slot| slot.id == 9)
            .unwrap()
            .visibility,
        CardVisibility::FaceUp
    );
    let reveal = resolved
        .resolution_events()
        .iter()
        .position(|event| {
            matches!(
                event,
                super::test_support::ResolutionEvent::RevealNewlyAvailableCards
            )
        })
        .unwrap();
    let complete = resolved
        .resolution_events()
        .iter()
        .position(|event| {
            matches!(
                event,
                super::test_support::ResolutionEvent::CompleteTurn { .. }
            )
        })
        .unwrap();
    assert!(reveal < complete);
}

#[test]
fn chapter_boundaries_refill_landmarks_and_preserve_unused_cards_outside_the_display() {
    let mut game = Game::new(0);
    let initial_cards = game.chapters[0]
        .layout
        .iter()
        .map(|slot| slot.card)
        .chain(game.chapters[0].discarded.iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(initial_cards.len(), CARDS_PER_CHAPTER);
    assert_eq!(game.chapters[0].discarded.len(), 3);
    game.landmarks.available.clear();
    game.chapters[0].layout.pop();
    game.queue_turn_completion();
    game.resolve_effects();
    assert_eq!(
        game.current_chapter, 1,
        "a nonempty display cannot transition"
    );
    assert!(
        game.landmarks.available.is_empty(),
        "Landmarks refill only at a boundary"
    );

    game.chapters[0].layout.clear();
    game.queue_turn_completion();
    game.resolve_effects();
    assert_eq!(game.current_chapter, 2);
    assert_eq!(game.chapters[1].layout.len(), CARDS_IN_LAYOUT);
    assert_eq!(game.chapters[1].discarded.len(), 3);
    assert_eq!(game.landmarks.available.len(), 3);
    assert_eq!(game.active_player(), Faction::Sauron);
}

#[test]
fn extra_turns_apply_on_normal_and_boundary_turns_and_can_overlap() {
    let yellow = ChapterCard {
        chapter: 1,
        number: 9,
        copy: 0,
    };
    let mut normal = Game::new(0);
    normal.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 1,
    });
    normal.pending_decision = Some(PendingDecision::CardDisposition { card: yellow });
    normal.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(normal.active_player(), Faction::Sauron);
    assert_eq!(normal.extra_turns, 0);

    let mut boundary = Game::new(0);
    boundary.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 1,
    });
    boundary.chapters[0].layout.retain(|slot| slot.id == 14);
    boundary.chapters[0].layout[0].card = yellow;
    boundary
        .apply(Action::TakeChapterCard { slot_id: 14 })
        .unwrap();
    boundary.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(
        (
            boundary.current_chapter,
            boundary.active_player(),
            boundary.extra_turns
        ),
        (2, Faction::Sauron, 0)
    );

    // Elf's Yellow-card reward and the Human quest movement can both grant an
    // extra turn when that movement reaches the position-10 bonus.
    let mut overlapping = Game::new(0);
    overlapping.sauron.alliance_tokens.extend([
        AllianceToken {
            race: Race::Elves,
            id: 1,
        },
        AllianceToken {
            race: Race::Humans,
            id: 1,
        },
    ]);
    overlapping.quest.sauron_position = 9;
    overlapping.quest.fellowship_position = 12;
    overlapping.pending_decision = Some(PendingDecision::CardDisposition { card: yellow });
    overlapping.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(
        overlapping.legal_actions(),
        vec![Action::AcceptQuestBonus, Action::SkipQuestBonus]
    );
    overlapping.apply(Action::AcceptQuestBonus).unwrap();
    assert_eq!(
        (overlapping.active_player(), overlapping.extra_turns),
        (Faction::Sauron, 1)
    );
}

#[test]
fn finished_games_do_not_complete_turn_or_start_another_chapter() {
    let mut game = Game::new(0);
    game.chapters[0].layout.clear();
    game.record_victory(Faction::Sauron, VictoryType::Quest);
    let before = game.clone();
    game.queue_turn_completion();
    game.resolve_effects();
    assert_eq!(game.current_chapter, before.current_chapter);
    assert_eq!(game.turn, before.turn);
    assert_eq!(game.active_player(), before.active_player());
}
