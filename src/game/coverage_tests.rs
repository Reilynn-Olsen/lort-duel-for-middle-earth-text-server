use crate::*;

use super::*;

fn card(chapter: u8, number: u8) -> ChapterCard {
    ChapterCard {
        chapter,
        number,
        copy: 0,
    }
}

#[test]
fn apply_for_limits_actions_to_the_active_or_deciding_player() {
    let mut game = Game::new(0);
    let action = game.legal_actions_for(Faction::Sauron)[0].clone();

    assert!(game.legal_actions_for(Faction::Fellowship).is_empty());
    assert_eq!(
        game.apply_for(Faction::Fellowship, action.clone()),
        Err(GameError::IllegalAction(action.clone()))
    );
    game.apply_for(Faction::Sauron, action).unwrap();

    assert!(
        game.legal_actions_for(Faction::Sauron)
            .contains(&Action::DiscardSelectedCard)
    );
    assert!(game.legal_actions_for(Faction::Fellowship).is_empty());
}

#[test]
fn illegal_actions_do_not_change_state_and_game_over_rejects_direct_actions() {
    let mut game = Game::new(0);
    let before = game.clone();
    let illegal = Action::TakeChapterCard {
        slot_id: usize::MAX,
    };
    assert_eq!(
        game.apply(illegal.clone()),
        Err(GameError::IllegalAction(illegal))
    );
    assert_eq!(game, before);

    game.record_victory(Faction::Sauron, VictoryType::Quest);
    assert_eq!(
        game.apply(Action::DiscardSelectedCard),
        Err(GameError::GameOver)
    );
    assert!(game.legal_actions_for(Faction::Sauron).is_empty());
}

#[test]
fn stale_actions_and_replayed_legal_sequences_cannot_diverge_state() {
    let mut stale = Game::new(0);
    let first = stale.legal_actions()[0].clone();
    stale.apply(first).unwrap();
    let stale_action = Action::TakeChapterCard { slot_id: 0 };
    let before = stale.clone();
    assert_eq!(
        stale.apply(stale_action.clone()),
        Err(GameError::IllegalAction(stale_action))
    );
    assert_eq!(stale, before);

    let mut first = Game::new(42);
    let mut second = Game::new(42);
    for _ in 0..12 {
        let action = first.legal_actions().into_iter().next().unwrap();
        first.apply(action.clone()).unwrap();
        second.apply(action).unwrap();
        assert_eq!(first, second);
        if first.is_over() {
            break;
        }
    }
}

#[test]
fn any_skill_reduces_one_missing_skill_and_is_not_offered_for_free_cards() {
    let mut game = Game::new(0);
    let costly = card(1, 13); // Requires one Strength.
    game.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 3,
    });
    assert_eq!(game.normal_payment_for(costly), Some(1));
    assert_eq!(game.payment_for_using_any_skill(costly), Some(0));

    game.pending_decision = Some(PendingDecision::CardDisposition { card: costly });
    assert!(
        game.legal_actions()
            .contains(&Action::PlaySelectedCardUsingAnySkill)
    );

    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 9) });
    assert!(
        !game
            .legal_actions()
            .contains(&Action::PlaySelectedCardUsingAnySkill)
    );
}

#[test]
fn landmark_payment_includes_fortress_surcharge_and_persistent_waiver() {
    let mut game = Game::new(0);
    game.sauron.coins = 10;
    for region in [Region::Lindon, Region::Rhovanion] {
        game.map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(Faction::Sauron);
    }
    // Helm's Deep is six Skills short with no tableau; two Fortresses add two Coins.
    assert_eq!(game.landmark_payment("helms_deep"), Some(8));
    game.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Dwarves,
        id: 1,
    });
    assert_eq!(game.landmark_payment("helms_deep"), Some(6));
    assert_eq!(game.landmark_payment_using_any_skill("helms_deep"), None);

    game.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Elves,
        id: 3,
    });
    assert_eq!(game.landmark_payment_using_any_skill("helms_deep"), Some(5));
}

