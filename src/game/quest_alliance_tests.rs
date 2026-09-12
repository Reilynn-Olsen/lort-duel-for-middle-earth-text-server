use crate::*;

use super::*;

fn card(chapter: u8, number: u8) -> ChapterCard {
    ChapterCard {
        chapter,
        number,
        copy: 0,
    }
}

fn token(race: Race, id: u8) -> AllianceToken {
    AllianceToken { race, id }
}

fn resolve_token(game: &mut Game, token: AllianceToken) {
    game.effect_queue.push_back(Effect::ResolveAllianceToken {
        player: Faction::Sauron,
        token,
    });
    game.resolve_effects();
}

#[test]
fn quest_movement_crossings_choices_and_boundaries_are_exact() {
    let mut fellowship = Game::new(0);
    fellowship.quest.fellowship_position = 6;
    fellowship.quest.sauron_position = 2;
    fellowship.advance_quest(Faction::Fellowship, 7);
    fellowship.resolve_effects();
    assert_eq!(
        (
            fellowship.quest.fellowship_position,
            fellowship.quest.sauron_position
        ),
        (13, 2)
    );
    assert!(matches!(
        fellowship.pending_decision,
        Some(PendingDecision::QuestBonusUnitPlacement { .. })
    ));
    fellowship
        .apply(Action::PlaceQuestBonusUnit {
            region: Region::Lindon,
        })
        .unwrap();
    assert!(matches!(
        fellowship.pending_decision,
        Some(PendingDecision::QuestBonusChoice {
            effect: QuestBonusEffect::TakeExtraTurn,
            ..
        })
    ));
    fellowship.apply(Action::AcceptQuestBonus).unwrap();
    assert!(
        fellowship
            .quest
            .bonuses
            .iter()
            .filter(|bonus| [7, 10, 13].contains(&bonus.position))
            .all(|bonus| bonus.claimed)
    );

    let mut sauron = Game::new(0);
    sauron.quest.fellowship_position = 2;
    sauron.quest.sauron_position = 13;
    sauron.advance_quest(Faction::Sauron, 1);
    assert!(!sauron.is_over());
    sauron.advance_quest(Faction::Sauron, 1);
    assert_eq!(sauron.winner(), Some(Faction::Sauron));

    let mut fellowship_win = Game::new(0);
    fellowship_win.quest.fellowship_position = 13;
    fellowship_win.quest.sauron_position = 2;
    fellowship_win.advance_quest(Faction::Fellowship, 1);
    assert!(!fellowship_win.is_over());
    fellowship_win.advance_quest(Faction::Fellowship, 1);
    assert_eq!(fellowship_win.winner(), Some(Faction::Fellowship));
}

#[test]
fn quest_unit_and_fortress_choices_can_change_map_outcomes() {
    let mut fortress = Game::new(0);
    fortress
        .map
        .iter_mut()
        .find(|state| state.region == Region::Arnor)
        .unwrap()
        .fortress = Some(Faction::Fellowship);
    fortress.effect_queue.push_back(Effect::ResolveQuestBonus {
        player: Faction::Sauron,
        effect: QuestBonusEffect::RemoveEnemyFortress,
    });
    fortress.resolve_effects();
    fortress
        .apply(Action::RemoveQuestBonusFortress {
            region: Region::Arnor,
        })
        .unwrap();
    assert_eq!(
        fortress
            .map
            .iter()
            .find(|state| state.region == Region::Arnor)
            .unwrap()
            .fortress,
        None
    );

    let mut domination = Game::new(0);
    for region in REGIONS
        .into_iter()
        .filter(|region| *region != Region::Lindon)
    {
        domination
            .map
            .iter_mut()
            .find(|state| state.region == region)
            .unwrap()
            .fortress = Some(Faction::Fellowship);
    }
    domination
        .effect_queue
        .push_back(Effect::ResolveQuestBonus {
            player: Faction::Fellowship,
            effect: QuestBonusEffect::PlaceUnit,
        });
    domination.resolve_effects();
    domination
        .apply(Action::PlaceQuestBonusUnit {
            region: Region::Lindon,
        })
        .unwrap();
    assert_eq!(domination.winner(), Some(Faction::Fellowship));
}

