use duel_for_middle_earth_rules::{Faction, Game};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game: Option<Game> = None;
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                reply(&mut stdout, format!("error: input: {error}"));
                continue;
            }
        };
        let mut words = line.split_whitespace();
        let command = match words.next() {
            Some(command) => command,
            None => continue,
        };
        let response = match command {
            "new" => match words.next().map(str::parse).transpose() {
                Ok(seed) => {
                    game = Some(Game::new(seed.unwrap_or(0)));
                    "game created; use `state <fellowship|sauron>`".into()
                }
                Err(_) => "error: seed must be an unsigned integer".into(),
            },
            "state" | "legal" => observation(&game, words.next()),
            "choose" => choose(&mut game, words.next(), words.next()),
            "help" => "commands: new [seed] | state <fellowship|sauron> | choose <fellowship|sauron> <action-number> | help | quit".into(),
            "quit" => break,
            _ => "error: unknown command (use 'help')".into(),
        };
        reply(&mut stdout, response);
    }
}

fn observation(game: &Option<Game>, player: Option<&str>) -> String {
    let Some(game) = game.as_ref() else {
        return no_game();
    };
    let Some(player) = player.and_then(|value| value.parse::<Faction>().ok()) else {
        return "error: state requires fellowship or sauron".into();
    };
    game.render_observation_for(player)
}
fn choose(game: &mut Option<Game>, player: Option<&str>, action_number: Option<&str>) -> String {
    let Some(game) = game.as_mut() else {
        return no_game();
    };
    let Some(player) = player.and_then(|value| value.parse::<Faction>().ok()) else {
        return "error: choose requires fellowship or sauron".into();
    };
    let Some(action_number) = action_number.and_then(|word| word.parse::<usize>().ok()) else {
        return "error: choose requires an action number from that player's latest response".into();
    };
    let Some(action) = game.legal_actions_for(player).get(action_number).cloned() else {
        return "error: action number is not legal in the current state".into();
    };
    match game.apply_for(player, action) {
        Ok(()) => game.render_observation_for(player),
        Err(error) => format!("error: {error}"),
    }
}
fn no_game() -> String {
    "error: start a game with 'new [seed]'".into()
}
fn reply(stdout: &mut impl Write, message: String) {
    writeln!(stdout, "{message}\n.").expect("stdout must be writable");
    stdout.flush().expect("stdout must be flushable");
}