#[test]
fn optional_quest_coin_and_extra_turn_bonuses_apply_only_when_accepted() {
    let mut coins = Game::new(0);
    coins.effect_queue.push_back(Effect::ResolveQuestBonus {
        player: Faction::Sauron,
        effect: QuestBonusEffect::GainCoin,
    });
    coins.resolve_effects();
    coins.apply(Action::AcceptQuestBonus).unwrap();
    assert_eq!(coins.sauron.coins, 3);

    let mut turn = Game::new(0);
    turn.effect_queue.push_back(Effect::ResolveQuestBonus {
        player: Faction::Sauron,
        effect: QuestBonusEffect::TakeExtraTurn,
    });
    turn.resolve_effects();
    turn.apply(Action::AcceptQuestBonus).unwrap();
    assert_eq!(turn.extra_turns, 1);
}

#[test]
fn optional_quest_fortress_removal_can_be_skipped_or_used() {
    let mut game = Game::new(0);
    game.map
        .iter_mut()
        .find(|state| state.region == Region::Arnor)
        .unwrap()
        .fortress = Some(Faction::Fellowship);
    game.effect_queue.push_back(Effect::ResolveQuestBonus {
        player: Faction::Sauron,
        effect: QuestBonusEffect::RemoveEnemyFortress,
    });
    game.resolve_effects();
    assert!(game.legal_actions().contains(&Action::SkipQuestBonus));
    game.apply(Action::RemoveQuestBonusFortress {
        region: Region::Arnor,
    })
    .unwrap();
    assert_eq!(game.map[1].fortress, None);
    assert_eq!(game.fellowship.fortresses_in_supply, 8);
}

#[test]
fn persistent_tokens_trigger_yellow_quest_and_extra_turn_effects() {
    let mut game = Game::new(0);
    game.sauron.alliance_tokens.extend([
        AllianceToken {
            race: Race::Elves,
            id: 1,
        },
        AllianceToken {
            race: Race::Humans,
            id: 1,
        },
    ]);
    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 9) });
    game.apply(Action::PlaySelectedCard).unwrap();

    assert_eq!(game.quest.sauron_position, 1);
    assert_eq!(game.active_player(), Faction::Sauron);
    assert_eq!(game.extra_turns, 0);
}

#[test]
fn persistent_red_deployment_and_extra_unit_expand_the_card_effect() {
    let mut game = Game::new(0);
    game.sauron.alliance_tokens.extend([
        AllianceToken {
            race: Race::Elves,
            id: 2,
        },
        AllianceToken {
            race: Race::Humans,
            id: 2,
        },
    ]);
    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 21) });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(game.legal_actions().len(), REGIONS.len());
    game.apply(Action::PlaceCardUnits {
        region: Region::Lindon,
    })
    .unwrap();
    assert_eq!(game.region_unit_count(Faction::Sauron, Region::Lindon), 2);
    assert_eq!(game.sauron.units_in_supply, 11);
}

#[test]
fn coin_gains_are_capped_and_discard_income_uses_the_largest_multiplier() {
    let mut capped = Game::new(0);
    capped.coin_reserve = 1;
    capped.gain_coins(Faction::Sauron, 5);
    assert_eq!((capped.sauron.coins, capped.coin_reserve), (3, 0));

    let mut discard = Game::new(0);
    discard.chapters[0].layout.clear();
    discard.transition_chapter_if_complete();
    discard.chapters[1].layout.clear();
    discard.transition_chapter_if_complete();
    discard.sauron.alliance_tokens.push(AllianceToken {
        race: Race::Humans,
        id: 3,
    });
    discard.pending_decision = Some(PendingDecision::CardDisposition { card: card(3, 1) });
    discard.apply(Action::DiscardSelectedCard).unwrap();
    assert_eq!(discard.sauron.coins, 8);
    assert_eq!(discard.coin_reserve, 19);
}

