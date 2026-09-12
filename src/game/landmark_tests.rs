use crate::*;

use super::*;

#[test]
fn isengard_requires_a_grey_card_choice_then_discards_it_and_advances_the_quest() {
    let mut game = Game::new(0);
    let grey_card = ChapterCard {
        chapter: 1,
        number: 1,
        copy: 0,
    };
    game.fellowship.tableau.push(grey_card);
    game.effect_queue.push_back(Effect::ResolveLandmark {
        player: Faction::Sauron,
        id: "isengard".into(),
    });
    game.resolve_effects();

    assert_eq!(
        game.legal_actions(),
        vec![Action::DiscardOpponentGreyCard { card: grey_card }]
    );
    game.apply(Action::DiscardOpponentGreyCard { card: grey_card })
        .unwrap();

    assert!(!game.fellowship.tableau.contains(&grey_card));
    assert!(game.chapters[0].discarded.contains(&grey_card));
    assert_eq!(game.quest.sauron_position, 1);
}
#[test]
fn isengard_skips_the_discard_choice_when_the_opponent_has_no_grey_cards() {
    let mut game = Game::new(0);
    game.effect_queue.push_back(Effect::ResolveLandmark {
        player: Faction::Sauron,
        id: "isengard".into(),
    });
    game.resolve_effects();

    assert!(game.pending_decision.is_none());
    assert_eq!(game.quest.sauron_position, 1);
}
#[test]
fn barad_dur_plays_any_shared_discarded_card_for_free_and_resolves_it() {
    let mut game = Game::new(0);
    let card = ChapterCard {
        chapter: 1,
        number: 9,
        copy: 0,
    };
    game.chapters[0].layout.retain(|slot| slot.card != card);
    game.chapters[0]
        .discarded
        .retain(|discarded| *discarded != card);
    game.chapters[0].discarded.push(card);
    game.effect_queue.push_back(Effect::ResolveLandmark {
        player: Faction::Sauron,
        id: "barad_dur".into(),
    });
    game.resolve_effects();

    assert!(
        game.legal_actions()
            .contains(&Action::PlayDiscardedCardForFree { card })
    );
    game.apply(Action::PlayDiscardedCardForFree { card })
        .unwrap();

    assert!(game.sauron.tableau.contains(&card));
    assert!(!game.discarded_cards().contains(&card));
    assert_eq!(game.sauron.coins, 4);
    assert_eq!(game.coin_reserve, 23);
}
#[test]
fn grey_havens_reveals_two_tokens_from_the_chosen_race_and_keeps_only_one_public() {
    let mut game = Game::new(0);
    game.alliance_stacks
        .iter_mut()
        .find(|stack| stack.race == Race::Elves)
        .unwrap()
        .token_order = vec![1, 2];
    game.effect_queue.push_back(Effect::ResolveLandmark {
        player: Faction::Sauron,
        id: "grey_havens".into(),
    });
    game.resolve_effects();
    game.apply(Action::ChooseAllianceRace { race: Race::Elves })
        .unwrap();
    game.apply(Action::TakeAllianceToken {
        token: AllianceToken {
            race: Race::Elves,
            id: 2,
        },
    })
    .unwrap();

    assert_eq!(
        game.sauron.alliance_tokens,
        vec![AllianceToken {
            race: Race::Elves,
            id: 2
        }]
    );
    assert_eq!(
        game.alliance_stacks
            .iter()
            .find(|stack| stack.race == Race::Elves)
            .unwrap()
            .token_order,
        vec![1]
    );
    assert!(game.render_state().contains("Red deployment anywhere"));
}