#[test]
fn race_triggers_reveal_return_and_restrict_rewards() {
    let mut pair = Game::new(0);
    pair.sauron.tableau.extend([card(1, 17), card(1, 17)]);
    pair.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    pair.resolve_effects();
    let pair_revealed = match pair.pending_decision.clone().unwrap() {
        PendingDecision::AllianceTokenChoice { revealed, .. } => revealed,
        _ => panic!("expected token choice"),
    };
    pair.apply(Action::TakeAllianceToken {
        token: pair_revealed[0],
    })
    .unwrap();
    assert!(
        pair.alliance_stacks
            .iter()
            .find(|stack| stack.race == Race::Humans)
            .unwrap()
            .token_order
            .contains(&pair_revealed[1].id)
    );

    let mut game = Game::new(0);
    game.sauron
        .tableau
        .extend([card(1, 17), card(1, 17), card(1, 18), card(1, 19)]);
    game.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    game.resolve_effects();
    assert_eq!(
        game.legal_actions(),
        vec![
            Action::ResolveAllianceTrigger {
                trigger: AllianceTrigger::MatchingRace(Race::Humans)
            },
            Action::ResolveAllianceTrigger {
                trigger: AllianceTrigger::ThreeDifferentRaces
            },
        ]
    );
    game.apply(Action::ResolveAllianceTrigger {
        trigger: AllianceTrigger::MatchingRace(Race::Humans),
    })
    .unwrap();
    let revealed = match game.pending_decision.clone().unwrap() {
        PendingDecision::AllianceTokenChoice { revealed, .. } => revealed,
        _ => panic!("expected token choice"),
    };
    assert_eq!(revealed.len(), 2);
    let kept = revealed[0];
    game.apply(Action::TakeAllianceToken { token: kept })
        .unwrap();
    assert!(game.sauron.alliance_tokens.contains(&kept));
    assert!(game.sauron.claimed_three_race_alliance);
    let choice = game.legal_actions()[0].clone();
    game.apply(choice).unwrap();
    game.effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    game.resolve_effects();
    assert!(
        !game
            .legal_actions()
            .contains(&Action::ResolveAllianceTrigger {
                trigger: AllianceTrigger::ThreeDifferentRaces
            })
    );

    let mut limited = Game::new(0);
    limited
        .alliance_stacks
        .iter_mut()
        .find(|stack| stack.race == Race::Elves)
        .unwrap()
        .token_order = vec![2];
    limited.pending_decision = Some(PendingDecision::AllianceRaceChoice {
        player: Faction::Sauron,
    });
    limited
        .apply(Action::ChooseAllianceRace { race: Race::Elves })
        .unwrap();
    assert_eq!(
        limited.legal_actions(),
        vec![Action::TakeAllianceToken {
            token: token(Race::Elves, 2)
        }]
    );
    assert!(limited.legal_actions_for(Faction::Fellowship).is_empty());
    assert!(
        !limited
            .render_observation_for(Faction::Fellowship)
            .contains("token_order")
    );
}

#[test]
fn all_six_races_support_victory_and_every_immediate_token_is_observable() {
    let mut races = Game::new(0);
    races.sauron.tableau.extend([
        card(1, 17),
        card(1, 18),
        card(1, 19),
        card(1, 20),
        card(3, 8),
        card(3, 10),
    ]);
    assert_eq!(races.player_races(Faction::Sauron), RACES);
    races
        .effect_queue
        .push_back(Effect::ResolveRaceAllianceTriggers {
            player: Faction::Sauron,
        });
    races.resolve_effects();
    assert_eq!(races.winner(), Some(Faction::Sauron));

    let mut ents = Game::new(0);
    resolve_token(&mut ents, token(Race::Ents, 1));
    assert_eq!(ents.extra_turns, 1);
    resolve_token(&mut ents, token(Race::Ents, 2));
    assert!(matches!(
        ents.pending_decision,
        Some(PendingDecision::QuestBonusFortressRemoval {
            optional: false,
            ..
        })
    ));
    let mut maneuvers = Game::new(0);
    resolve_token(&mut maneuvers, token(Race::Ents, 3));
    assert!(matches!(
        maneuvers.pending_decision,
        Some(PendingDecision::EntManeuverChoice { remaining: 3, .. })
    ));
    let mut wizard_quest = Game::new(0);
    resolve_token(&mut wizard_quest, token(Race::Wizards, 1));
    assert_eq!(wizard_quest.quest.sauron_position, 2);
    let mut wizard_units = Game::new(0);
    resolve_token(&mut wizard_units, token(Race::Wizards, 2));
    assert!(matches!(
        wizard_units.pending_decision,
        Some(PendingDecision::AllianceUnitPlacement { remaining: 2, .. })
    ));
    let mut wizard_discard = Game::new(0);
    wizard_discard.chapters[0].discarded.push(card(1, 9));
    resolve_token(&mut wizard_discard, token(Race::Wizards, 3));
    assert!(matches!(
        wizard_discard.pending_decision,
        Some(PendingDecision::DiscardedCardFreePlay { .. })
    ));
}

#[test]
fn persistent_tokens_modify_observable_actions_and_reset_turn_scoped_skill_use() {
    let mut game = Game::new(0);
    game.sauron.alliance_tokens.extend([
        token(Race::Elves, 1),
        token(Race::Elves, 2),
        token(Race::Elves, 3),
        token(Race::Dwarves, 1),
        token(Race::Dwarves, 2),
        token(Race::Dwarves, 3),
        token(Race::Hobbits, 1),
        token(Race::Hobbits, 2),
        token(Race::Hobbits, 3),
        token(Race::Humans, 1),
        token(Race::Humans, 2),
        token(Race::Humans, 3),
    ]);
    assert!(game.can_use_any_skill(Faction::Sauron));
    game.sauron.coins = 10;
    assert_eq!(game.landmark_payment("helms_deep"), Some(6));
    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 21) });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(game.legal_actions().len(), REGIONS.len());
    game.apply(Action::PlaceCardUnits {
        region: Region::Lindon,
    })
    .unwrap();
    assert_eq!(game.region_unit_count(Faction::Sauron, Region::Lindon), 2);
    game.active_player = Faction::Sauron;
    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 9) });
    game.apply(Action::PlaySelectedCard).unwrap();
    assert_eq!(game.quest.sauron_position, 1);
    assert_eq!(game.active_player(), Faction::Sauron);
    game.pending_decision = Some(PendingDecision::CardDisposition { card: card(1, 13) });
    game.apply(Action::PlaySelectedCardUsingAnySkill).unwrap();
    game.apply(Action::PlaceAllianceUnits {
        region: Region::Gondor,
    })
    .unwrap();
    assert!(!game.sauron.used_any_skill_this_turn);
}
