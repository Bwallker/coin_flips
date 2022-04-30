use crate::Suspect;
use crate::ValidMove::Guess;
use crate::{Cheating, Honest, RoundState};
use crate::{PermanentState, ValidMove};
use rand;
use rand::Rng;

pub fn random_guess(
    _permanent_state: &mut PermanentState,
    suspect: Suspect,
    _round_state: &mut RoundState,
    _made_moves: &[ValidMove],
) -> ValidMove {
    let prob = rand::thread_rng().gen_bool(0.5);
    if prob {
        Guess(Honest)
    } else {
        Guess(Cheating {
            probability_of_heads: suspect.get_probability(),
        })
    }
}
