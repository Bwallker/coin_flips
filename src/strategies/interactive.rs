use std::io::{stdin, stdout, Write};
use std::process::exit;

use Move::{Empty, Invalid, Valid};

use crate::ValidMove::{Flip, Guess, TryAgain};
use crate::{Cheating, Honest, PermanentState, RoundState, Suspect, ValidMove};

enum Move<'a> {
    Valid(ValidMove),
    Invalid(&'a str),
    Empty,
}
macro_rules! honest_pattern {
    () => {
        "h" | "honest"
    };
}

macro_rules! cheating_pattern {
    () => {
        "c" | "cheater"
    };
}

pub fn interactive(
    permanent_state: &mut PermanentState,
    suspect: Suspect,
    round_state: &mut RoundState,
    _moves_made: &[ValidMove],
) -> ValidMove {
    println!();
    println!();
    println!("Make your next move!");
    println!("Coin flips left: {}", permanent_state.remaining_coin_flips);
    println!(
        "Heads performed this round: {}",
        round_state.amount_of_heads_flipped
    );
    println!(
        "Tails performed this round: {}",
        round_state.amount_of_tails_flipped
    );
    println!(
        "Total flips performed this round: {}",
        round_state.total_flips()
    );
    println!("Available commands: ");
    println!("guess | g --- guess if the suspect is playing honest or is cheating. Possible subcommands are [h, honest] if you think they are honest or [c, cheater] if you think they are cheating. Eg type g c if you think they are cheating.");
    println!("flip | f --- flip the coin x times where x is the first subcommand. eg type f 100 if you wish to flip the 100 times.");
    println!();
    println!();
    print!("> ");
    let _ = stdout().flush();
    let mut buffer = String::new();
    let stdin = stdin();
    let res = stdin.read_line(&mut buffer);
    if let Err(e) = res {
        println!(
            "There was an error reading in your response from STDIN. Exiting with error code 1."
        );
        println!("Here is the error: {e}");
        exit(1);
    }
    buffer.make_ascii_lowercase();
    let input = buffer.trim();
    let handler = get_handler(input);
    match handler(input, suspect, permanent_state) {
        Empty => {
            println!("Your response contained nothing but whitespace. try again!");
            TryAgain
        }
        Invalid(x) => {
            println!("Your input could not be parsed: it was: \"{x}\"");
            TryAgain
        }
        Valid(x) => x,
    }
}

type Handler<'a> = dyn Fn(&'a str, Suspect, &PermanentState) -> Move<'a>;

fn get_handler(input: &str) -> &Handler {
    match input.split_whitespace().next() {
        None => &handle_empty_command,
        Some(v) => match v {
            "flip" | "f" => &handle_flip,
            "guess" | "g" => &handle_guess,
            _ => &handle_invalid_command,
        },
    }
}

fn handle_guess<'a>(
    input: &'a str,
    suspect: Suspect,
    _permanent_state: &PermanentState,
) -> Move<'a> {
    let guess = input.split_whitespace().nth(1);
    match guess {
        None => {
            println!("You must include who you think the suspect is in your guess command.");
            Invalid(input)
        }
        Some(v) => match v {
            cheating_pattern!() => Valid(Guess(Cheating {
                probability_of_heads: suspect.get_probability(),
            })),
            honest_pattern!() => Valid(Guess(Honest)),
            x => {
                println!("{x} is an invalid guess.");
                Invalid(input)
            }
        },
    }
}

fn handle_flip<'a>(
    input: &'a str,
    _suspect: Suspect,
    permanent_state: &PermanentState,
) -> Move<'a> {
    let amount_of_flips = input.split_whitespace().nth(1).map(|x| x.parse::<i32>());
    match amount_of_flips {
        None => {
            println!("You must include the number of flips to perform in your command.");
            Invalid(input)
        }
        Some(res) => {
            match res {
                Ok(val) => {
                    if val <= 0 {
                        println!("The amount of flips to perform must be a positive integer. It was {val}");
                        Invalid(input)
                    } else if val > permanent_state.remaining_coin_flips {
                        println!("The amount of flips to perform must less than or equal to the amount of remaining flips. It was {val} and the remaining flips were {}", permanent_state.remaining_coin_flips);
                        Invalid(input)
                    } else {
                        Valid(Flip(val))
                    }
                }
                Err(_parse_error) => {
                    println!("The amount of flips to perform must be a valid positive integer less than 2 billion.");
                    Invalid(input)
                }
            }
        }
    }
}

fn handle_invalid_command<'a>(
    input: &'a str,
    _suspect: Suspect,
    _permanent_state: &PermanentState,
) -> Move<'a> {
    Invalid(input)
}

fn handle_empty_command<'a>(
    _input: &'a str,
    _suspect: Suspect,
    _permanent_state: &PermanentState,
) -> Move<'a> {
    Empty
}
