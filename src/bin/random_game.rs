use duel_for_middle_earth_rules::{Faction, Game};

const DEFAULT_SEED: u64 = 0;
const DEFAULT_MAX_ACTIONS: u32 = 500;

fn main() {
    let (seed, max_actions) = arguments();
    let mut game = Game::new(seed);
    let mut rng = seed ^ 0xC0FF_EE;

    println!("random game: seed={seed}, max_actions={max_actions}\n");
    println!("{}\n", game.render_observation_for(game.active_player()));

    let mut completed_actions = 0;
    for step in 1..=max_actions {
        if game.is_over() {
            break;
        }
        let Some((player, actions)) = acting_player(&game) else {
            println!("stalled after {} action(s): no legal action", step - 1);
            break;
        };
        rng = next_random(rng);
        let action = actions[(rng % actions.len() as u64) as usize].clone();
        println!("{step}. {player}: {action}");
        game.apply_for(player, action)
            .expect("action returned by legal_actions_for must apply");
        completed_actions = step;
        println!("{}\n", game.render_observation_for(game.active_player()));
    }

    if game.is_over() {
        println!(
            "game over after {} turns: {:?}",
            game.turn(),
            game.victory_type()
        );
    } else if completed_actions == max_actions {
        println!("stopped at the {max_actions}-action guard");
    }
}

fn acting_player(game: &Game) -> Option<(Faction, Vec<duel_for_middle_earth_rules::Action>)> {
    [Faction::Fellowship, Faction::Sauron]
        .into_iter()
        .find_map(|player| {
            let actions = game.legal_actions_for(player);
            (!actions.is_empty()).then_some((player, actions))
        })
}

fn next_random(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

fn arguments() -> (u64, u32) {
    let mut arguments = std::env::args().skip(1);
    let seed = arguments
        .next()
        .map(|argument| argument.parse().expect("seed must be an unsigned integer"))
        .unwrap_or(DEFAULT_SEED);
    let max_actions = arguments
        .next()
        .map(|argument| {
            argument
                .parse()
                .expect("max actions must be an unsigned integer")
        })
        .unwrap_or(DEFAULT_MAX_ACTIONS);
    if arguments.next().is_some() {
        panic!("usage: cargo run --bin random_game -- [seed] [max-actions]");
    }
    (seed, max_actions)
}
