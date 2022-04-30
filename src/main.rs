extern crate core;

use std::fmt::{Debug, Display, Formatter};
use std::process::exit;

use rand::{thread_rng, Rng};

use crate::strategies::interactive::interactive;
use crate::strategies::random_guess::random_guess;
use crate::CoinFlip::{Heads, Tails};
use crate::Suspect::{Cheating, Honest};

mod strategies;

fn print_newlines(amount: u32) {
    for _ in 0..amount {
        println!();
    }
}

const PROBABILITY_OF_BEING_HONEST: f64 = 0.5;
const STARTING_COIN_FLIPS: i32 = 100;
const RIGHT_GUESS_REWARD: i32 = 15;
const WRONG_GUESS_PENALTY: i32 = 30;
const MIN_CHEATING_PROBABILITY: f64 = 0.5;
const MAX_CHEATING_PROBABILITY: f64 = 1.0;

#[derive(Debug, PartialEq, Eq)]
enum CoinFlip {
    Heads,
    Tails,
}

impl Display for CoinFlip {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Suspect {
    Honest,
    Cheating { probability_of_heads: f64 },
}

impl Debug for Suspect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Honest => "honest",
                _ => "cheating",
            }
        )
    }
}

impl Display for Suspect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Suspect {
    fn flip_coin(&self) -> CoinFlip {
        let prob = self.get_probability();
        if thread_rng().gen_bool(prob) {
            Heads
        } else {
            Tails
        }
    }
    fn get_probability(&self) -> f64 {
        match self {
            Honest => 0.5,
            Cheating {
                probability_of_heads,
            } => *probability_of_heads,
        }
    }
}

fn get_next_suspect() -> Suspect {
    let mut rng = rand::thread_rng();
    let is_honest = rng.gen_bool(PROBABILITY_OF_BEING_HONEST);
    if is_honest {
        Honest
    } else {
        let cheating_heads_probability: f64 =
            rng.gen_range(MIN_CHEATING_PROBABILITY..=MAX_CHEATING_PROBABILITY);
        Cheating {
            probability_of_heads: cheating_heads_probability,
        }
    }
}

#[derive(Debug)]
pub struct PermanentState {
    remaining_coin_flips: i32,
    score: i32,
    incorrect_guesses_so_far: i32,
    correct_guesses_so_far: i32,
    amount_of_cheating_suspects_so_far: i32,
    amount_of_honest_suspects_so_far: i32,
}

#[derive(Debug)]
pub struct RoundState {
    amount_of_heads_flipped: i32,
    amount_of_tails_flipped: i32,
}

