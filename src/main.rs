extern crate clap;
extern crate rand;

use std::collections::LinkedList;
use std::time::{Duration, Instant};
use std::cmp::{max, min};

use clap::Parser;
use rand::Rng;

mod term;
use term::{Input, Terminal};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'w', long = "width", help = "The width of the board (default is terminal width)")]
    width: Option<usize>,
    #[clap(short = 'h', long = "height", help = "The height of the board (default is terminal height)")]
    height: Option<usize>,
    #[clap(short = 'f', long = "frequency", help = "The amount of steps the snake makes per second")]
    freq: Option<u64>,
    #[clap(short = 's', long = "start_length", help = "The length the snake starts with")]
    start_length: Option<usize>,
    #[clap(short = 'r', long = "reward", help = "The length the snake gains from eating food")]
    reward: Option<usize>,
}

// Display constants

const EMPTY: u8 = ' ' as u8;
const BORDER: u8 = '#' as u8;
const FOOD: u8 = 'F' as u8;
const HEAD: u8 = '@' as u8;
const BODY: u8 = 'B' as u8;


#[derive(Clone, Copy)]
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
    terminal.write_cell(FOOD, food.1, food.0).unwrap();
}

// move snake in direction and update board. return ({crashed into wall or myself}, {eaten food})
fn advance_snake(terminal: &Terminal, board: &mut Vec<u8>, width: usize, snake: &mut LinkedList<(usize, usize)>, direction: &Direction, target_length: &mut usize, reward: usize) -> (bool, bool) {
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
        terminal.write_cell(HEAD, new_head_w, new_head_h).unwrap();
        terminal.write_cell(BODY, old_w, old_h).unwrap();
        board[new_head_h * board_width(width) + new_head_w] = HEAD;
        board[old_h * board_width(width) + old_w] = BODY;

        if out.0 { // eaten food
            *target_length += reward;
        }

        // remove tail if snake length is as big as target_length
        if snake.len() > *target_length {
            if let Some((h, w)) = snake.pop_back() {
                terminal.write_cell(EMPTY, w, h).unwrap();
                board[h * board_width(width) + w] = EMPTY;
            }
        }
        out
    } else {
        panic!("Snake has no head!");
    }
}

// find random free spot on the board in O(n) guaranteed
fn random_free_spot(board: &Vec<u8>, width: usize) -> Option<(usize, usize)> {
    let num_free = board.into_iter().filter(|c| -> bool {**c == EMPTY}).count();
    if num_free == 0 {
        return None;
    }
    let nth_free_i = rand::thread_rng().gen_range(0, num_free);
    let mut free_cnt = 0;
    for i in 0..board.len() {
        if board[i] == EMPTY {
            if free_cnt == nth_free_i {
                return Some((i / board_width(width), i % board_width(width)))
            } else {
                free_cnt += 1;
            }
        }
    }
    None
}

fn quit(term: &Terminal) -> ! {
    term.clean().unwrap();
    std::process::exit(0);
}

fn main() {

    let args = Args::parse();
    let terminal = Terminal::new();
    
    // default is terminal width / height
    let default_width = || -> usize {terminal.get_size().unwrap().0 - 2};
    let default_height = || -> usize {terminal.get_size().unwrap().1 - 3};

    let width = args.width.unwrap_or_else(default_width);
    let height = args.height.unwrap_or_else(default_height);
    
    // default is chosen so it takes the snake 4 seconds across the board 
    let freq = args.freq.unwrap_or(max(1, (min(width, height) / 4) as u64));
    let start_length = args.start_length.unwrap_or(5);
    let reward = args.reward.unwrap_or(5);

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_height(height) * board_width(width));
    let mut target_length = 1 + start_length;
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();
    let mut direction = Direction::Right;
    let mut food: (usize, usize);


    terminal.setup().unwrap();

    // only draw border once
    draw_border(&mut board, width, height);
    
    // draw snake and food the first time
    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, width, &snake);
    food = match random_free_spot(&board, width) {
        Some(food) => food,
        None => quit(&terminal)
    };

    draw_food(&terminal, &mut board, width, &food);

    terminal.display(board.as_slice()).unwrap();


    let update_interval = Duration::from_micros((1000 * 1000) / freq);
    let mut update_deadline = Instant::now() + update_interval;

    let mut pause = false;

    loop {
        // user input
        if let Ok(Some(key)) = terminal.user_input(&update_deadline) {
            direction = match (key, direction) {
                (Input::Right, Direction::Right) => Direction::Down,
                (Input::Right, Direction::Down) => Direction::Left,
                (Input::Right, Direction::Left) => Direction::Up,
                (Input::Right, Direction::Up) => Direction::Right,
                (Input::Left, Direction::Right) => Direction::Up,
                (Input::Left, Direction::Up) => Direction::Left,
                (Input::Left, Direction::Left) => Direction::Down,
                (Input::Left, Direction::Down) => Direction::Right,
                (Input::Exit, _) => quit(&terminal),
                (Input::Pause, _) => {
                    pause = !pause;
                    continue;
                }
            };
        }

        if pause {
            continue;
        }

        update_deadline = Instant::now() + update_interval;

        // step: redraw snake and food if eaten
        let (eaten, crashed) = advance_snake(&terminal, &mut board, width, &mut snake, &direction, &mut target_length, reward);
        if eaten {
            food = match random_free_spot(&board, width) {
                Some(food) => food,
                None => quit(&terminal)
            };
            draw_food(&terminal, &mut board, width, &food);
        }
        if crashed {
            quit(&terminal);
        }
    }

}
