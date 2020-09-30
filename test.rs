//////////////////////////////// CODE ADDED IN BOARD.RS /////////////////////////////

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            _ => Direction::Down,
        }
    }
} 


//////////////////////////////// CODE IN TEST RUN IN MAIN.RS /////////////////////

use rand::prelude::*;
use rayon::prelude::*;

use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

const TURNS: u32 = 100000;
const MAX_STALL: u64 = 2000;
const TESTS: u128 = 50;
const MAX_DEEPTH: u128 = 300;
const INITIAL_DIRECTIONAL_MOVES: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

const BAD: [usize; 3] = [3, 7, 11];

fn mcs_test() -> std::io::Result<()> {
    #[rustfmt::skip]
    let board: Board = Board::from(vec![
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
    ]);

    let mut game = GameBuilder::default()
        .initial_board(board)
        .proba_4(0.1)
        .build();
    game.populate_new_tile();
    game.populate_new_tile();

    let initial_board = game.board;

    let test_vec: Vec<Vec<_>> = (0..MAX_DEEPTH)
        .into_par_iter()
        .map(|depth| {
            (0..TESTS)
                .into_par_iter()
                .map(|_test| (depth, calculate_score(single_test(initial_board, depth))))
                .collect()
        })
        .collect();
    let mut file = File::create(format!("{:?}test.txt", SystemTime::now()))?;
    write!(file, "{:#?}", test_vec)?;
    Ok(())
}

fn direction_chooser(
    rng_thread: &mut ThreadRng,
    depth: u128,
    currentboard: Board,
) -> Option<Direction> {
    let moves: Vec<Vec<Direction>> = (0..4)
        .map(|_| (0..depth).map(|_| rng_thread.gen()).collect())
        .collect();

    let found_scores: Vec<_> = moves
        .into_par_iter()
        .zip(INITIAL_DIRECTIONAL_MOVES.into_par_iter())
        .map(|(moveset, initial_move)| {
            (
                initial_move,
                reached_score_stuck(currentboard, *initial_move, moveset),
            )
        })
        .collect();
    // Find the max score
    let mut max_index = 0;

    for (index, &value) in found_scores.iter().enumerate() {
        if &value.1 > &found_scores[max_index].1 {
            max_index = index
        }
    }
    if found_scores[max_index].1 == 0 {
        return None;
    } else {
        return Some(*found_scores[max_index].0);
    }
}

fn headless_play_stuck(game: &mut Game, direction: Direction) -> bool {
    let previous_board = game.board;
    game.play(direction);
    if previous_board == game.board {
        if previous_board.count_empty_tiles() == 0 {
            return gameover(previous_board.into());
        }
    }
    game.populate_new_tile();
    return false;
}

fn reached_score_stuck(board: Board, initial_move: Direction, moves: Vec<Direction>) -> u128 {
    let mut bad_path = false;
    let mut tmpgame: Game = GameBuilder::default()
        .initial_board(board)
        .proba_4(0.1)
        .build();
    headless_play(&mut tmpgame, initial_move);
    for direction in moves.into_iter() {
        if headless_play_stuck(&mut tmpgame, direction) {
            bad_path = true;
            break;
        }
    }
    if bad_path {
        return 0;
    } else {
        let vec: Vec<u16> = tmpgame.board.into();
        //    return vec.into_iter().map(|tile| {let a = tile as u128; if a == 0 {return 1000} else {return a}}).sum()}
        return calculate_score(vec) as u128;
    }
}

fn gameover(v: Vec<u16>) -> bool {
    let mut j = (0..15).into_iter();
    let mut i = (0..12).into_iter();

    //   let row_pair = ((v[i]),(v[j+1]));
    //    let coloumn_pair = ((v[i]),(v[j+4]));
    //    println!("Row pair: {:#?} \n Coloumn pair{:#?}",row_pair,
    //    coloumn_pair);
    loop {
        match (j.next(), i.next()) {
            (Some(n), Some(k)) => {
                if (v[n] == v[n + 1] && !BAD.contains(&n)) || v[n] == v[k + 4] {
                    return false;
                }
            }
            (Some(n), None) => {
                if v[n] == v[n + 1] {
                    return false;
                }
            }
            (None, None) => break,
            _ => break,
        };
    }
    return true;
}

fn calculate_score(board: Vec<u16>) -> u32 {
    return board.iter().fold(0u32, |acc, tile| {
        acc + match *tile as u32 {
            4 => 4,
            8 => 16,
            16 => 48,
            32 => 128,
            64 => 320,
            128 => 768,
            256 => 1792,
            512 => 4096,
            1024 => 9216,
            2048 => 20480,
            4096 => 45056,
            8192 => 98304,
            16384 => 212992,
            _ => 0,
        }
    });
}