impl RoundState {
    fn total_flips(&self) -> i32 {
        self.amount_of_heads_flipped + self.amount_of_tails_flipped
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ValidMove {
    Guess(Suspect),
    Flip(i32),
    TryAgain,
}

impl Display for ValidMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn run(strategy: &Strategy) {
    println!("Welcome to the cheating detector game");
    println!(
        "Your job is to guess whether the coin each suspect is using is biased towards heads."
    );
    println!("You can ask each suspect to flip their coin as many times as would like, but you only have a certain amount of total coin flips.");
    println!("You start with {STARTING_COIN_FLIPS} coin flips. Every time you guess right you gain {RIGHT_GUESS_REWARD} coin flips and each time you guess wrong you lose {WRONG_GUESS_PENALTY} coin flips.");
    print_newlines(2);
    let mut state = PermanentState {
        remaining_coin_flips: STARTING_COIN_FLIPS,
        score: 0,
        incorrect_guesses_so_far: 0,
        correct_guesses_so_far: 0,
        amount_of_cheating_suspects_so_far: 0,
        amount_of_honest_suspects_so_far: 0,
    };
    let mut all_moves_made = Vec::new();
    while state.remaining_coin_flips > 0 {
        play_round(
            strategy,
            &mut state,
            get_next_suspect(),
            &mut all_moves_made,
        );
    }
    print_newlines(2);
    println!("Ending game because you ran out of flips.");
    println!("Here are your stats:");
    println!("Score: {}", state.score);
    println!("Total wrong guesses: {}", state.incorrect_guesses_so_far);
    println!("Total right guesses: {}", state.correct_guesses_so_far);
    println!(
        "Total amount of cheating suspects: {}",
        state.amount_of_cheating_suspects_so_far
    );
    println!(
        "Total amount of honest suspects: {}",
        state.amount_of_honest_suspects_so_far
    );

    println!("Here are all the moves that were made:");
    for (round_number, round_moves) in all_moves_made.iter().enumerate() {
        println!("Printing moves for round number: {}", round_number + 1);
        for (item_number, round_move) in round_moves.iter().enumerate() {
            println!("\t{} - {round_move}", item_number + 1);
        }
    }
    println!("See you next time!");
}

type Strategy = dyn Fn(&mut PermanentState, Suspect, &mut RoundState, &[ValidMove]) -> ValidMove;

fn main() {
    let strategy = get_strategy();

    run(strategy);
}

fn get_strategy<'a>() -> &'a Strategy {
    let strategy = std::env::args().nth(1);
    if strategy.is_none() {
        println!("You need to supply a strategy as the second as the second argument to this program! Exiting with error code 4");
        exit(4);
    }
    let strategy = unsafe { strategy.unwrap_unchecked() };
    match strategy.trim() {
        "interactive" | "i" => &interactive,
        "random_guess" | "rg" => &random_guess,
        x => {
            println!("\"{x}\" is not a valid strategy!: Exiting with error code 3");
            exit(3);
        }
    }
}

fn play_round(
    strategy: &Strategy,
    permanent_state: &mut PermanentState,
    suspect: Suspect,
    all_moves_made: &mut Vec<Vec<ValidMove>>,
) {
    println!("New round! Your score so far is {}", permanent_state.score);
    let mut round_state = RoundState {
        amount_of_tails_flipped: 0,
        amount_of_heads_flipped: 0,
    };
    let mut moves_made_this_round = Vec::new();
    loop {
        let strategy_play: ValidMove = strategy(
            permanent_state,
            suspect,
            &mut round_state,
            &moves_made_this_round,
        );
        moves_made_this_round.push(strategy_play);
        println!("Strategy decided to play: {strategy_play}");
        match strategy_play {
            ValidMove::TryAgain => {
                println!("Strategy was unable to generate a valid move. Trying again.");
                continue;
            }
            ValidMove::Flip(num_of_flips) => {
                println!("Strategy chose to flip the coin {num_of_flips} times.");
                assert!(num_of_flips > 0);
                assert!(num_of_flips <= permanent_state.remaining_coin_flips);
                for i in 1..=num_of_flips {
                    println!("Performing flip number {i}");
                    let flip = suspect.flip_coin();
                    println!("Flipped {flip}.");
                    match flip {
                        Heads => {
                            round_state.amount_of_heads_flipped += 1;
                        }
                        Tails => {
                            round_state.amount_of_tails_flipped += 1;
                        }
                    }
                }
                println!("Flipped Heads {} times and tails {} times. Total amount of flips performed where {}.", round_state.amount_of_heads_flipped, round_state.amount_of_tails_flipped, round_state.total_flips());
                permanent_state.remaining_coin_flips -= num_of_flips;
            }
            ValidMove::Guess(guess) => {
                println!("Strategy chose to guess that the suspect is {guess}.");
                if guess == suspect {
                    println!("This was correct.");
                    permanent_state.correct_guesses_so_far += 1;
                    permanent_state.remaining_coin_flips += RIGHT_GUESS_REWARD;
                } else {
                    println!("This was incorrect");
                    permanent_state.incorrect_guesses_so_far += 1;
                    permanent_state.remaining_coin_flips -= WRONG_GUESS_PENALTY;
                }
                permanent_state.score += 1;
                if suspect == Honest {
                    permanent_state.amount_of_honest_suspects_so_far += 1;
                } else {
                    permanent_state.amount_of_cheating_suspects_so_far += 1;
                }
                break;
            }
        }
    }
    println!(
        "Cheating so far: {}",
        permanent_state.amount_of_cheating_suspects_so_far
    );
    println!(
        "Honest so far: {}",
        permanent_state.amount_of_honest_suspects_so_far
    );
    println!("Remaining flips:  {}", permanent_state.remaining_coin_flips);
    println!(
        "Wrong guesses so far: {}",
        permanent_state.incorrect_guesses_so_far
    );
    println!(
        "Right guesses for far: {}",
        permanent_state.correct_guesses_so_far
    );
    println!("Round ended!");
    all_moves_made.push(moves_made_this_round);
}

#[cfg(test)]
pub mod tests {
    use super::main;

    #[test]
    fn test_1() {
        main();
    }
}
