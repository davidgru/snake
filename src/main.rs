extern crate clap;
extern crate rand;

mod term;
use term::{Input, Terminal};

use std::collections::LinkedList;

use clap::Parser;
use rand::Rng;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'w', long = "width", help = "The width of the board (default is terminal width)")]
    width: Option<usize>,
    #[clap(short = 'h', long = "height", help = "The height of the board (default is terminal height)")]
    height: Option<usize>,
    #[clap(short = 'f', long = "frequency", help = "The amount of steps the snake makes per second")]
    freq: Option<u64>
}

// Display constants
const EMPTY: u8 = ' ' as u8;
const BORDER: u8 = '#' as u8;
const FOOD: u8 = 'F' as u8;
const HEAD: u8 = '@' as u8;
const BODY: u8 = 'B' as u8;

enum Direction {
    Right, Up, Left, Down
}

impl Direction {
    pub fn velocity(&self) -> (i32, i32) {
        match *self {
            Direction::Right => (0, 1),
            Direction::Up => (-1, 0),
            Direction::Left => (0, -1),
            Direction::Down => (1, 0)
        }
    }
}

// height + space for border
fn board_height(height: usize) -> usize {
    height + 2
}

// width + space for border and \n\r
fn board_width(width: usize) -> usize {
    width + 4
}

// initialize the board and draw boarder and \n\r
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
        board[h * board_width(width) + width + 3] = '\r' as u8;
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

fn draw_food(terminal: &Terminal, board: &mut Vec<u8>, width: usize, food: &(usize, usize)) {
    board[food.0 * board_width(width) + food.1] = FOOD;
    terminal.write_cell(FOOD, food.1, food.0);
}

// move snake in direction and update board. return ({crashed into wall or myself}, {eaten food})
fn advance_snake(terminal: &Terminal, board: &mut Vec<u8>, width: usize, snake: &mut LinkedList<(usize, usize)>, direction: &Direction) -> (bool, bool) {
    if let Some(&(old_h, old_w)) = snake.front() {
        let new_head_h = (old_h as i32 + direction.velocity().0) as usize;
        let new_head_w = (old_w as i32 + direction.velocity().1) as usize;

        // advance
        snake.push_front((new_head_h, new_head_w));

        // check for collision
        let out = match board[new_head_h * board_width(width) + new_head_w] {
            FOOD => (true, false),
            BORDER => (false, true),
            BODY => (false, true),
            EMPTY => (false, false),
            _ => panic!("Impossible")
        };
        
        // write new head
        terminal.write_cell(HEAD, new_head_w, new_head_h);
        terminal.write_cell(BODY, old_w, old_h);
        board[new_head_h * board_width(width) + new_head_w] = HEAD;
        board[old_h * board_width(width) + old_w] = BODY;

        // remove tail if not eaten food
        if !out.0 {
            if let Some((h, w)) = snake.pop_back() {
                terminal.write_cell(EMPTY, w, h);
                board[h * board_width(width) + w] = EMPTY;
            }
        }
        out
    } else {
        panic!("Snake has no head!");
    }
}

// find random free spot on the board in O(n) guaranteed
fn random_free_spot(board: &Vec<u8>, width: usize) -> (usize, usize) {
    let num_free = board.into_iter().filter(|c| -> bool {**c == EMPTY}).count();
    let nth_free_i = rand::thread_rng().gen_range(0, num_free);
    let mut free_cnt = 0;
    for i in 0..board.len() {
        if board[i] == EMPTY {
            if free_cnt == nth_free_i {
                return (i / board_width(width), i % board_width(width))
            } else {
                free_cnt += 1;
            }
        }
    }
    panic!("How did I get here?");
}

fn main() {

    let args = Args::parse();
    let terminal = Terminal::new();

    // default is terminal width
    let width = args.width.unwrap_or_else(|| -> usize {terminal.get_size().unwrap().0 - 2});
    // default is terminal height
    let height = args.height.unwrap_or_else(|| -> usize {terminal.get_size().unwrap().1 - 3});
    // default is chosen so it takes the snake 4 seconds across the board 
    let freq = args.freq.unwrap_or_else(|| -> u64 {(std::cmp::min(width, height) / 4) as u64});

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_height(height) * board_width(width));
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();
    let mut direction = Direction::Right;
    let mut food: (usize, usize);


    terminal.setup();

    // only draw border once
    draw_border(&mut board, width, height);
    
    // draw snake and food the first time
    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, width, &snake);
    food = random_free_spot(&board, width);
    draw_food(&terminal, &mut board, width, &food);

    terminal.display(board.as_slice());

    loop {
        // input
        if let Some(user_input) = terminal.user_input((1000 * 1000 / freq) as u64) {
            direction = match (user_input, direction) {
                (Input::Right, Direction::Right) => Direction::Down,
                (Input::Right, Direction::Down) => Direction::Left,
                (Input::Right, Direction::Left) => Direction::Up,
                (Input::Right, Direction::Up) => Direction::Right,
                (Input::Left, Direction::Right) => Direction::Up,
                (Input::Left, Direction::Up) => Direction::Left,
                (Input::Left, Direction::Left) => Direction::Down,
                (Input::Left, Direction::Down) => Direction::Right,
                (Input::Exit, _) => break,
            };
        }

        // step: redraw snake and food if eaten
        let (eaten, crashed) = advance_snake(&terminal, &mut board, width, &mut snake, &direction);
        if eaten {
            food = random_free_spot(&board, width);
            draw_food(&terminal, &mut board, width, &food);
        }

        if crashed {
            break;
        }
    }

    terminal.clean();
}