#[test]
fn unavailable_unit_effects_expose_explicit_fallback_actions() {
    let mut placement = Game::new(0);
    placement.sauron.units_in_supply = 0;
    placement.pending_decision = Some(PendingDecision::CardUnitPlacement {
        player: Faction::Sauron,
        amount: 2,
        regions: vec![Region::Lindon],
    });
    assert_eq!(
        placement.legal_actions(),
        vec![Action::ResolveEmptyCardUnitPlacement]
    );
    placement
        .apply(Action::ResolveEmptyCardUnitPlacement)
        .unwrap();

    let mut movement = Game::new(0);
    movement
        .map
        .iter_mut()
        .for_each(|state| state.sauron_units = 0);
    movement.pending_decision = Some(PendingDecision::CardUnitMovement {
        player: Faction::Sauron,
        remaining: 1,
    });
    assert_eq!(
        movement.legal_actions(),
        vec![Action::ResolveUnavailableManeuver]
    );
    movement.apply(Action::ResolveUnavailableManeuver).unwrap();

    let mut removal = Game::new(0);
    removal
        .map
        .iter_mut()
        .for_each(|state| state.fellowship_units = 0);
    removal.pending_decision = Some(PendingDecision::CardEnemyUnitRemoval {
        player: Faction::Sauron,
        remaining: 1,
    });
    assert_eq!(
        removal.legal_actions(),
        vec![Action::ResolveUnavailableManeuver]
    );
}

#[test]
fn alliance_unit_placement_and_ent_maneuvers_continue_one_step_at_a_time() {
    let mut units = Game::new(0);
    units.effect_queue.push_back(Effect::ResolveAllianceEffect {
        player: Faction::Sauron,
        effect: alliance_schema::AllianceEffect::PlaceUnitsAnywhere { amount: 2 },
    });
    units.resolve_effects();
    units
        .apply(Action::PlaceAllianceUnits {
            region: Region::Lindon,
        })
        .unwrap();
    assert!(matches!(
        units.pending_decision,
        Some(PendingDecision::AllianceUnitPlacement { remaining: 1, .. })
    ));
    units
        .apply(Action::PlaceAllianceUnits {
            region: Region::Gondor,
        })
        .unwrap();
    assert_eq!(units.region_unit_count(Faction::Sauron, Region::Lindon), 1);
    assert_eq!(units.region_unit_count(Faction::Sauron, Region::Gondor), 1);

    let mut ents = Game::new(0);
    ents.effect_queue.push_back(Effect::ResolveEntManeuvers {
        player: Faction::Sauron,
        remaining: 3,
    });
    ents.resolve_effects();
    ents.apply(Action::ChooseEntManeuver {
        maneuver: EntManeuver::OpponentLosesCoin,
    })
    .unwrap();
    assert_eq!(ents.fellowship.coins, 2);
    assert!(matches!(
        ents.pending_decision,
        Some(PendingDecision::EntManeuverChoice { remaining: 2, .. })
    ));
}

#[test]
fn final_territorial_comparison_counts_fortresses_for_sauron() {
    let mut game = Game::new(0);
    for region in [Region::Lindon, Region::Rhovanion] {
        game.map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(Faction::Sauron);
    }
    game.resolve_final_territorial_comparison();
    assert_eq!(game.winner(), Some(Faction::Sauron));
    assert_eq!(
        game.victory_type(),
        Some(VictoryType::TerritorialComparison)
    );
}

#[test]
#[ignore = "Ent token's mandatory fortress removal has no specified no-target fallback, leaving no legal action"]
fn regression_mandatory_ent_fortress_removal_without_enemy_fortress_deadlocks() {
    let mut game = Game::new(0);
    game.effect_queue.push_back(Effect::ResolveAllianceEffect {
        player: Faction::Sauron,
        effect: alliance_schema::AllianceEffect::RemoveEnemyFortress,
    });
    game.resolve_effects();
    assert!(game.legal_actions().is_empty());
}
