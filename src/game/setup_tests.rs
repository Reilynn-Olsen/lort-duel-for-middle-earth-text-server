use crate::catalog::chapter_cards;
use crate::*;

use super::*;

#[test]
fn setup_is_replayable() {
    assert_eq!(Game::new(7), Game::new(7));
    assert_ne!(
        Game::new(7).landmarks.available,
        Game::new(8).landmarks.available
    );
}
#[test]
fn initial_factions_and_economy_match_setup() {
    let game = Game::new(0);
    assert_eq!(game.active_player(), Faction::Sauron);
    assert_eq!(
        (game.fellowship.coins, game.sauron.coins, game.coin_reserve),
        (3, 2, 25)
    );
    assert_eq!(
        game.map
            .iter()
            .find(|r| r.region == Region::Arnor)
            .unwrap()
            .fellowship_units,
        2
    );
    assert_eq!(
        game.map
            .iter()
            .find(|r| r.region == Region::Mordor)
            .unwrap()
            .sauron_units,
        2
    );
}
#[test]
fn initial_chapter_uses_its_prescribed_layout_and_discards_the_remainder() {
    let game = Game::new(0);
    assert_eq!(game.chapters.len(), 1);
    assert_eq!(game.chapters[0].layout.len(), 20);
    assert_eq!(game.chapters[0].discarded.len(), 3);
}
#[test]
fn catalogue_has_one_explicit_physical_instance_per_card() {
    let cards = chapter_cards(1);
    assert!(cards.contains(&ChapterCard {
        chapter: 1,
        number: 1,
        copy: 0
    }));
    assert!(cards.contains(&ChapterCard {
        chapter: 1,
        number: 23,
        copy: 0
    }));
    assert_eq!(cards.len(), 23);
}
#[test]
fn layout_records_the_overlaps_that_determine_availability() {
    let chapter = &Game::new(0).chapters[0];
    assert!(
        chapter
            .layout
            .iter()
            .any(|slot| !slot.covering_slots.is_empty())
    );
    assert!(
        chapter
            .layout
            .iter()
            .any(|slot| slot.covering_slots.is_empty())
    );
}
#[test]
fn only_face_up_uncovered_cards_are_available() {
    let game = Game::new(12);
    let available = game.available_chapter_slot_ids();
    assert_eq!(available.len(), 6);
    assert!(available.iter().all(|id| {
        let slot = game.chapters[0]
            .layout
            .iter()
            .find(|slot| slot.id == *id)
            .unwrap();
        slot.visibility == CardVisibility::FaceUp && slot.covering_slots.is_empty()
    }));
    let facedown = game.chapters[0]
        .layout
        .iter()
        .find(|slot| slot.visibility == CardVisibility::FaceDown)
        .unwrap();
    assert!(!available.contains(&facedown.id));
}
#[test]
fn removing_covering_cards_reveals_newly_available_cards_deterministically() {
    let mut first = Game::new(12);
    let mut second = Game::new(12);
    let initial_available = first.available_chapter_slot_ids();
    for slot_id in initial_available {
        for game in [&mut first, &mut second] {
            game.apply(Action::TakeChapterCard { slot_id }).unwrap();
            game.apply(Action::DiscardSelectedCard).unwrap();
        }
    }
    let hidden_row = first.chapters[0]
        .layout
        .iter()
        .filter(|slot| slot.row == 3)
        .collect::<Vec<_>>();
    assert!(!hidden_row.is_empty());
    assert!(
        hidden_row
            .iter()
            .all(|slot| slot.visibility == CardVisibility::FaceUp)
    );
    assert_eq!(first.render_state(), second.render_state());
}
#[test]
fn landmark_state_is_isolated_from_chapter_card_display() {
    let mut game = Game::new(12);
    let display_before = game.chapters[0].layout.clone();
    game.landmarks.available.remove(0);
    assert_eq!(game.chapters[0].layout, display_before);
}
#[test]
fn public_state_does_not_leak_facedown_card_identity() {
    let game = Game::new(4);
    let hidden = game.chapters[0]
        .layout
        .iter()
        .find(|slot| slot.visibility == CardVisibility::FaceDown)
        .unwrap()
        .card
        .to_string();
    assert!(!game.render_state().contains(&hidden));
}
#[test]
fn taking_a_card_requires_a_follow_up_disposition_choice() {
    let mut game = Game::new(42);
    let action = game.legal_actions().into_iter().next().unwrap();
    game.apply(action).unwrap();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::CardDisposition { .. })
    ));
    assert_eq!(
        game.legal_actions(),
        vec![Action::PlaySelectedCard, Action::DiscardSelectedCard]
    );
    assert_eq!(game.turn(), 0);
}
#[test]
fn discarding_runs_the_turn_lifecycle_and_reveals_uncovered_cards() {
    let mut game = Game::new(42);
    let action = game.legal_actions().into_iter().next().unwrap();
    game.apply(action).unwrap();
    game.apply(Action::DiscardSelectedCard).unwrap();
    assert_eq!(game.turn(), 1);
    assert_eq!(game.active_player(), Faction::Fellowship);
    assert_eq!(game.sauron.coins, 3);
    assert_eq!(game.coin_reserve, 24);
    assert!(game.pending_decision.is_none());
}
