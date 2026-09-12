use crate::*;

use super::*;

#[test]
fn a_choice_skill_contributes_only_one_of_its_alternatives_to_a_payment() {
    let mut game = Game::new(0);
    // C2-02 provides Knowledge *or* Leadership, while C2-01 requires
    // both. Its single choice symbol therefore leaves one missing Skill.
    game.sauron.tableau.push(ChapterCard {
        chapter: 2,
        number: 2,
        copy: 0,
    });
    let target = ChapterCard {
        chapter: 2,
        number: 1,
        copy: 0,
    };
    assert_eq!(game.payment_for(target), Some(2)); // 1 printed + 1 missing Skill
    game.sauron.coins = 1;
    assert_eq!(game.payment_for(target), None);
    game.pending_decision = Some(PendingDecision::CardDisposition { card: target });
    assert_eq!(game.legal_actions(), vec![Action::DiscardSelectedCard]);
    assert_eq!(
        game.apply(Action::PlaySelectedCard),
        Err(GameError::IllegalAction(Action::PlaySelectedCard))
    );
}
#[test]
fn playing_pays_then_keeps_the_card_and_resolves_its_effect() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 1,
        number: 9,
        copy: 0,
    };
    // C1-09 costs nothing and grants two coins according to cards.json.
    game.chapters[0]
        .discarded
        .retain(|discarded| *discarded != card);
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert!(game.sauron.tableau.contains(&card));
    assert!(!game.chapters[0].discarded.contains(&card));
    assert_eq!(game.sauron.coins, 4);
    assert_eq!(game.coin_reserve, 23);
    assert_eq!(game.active_player(), Faction::Fellowship);
}
#[test]
fn discarding_never_applies_the_printed_effect() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 1,
        number: 9,
        copy: 0,
    };
    // This is the same two-coin card used above, but discard pays only the
    // current chapter's discard income.
    game.chapters[0]
        .discarded
        .retain(|discarded| *discarded != card);
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game.apply(Action::DiscardSelectedCard).unwrap();
    assert!(game.chapters[0].discarded.contains(&card));
    assert!(!game.sauron.tableau.contains(&card));
    assert_eq!(game.sauron.coins, 3);
    assert_eq!(game.coin_reserve, 24);
}
#[test]
fn matching_chain_makes_the_entire_card_free_including_printed_coins() {
    let mut game = Game::new(0);
    // C1-14 grants Backpack. C2-10 has a Backpack prerequisite and a
    // printed one-coin cost; the rules define a chain as playing for free.
    game.sauron.tableau.push(ChapterCard {
        chapter: 1,
        number: 14,
        copy: 0,
    });
    let chained_card = ChapterCard {
        chapter: 2,
        number: 10,
        copy: 0,
    };
    assert_eq!(game.payment_for(chained_card), Some(0));
    game.pending_decision = Some(PendingDecision::CardDisposition { card: chained_card });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(game.sauron.coins, 2);
    assert!(game.sauron.tableau.contains(&chained_card));
}
#[test]
fn fellowship_quest_movement_preserves_the_existing_separation_and_claims_crossed_bonuses() {
    let mut game = Game::new(0);
    game.advance_quest(Faction::Fellowship, 3);
    game.resolve_effects();
    while game.legal_actions().contains(&Action::AcceptQuestBonus) {
        game.apply(Action::AcceptQuestBonus).unwrap();
    }
    assert_eq!(game.quest.fellowship_position, 18);
    assert_eq!(game.quest.sauron_position, 3);
    assert_eq!(
        game.quest.fellowship_position - game.quest.sauron_position,
        15
    );
    // Both moved characters crossed a one-time coin space: 3 and 18.
    assert_eq!(game.fellowship.coins, 5);
    assert!(
        game.quest
            .bonuses
            .iter()
            .filter(|bonus| bonus.position == 3 || bonus.position == 18)
            .all(|bonus| bonus.claimed)
    );
}
#[test]
fn playing_a_blue_card_routes_quest_movement_through_the_effect_queue() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 1,
        number: 16,
        copy: 0,
    };
    // C1-16 has one AdvanceQuest effect in cards.json.
    game.active_player = Faction::Fellowship;
    game.chapters[0]
        .discarded
        .retain(|discarded| *discarded != card);
    game.pending_decision = Some(PendingDecision::CardDisposition { card });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(game.quest.fellowship_position, 16);
    assert_eq!(game.quest.sauron_position, 1);
    assert_eq!(game.active_player(), Faction::Sauron);
}
#[test]
fn quest_bonus_unit_placement_is_an_explicit_choice_and_is_claimed_once() {
    let mut game = Game::new(0);
    game.quest.fellowship_position = 20;
    game.quest.sauron_position = 5;
    game.advance_quest(Faction::Fellowship, 1);
    game.resolve_effects();
    assert!(matches!(
        game.pending_decision,
        Some(PendingDecision::QuestBonusUnitPlacement { .. })
    ));
    assert!(game.legal_actions().contains(&Action::PlaceQuestBonusUnit {
        region: Region::Gondor
    }));
    game.apply(Action::PlaceQuestBonusUnit {
        region: Region::Gondor,
    })
    .unwrap();
    assert_eq!(
        game.map
            .iter()
            .find(|state| state.region == Region::Gondor)
            .unwrap()
            .fellowship_units,
        1
    );
    assert!(
        game.quest
            .bonuses
            .iter()
            .find(|bonus| bonus.position == 21)
            .unwrap()
            .claimed
    );
}
#[test]
fn quest_reaching_mount_doom_or_catching_the_fellowship_ends_the_game_immediately() {
    let mut fellowship_win = Game::new(0);
    fellowship_win.quest.fellowship_position = 29;
    fellowship_win.quest.sauron_position = 10;
    fellowship_win.advance_quest(Faction::Fellowship, 1);
    assert_eq!(fellowship_win.winner(), Some(Faction::Fellowship));
    assert_eq!(fellowship_win.victory_type(), Some(VictoryType::Quest));

    let mut sauron_win = Game::new(0);
    sauron_win.quest.fellowship_position = 20;
    sauron_win.quest.sauron_position = 18;
    sauron_win.advance_quest(Faction::Sauron, 2);
    assert_eq!(sauron_win.winner(), Some(Faction::Sauron));
    assert_eq!(sauron_win.victory_type(), Some(VictoryType::Quest));
}
