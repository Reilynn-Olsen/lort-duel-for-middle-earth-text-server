use std::collections::{BTreeMap, BTreeSet};

use crate::catalog::chapter_cards;
use crate::*;

use super::*;

const NORMAL_GAMES: u32 = 250;
const MAX_ACTIONS: u32 = 500;

fn check_invariants(
    game: &Game,
    previous: Option<&Game>,
    action: Option<&Action>,
) -> Result<(), String> {
    if game.fellowship.coins as u16 + game.sauron.coins as u16 + game.coin_reserve as u16
        != TOTAL_COINS as u16
    {
        return Err("coins do not sum to the physical reserve".into());
    }
    for faction in [Faction::Fellowship, Faction::Sauron] {
        let units = game
            .map
            .iter()
            .map(|region| game.region_unit_count(faction, region.region) as u16)
            .sum::<u16>();
        if units + game.player(faction).units_in_supply as u16 != 15 {
            return Err(format!("{faction} has an invalid Unit total"));
        }
        let fortresses = game
            .map
            .iter()
            .filter(|region| region.fortress == Some(faction))
            .count() as u8;
        if fortresses + game.player(faction).fortresses_in_supply != 7 {
            return Err(format!("{faction} has an invalid Fortress total"));
        }
    }
    if game
        .map
        .iter()
        .any(|region| region.fellowship_units > 0 && region.sauron_units > 0)
    {
        return Err("opposing Units remain after resolution".into());
    }
    if let (Some(previous), Some(Action::MoveCardUnit { .. })) = (previous, action) {
        if previous
            .map
            .iter()
            .map(|region| region.fortress)
            .ne(game.map.iter().map(|region| region.fortress))
        {
            return Err("ordinary movement moved a Fortress".into());
        }
    }

    // Covering-slot IDs are immutable layout dependencies. A removed ID remains
    // recorded specifically so availability can prove it no longer covers a card.
    let present_slots = game
        .current_chapter_state()
        .layout
        .iter()
        .map(|slot| slot.id)
        .collect::<BTreeSet<_>>();
    let expected_available = game
        .current_chapter_state()
        .layout
        .iter()
        .filter(|slot| {
            slot.visibility == CardVisibility::FaceUp
                && slot
                    .covering_slots
                    .iter()
                    .all(|cover| !present_slots.contains(cover))
        })
        .map(|slot| slot.id)
        .collect::<Vec<_>>();
    if game.available_chapter_slot_ids() != expected_available {
        return Err("available cards are not exactly face-up uncovered cards".into());
    }

    let mut cards = BTreeMap::new();
    for chapter in &game.chapters {
        for card in chapter_cards(chapter.chapter) {
            cards.insert(card.to_string(), 0u8);
        }
    }
    for chapter in &game.chapters {
        for card in chapter
            .layout
            .iter()
            .map(|slot| slot.card)
            .chain(chapter.discarded.iter().copied())
        {
            *cards.entry(card.to_string()).or_default() += 1;
        }
    }
    for player in [&game.fellowship, &game.sauron] {
        for card in &player.tableau {
            *cards.entry(card.to_string()).or_default() += 1;
        }
    }
    if let Some(PendingDecision::CardDisposition { card }) = game.pending_decision {
        *cards.entry(card.to_string()).or_default() += 1;
    }
    if cards.values().any(|count| *count != 1) {
        return Err("a Chapter card is not in exactly one valid location".into());
    }

    let mut landmarks = BTreeMap::new();
    for id in crate::catalog::landmark_ids() {
        landmarks.insert(id, 0u8);
    }
    for id in game
        .landmarks
        .available
        .iter()
        .chain(game.landmarks.facedown_deck.iter())
        .chain(game.fellowship.landmarks.iter())
        .chain(game.sauron.landmarks.iter())
    {
        *landmarks.entry(id.clone()).or_default() += 1;
    }
    if landmarks.values().any(|count| *count != 1) {
        return Err("a Landmark is not in exactly one valid location".into());
    }

    let mut tokens = BTreeMap::new();
    for race in RACES {
        for id in 1..=3 {
            tokens.insert(format!("{race}:{id}"), 0u8);
        }
    }
    for stack in &game.alliance_stacks {
        for id in &stack.token_order {
            *tokens.entry(format!("{}:{id}", stack.race)).or_default() += 1;
        }
    }
    for player in [&game.fellowship, &game.sauron] {
        for token in &player.alliance_tokens {
            *tokens
                .entry(format!("{}:{}", token.race, token.id))
                .or_default() += 1;
        }
    }
    if let Some(PendingDecision::AllianceTokenChoice { ref revealed, .. }) = game.pending_decision {
        for token in revealed {
            *tokens
                .entry(format!("{}:{}", token.race, token.id))
                .or_default() += 1;
        }
    }
    if tokens.values().any(|count| *count != 1) {
        return Err("an Alliance token is not in exactly one valid location".into());
    }

    if game
        .quest
        .bonuses
        .iter()
        .map(|bonus| bonus.position)
        .collect::<BTreeSet<_>>()
        .len()
        != game.quest.bonuses.len()
    {
        return Err("a one-time quest bonus position is duplicated".into());
    }
    if let Some(decision_player) = game.decision_player() {
        if game.legal_actions_for(decision_player.other()).len() != 0 {
            return Err("pending choice is exposed to the wrong player".into());
        }
    }
    if game.is_over() {
        if !game.legal_actions().is_empty() {
            return Err("finished game accepts gameplay actions".into());
        }
    } else if !matches!(game.active_player, Faction::Fellowship | Faction::Sauron) {
        return Err("unfinished game has no active player".into());
    }
    Ok(())
}

