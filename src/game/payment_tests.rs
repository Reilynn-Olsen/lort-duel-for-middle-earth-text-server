use crate::*;

use super::test_support::assert_illegal_action_unchanged;
use super::*;

fn card(chapter: u8, number: u8) -> ChapterCard {
    ChapterCard {
        chapter,
        number,
        copy: 0,
    }
}

fn selected(card: ChapterCard) -> Game {
    let mut game = Game::new(0);
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game
}

#[test]
fn skill_payments_cover_exact_missing_repeated_and_choice_symbols() {
    // C2-12 requires Strength once and Courage twice.
    let target = card(2, 12);
    let mut exact = selected(target);
    exact
        .sauron
        .tableau
        .extend([card(1, 5), card(1, 3), card(1, 4)]);
    assert_eq!(exact.payment_for(target), Some(0));

    let mut one_missing = selected(target);
    one_missing.sauron.tableau.extend([card(1, 5), card(1, 3)]);
    assert_eq!(one_missing.payment_for(target), Some(1));

    let mut multiple_missing = selected(target);
    multiple_missing.sauron.tableau.push(card(1, 5));
    assert_eq!(multiple_missing.payment_for(target), Some(2));

    // C2-13 requires Strength twice and Knowledge once. One Strength symbol
    // cannot satisfy both printed Strength requirements.
    let repeated = card(2, 13);
    let mut repeated_skill = selected(repeated);
    repeated_skill.sauron.tableau.push(card(1, 5));
    assert_eq!(repeated_skill.payment_for(repeated), Some(2));

    // C2-02 supplies exactly one of Knowledge or Leadership each turn.
    let choice = card(2, 2);
    let choice_target = card(2, 1); // Leadership + Knowledge, plus one coin.
    let mut choice_as_knowledge = selected(choice_target);
    choice_as_knowledge
        .sauron
        .tableau
        .extend([card(1, 8), choice]);
    assert_eq!(choice_as_knowledge.payment_for(choice_target), Some(1));
    let mut choice_as_leadership = selected(choice_target);
    choice_as_leadership
        .sauron
        .tableau
        .extend([card(1, 7), choice]);
    assert_eq!(choice_as_leadership.payment_for(choice_target), Some(1));
    let mut choice_cannot_be_both = selected(choice_target);
    choice_cannot_be_both.sauron.tableau.push(choice);
    assert_eq!(choice_cannot_be_both.payment_for(choice_target), Some(2));
}

#[test]
fn printed_costs_coins_free_cards_and_chains_have_exact_payments() {
    let target = card(2, 1); // One printed coin and two missing Skills.
    let game = selected(target);
    assert_eq!(game.payment_for(target), None); // Sauron begins with two coins.

    let mut combined = selected(target);
    combined.sauron.coins = 3;
    assert_eq!(combined.payment_for(target), Some(3));
    combined.sauron.tableau.push(card(1, 8));
    assert_eq!(combined.payment_for(target), Some(2)); // owned Leadership + Knowledge substitution

    let free = card(1, 9);
    assert_eq!(selected(free).payment_for(free), Some(0));

    // C1-14 provides Backpack; C2-10 normally costs one coin.
    let chained = card(2, 10);
    let mut chain = selected(chained);
    chain.sauron.tableau.push(card(1, 14));
    assert_eq!(chain.normal_payment_for(chained), Some(1));
    assert_eq!(chain.payment_for(chained), Some(0));
    chain.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 3,
    });
    assert_eq!(chain.payment_for_using_any_skill(chained), None);
    assert!(
        !chain
            .legal_actions()
            .contains(&Action::PlaySelectedCardUsingAnySkill)
    );
    assert!(chain.legal_actions().contains(&Action::PlaySelectedCard));
    assert!(
        chain
            .legal_actions()
            .contains(&Action::PlaySelectedCardNormally)
    );
    chain.apply(Action::PlaySelectedCardNormally).unwrap();
    assert_eq!(chain.sauron.coins, 1);
}

