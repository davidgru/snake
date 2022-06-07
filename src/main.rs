extern crate terminal;
extern crate rand;

use terminal::{Clear, Action};
use std::io::Write;

use rand::Rng;
use std::collections::LinkedList;


const EMPTY: u8 = ' ' as u8;
const BORDER: u8 = '#' as u8;
const FOOD: u8 = 'F' as u8;
const HEAD: u8 = '@' as u8;
const BODY: u8 = 'B' as u8;


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

fn board_height(height: usize) -> usize {
    height + 2
}

fn board_width(width: usize) -> usize {
    width + 3
}

fn draw_border(board: &mut Vec<u8>, width: usize, height: usize) {
    board.clear();
    board.resize(board_height(height) * board_width(width), EMPTY);
    for w in 0..width {
        board[w + 1] = BORDER;
        board[(board_height(height) - 1) * board_width(width) + w + 1] = BORDER;
    }

    for h in 0..board_height(height) {
        board[h * board_width(width) + 0] = BORDER;
        board[h * board_width(width) + width + 1] = BORDER;
        board[h * board_width(width) + width + 2] = '\n' as u8;
    }
}

fn draw_snake(board: &mut Vec<u8>, width: usize, snake: &LinkedList<(usize, usize)>) {
    let mut it = snake.into_iter();
    if let Some((h, w)) = it.next() {
        board[h * board_width(width) + w] = HEAD;
    } else {
        panic!("Snake has no head!");
    }
    while let Some((h, w)) = it.next() {
        board[h * board_width(width) + w] = BODY;
    }
}

fn advance_snake(board: &mut Vec<u8>, width: usize, snake: &mut LinkedList<(usize, usize)>, direction: &(usize, usize)) -> (bool, bool) {
    if let Some((h, w)) = snake.front() {
        let new_head_h = h + direction.0;
        let new_head_w = w + direction.1;

        snake.push_front((new_head_h, new_head_w));

        let out = match board[new_head_h * board_width(width) + new_head_w] {
            FOOD => (true, false),
            BORDER => (false, true),
            BODY => (false, true),
            EMPTY => (false, false),
            _ => panic!("Impossible")
        };
        if !out.0 {
            snake.pop_back();
        }
        out
    } else {
        panic!("Snake has no head!");
    }
}

fn draw_food(board: &mut Vec<u8>, width: usize, food: &(usize, usize)) {
    board[food.0 * board_width(width) + food.1] = FOOD;
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

    let board_size = board_height(height) * board_width(width);

    let mut term = terminal::stdout();

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_size);

    let mut food: (usize, usize);
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();
    let mut direction: (usize, usize) = (0, 1);

    draw_border(&mut board, width, height);
    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, width, &snake);
    food = random_free_spot(&board, width);
    draw_food(&mut board, width, &food);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let (eaten, crashed) = advance_snake(&mut board, width, &mut snake, &direction);

        draw_border(&mut board, width, height);
        draw_snake(&mut board, width, &snake);
        if eaten {
            food = random_free_spot(&board, width);
        }
        draw_food(&mut board, width, &food);

        // display
        if term.batch(Action::ClearTerminal(Clear::All)).is_err() ||
            term.batch(Action::MoveCursorTo(0, 0)).is_err() ||
            term.flush_batch().is_err() ||
            term.write(board.as_slice()).is_err() {
            std::process::exit(1);
        }

        if crashed {
            break;
        }
    }

}
