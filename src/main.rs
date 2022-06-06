extern crate terminal;
use terminal::{Clear, Action};
use std::io::Write;

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
    board.resize(board_size, EMPTY);


    let mut terminal = terminal::stdout();

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

    if terminal.write(board.as_slice()).is_err() {
        std::process::exit(1);
    }



}
