extern crate terminal;
extern crate rand;

use terminal::{Clear, Action};
use std::io::Write;

use rand::Rng;
use std::collections::LinkedList;

fn print_usage() -> ! {
    println!("Usage");
    std::process::exit(1);
}

fn parse_arg<T: std::str::FromStr>(nth: usize) -> T {
    match std::env::args().nth(nth) {
        Some(arg) => match arg.parse::<T>() {
            Ok(targ) => targ,
            Err(_) => print_usage()
        },
        None => print_usage()
    }
}

const EMPTY: u8 = ' ' as u8;
const BORDER: u8 = '#' as u8;
const FOOD: u8 = 'F' as u8;
const HEAD: u8 = '@' as u8;
const BODY: u8 = 'B' as u8;

fn draw_snake(board: &mut Vec<u8>, width: usize, snake: &LinkedList<(usize, usize)>) {
    let mut it = snake.into_iter();
    if let Some((h, w)) = it.next() {
        board[h * width + w] = HEAD;
    } else {
        panic!("Snake has no head!");
    }
    while let Some((h, w)) = it.next() {
        board[h * width + w] = BODY;
    }
}

fn random_free_spot(board: &Vec<u8>, width: usize) -> (usize, usize) {
    let num_free = board.into_iter().filter(|c| -> bool {c == &&EMPTY}).count();
    let nth_free_i = rand::thread_rng().gen_range(0, num_free);
    let mut free_cnt = 0;
    for i in 0..board.len() {
        if board[i] == EMPTY {
            if free_cnt == nth_free_i {
                return (i / width, i % width)
            } else {
                free_cnt += 1;
            }
        }
    }
    panic!("How did I get here?");
}

fn main() {
    if std::env::args().count() != 3 {
        print_usage();
    }

    let width: usize = parse_arg(1);
    let height: usize = parse_arg(2);

    let board_width = width + 2 + 1; // plus border plus newline
    let board_height = height + 2; // plus border
    let board_size = board_height * board_width;

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_size);
    let food: (usize, usize);
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();

    board.resize(board_size, EMPTY);

    let mut term = terminal::stdout();

    // reset board

    for w in 0..width {
        board[w + 1] = BORDER;
        board[(board_height - 1) * board_width + w + 1] = BORDER;
    }

    for h in 0..board_height {
        board[h * board_width + 0] = BORDER;
        board[h * board_width + width + 1] = BORDER;
        board[h * board_width + width + 2] = '\n' as u8;
    }

    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, board_width, &snake);

    food = random_free_spot(&board, board_width);
    board[food.0 * board_width + food.1] = FOOD;

    if term.write(board.as_slice()).is_err() {
        std::process::exit(1);
    }

    
}