#[derive(Default)]
struct Statistics {
    victories: BTreeMap<String, u32>,
    chapters: BTreeMap<u8, u32>,
    lengths: BTreeMap<u32, u32>,
    stalled: u32,
}

fn simulate(games: u32) {
    let mut statistics = Statistics::default();
    for seed in 0..games as u64 {
        let mut game = Game::new(seed);
        let mut rng = seed ^ 0xC0FFEE;
        let mut log = Vec::new();
        check_invariants(&game, None, None)
            .unwrap_or_else(|error| panic!("seed {seed}: {error}\n{}", game.render_state()));
        for _ in 0..MAX_ACTIONS {
            if game.is_over() {
                break;
            }
            let actions = game.legal_actions();
            if actions.is_empty() {
                statistics.stalled += 1;
                break;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let action = actions[(rng % actions.len() as u64) as usize].clone();
            let before = game.clone();
            log.push(action.to_string());
            game.apply(action.clone()).unwrap();
            if let Err(error) = check_invariants(&game, Some(&before), Some(&action)) {
                panic!(
                    "seed {seed}: {error}\nactions:\n{}\nstate:\n{}",
                    log.join("\n"),
                    game.render_state()
                );
            }
        }
        *statistics.chapters.entry(game.current_chapter).or_default() += 1;
        *statistics.lengths.entry(game.turn).or_default() += 1;
        let outcome = game
            .victory_type
            .map(|victory| victory.to_string())
            .unwrap_or_else(|| "unfinished".into());
        *statistics.victories.entry(outcome).or_default() += 1;
    }
    eprintln!(
        "random legal-game statistics: games={games}, victories={:?}, chapters={:?}, lengths={:?}, stalled={}",
        statistics.victories, statistics.chapters, statistics.lengths, statistics.stalled
    );
}

#[test]
fn seeded_legal_games_preserve_invariants() {
    simulate(NORMAL_GAMES);
}

#[test]
#[ignore = "stress test: run `cargo test ten_thousand_seeded_legal_games_preserve_invariants -- --ignored --nocapture`"]
fn ten_thousand_seeded_legal_games_preserve_invariants() {
    simulate(10_000);
}