#[test]
fn landmark_payments_include_each_fortress_surcharge_and_waivers() {
    let mut game = Game::new(0);
    game.sauron.coins = 20;
    // Helm's Deep requires six Skills; each owned Fortress adds one coin.
    for (count, region) in REGIONS.into_iter().enumerate() {
        assert_eq!(game.landmark_payment("helms_deep"), Some(6 + count as u8));
        game.map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(Faction::Sauron);
    }
    assert_eq!(game.landmark_payment("helms_deep"), Some(13));
    game.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Dwarves,
        id: 1,
    });
    assert_eq!(game.landmark_payment("helms_deep"), Some(6));

    game.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 3,
    });
    assert_eq!(game.landmark_payment_using_any_skill("helms_deep"), Some(5));

    let mut exact_skills = Game::new(0);
    // Helm's Deep needs Ruse x3, Leadership x2, and Knowledge x1.
    exact_skills.sauron.tableau.extend([
        card(1, 1),
        card(2, 3),
        card(1, 8),
        card(2, 6),
        card(1, 7),
    ]);
    assert_eq!(exact_skills.landmark_payment("helms_deep"), Some(0));
}

#[test]
fn discarding_is_legal_when_playing_is_unaffordable_and_uses_chapter_income() {
    let costly = card(1, 13);
    let mut game = selected(costly);
    game.sauron.coins = 0;
    assert_eq!(game.legal_actions(), vec![Action::DiscardSelectedCard]);
    game.apply(Action::DiscardSelectedCard).unwrap();
    assert_eq!(game.sauron.coins, 1); // Chapter 1 income.

    let mut chapter_two = Game::new(0);
    chapter_two.chapters[0].layout.clear();
    chapter_two.transition_chapter_if_complete();
    chapter_two.pending_decision = Some(PendingDecision::CardDisposition { card: card(2, 1) });
    chapter_two.apply(Action::DiscardSelectedCard).unwrap();
    assert_eq!(chapter_two.sauron.coins, 4); // Chapter 2 income.
}

#[test]
fn illegal_card_and_landmark_selections_are_precise_and_have_no_partial_execution() {
    let mut covered = Game::new(0);
    let slot = covered.chapters[0]
        .layout
        .iter()
        .find(|slot| !slot.covering_slots.is_empty())
        .unwrap()
        .id;
    assert_illegal_action_unchanged(&mut covered, Action::TakeChapterCard { slot_id: slot });

    let mut facedown = Game::new(0);
    let slot = facedown.chapters[0]
        .layout
        .iter()
        .find(|slot| slot.visibility == CardVisibility::FaceDown)
        .unwrap()
        .id;
    assert_illegal_action_unchanged(&mut facedown, Action::TakeChapterCard { slot_id: slot });

    let mut removed = Game::new(0);
    assert_illegal_action_unchanged(
        &mut removed,
        Action::TakeChapterCard {
            slot_id: usize::MAX,
        },
    );

    let mut unavailable_landmark = Game::new(0);
    unavailable_landmark.landmarks.available.clear();
    assert_illegal_action_unchanged(
        &mut unavailable_landmark,
        Action::TakeLandmark {
            id: "helms_deep".into(),
        },
    );

    let mut unaffordable_landmark = Game::new(0);
    unaffordable_landmark.landmarks.available = vec!["helms_deep".into()];
    assert_illegal_action_unchanged(
        &mut unaffordable_landmark,
        Action::TakeLandmark {
            id: "helms_deep".into(),
        },
    );
}

#[test]
fn rejected_card_dispositions_preserve_state_and_emit_no_events() {
    let costly = card(2, 1);
    let mut game = selected(costly);
    assert_eq!(game.resolution_events(), []);
    assert_illegal_action_unchanged(&mut game, Action::PlaySelectedCard);
    assert_illegal_action_unchanged(&mut game, Action::PlaySelectedCardUsingAnySkill);
    assert_eq!(game.resolution_events(), []);
}
