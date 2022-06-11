extern crate clap;
extern crate rand;
extern crate terminal;

use std::collections::LinkedList;
use std::io::Write;
use std::time::{Duration, Instant};

use clap::Parser;
use rand::Rng;
use terminal::{error, Clear, Action, Value, Retrieved, Event, KeyCode, KeyEvent};

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

#[derive(PartialEq)]
enum UserInput {
    Left, Right, Exit
}

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

// with + space for border and \n\r
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

fn draw_food<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>, board: &mut Vec<u8>, width: usize, food: &(usize, usize)) -> error::Result<()> {
    lock.batch(Action::MoveCursorTo(food.1 as u16, food.0 as u16))?;
    lock.write(&[FOOD])?;
    board[food.0 * board_width(width) + food.1] = FOOD;
    lock.flush_batch()
}

// move snake in direction and update board. return ({crashed into wall or myself}, {eaten food})
fn advance_snake<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>, board: &mut Vec<u8>, width: usize, snake: &mut LinkedList<(usize, usize)>, direction: &Direction) -> (bool, bool) {
    if let Some(&(h, w)) = snake.front() {
        let new_head_h = (h as i32 + direction.velocity().0) as usize;
        let new_head_w = (w as i32 + direction.velocity().1) as usize;

        snake.push_front((new_head_h, new_head_w));

        let out = match board[new_head_h * board_width(width) + new_head_w] {
            FOOD => (true, false),
            BORDER => (false, true),
            BODY => (false, true),
            EMPTY => (false, false),
            _ => panic!("Impossible")
        };
        lock.batch(Action::MoveCursorTo(new_head_w as u16, new_head_h as u16)).unwrap();
        lock.write(&[HEAD]).unwrap();
        lock.batch(Action::MoveCursorTo(w as u16, h as u16)).unwrap();
        lock.write(&[BODY]).unwrap();
        board[new_head_h * board_width(width) + new_head_w] = HEAD;
        board[h * board_width(width) + w] = BODY;
        if !out.0 {
            if let Some((h, w)) = snake.pop_back() {
                lock.batch(Action::MoveCursorTo(w as u16, h as u16)).unwrap();
                lock.write(&[EMPTY]).unwrap();
                board[h * board_width(width) + w] = EMPTY;
            }
        }
        lock.flush_batch().unwrap();
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

// listen for user input for an interval. return the last entered direction, or exit
fn term_user_input<T: std::io::Write>(lock: &terminal::TerminalLock<T>, interval_us: u64) -> Option<UserInput> {
    let now = Instant::now();
    let deadline = now + Duration::from_micros(interval_us);
    
    let mut code: Option<UserInput> = None;
    loop {
        let now = Instant::now();
        if let Ok(Retrieved::Event(Some(Event::Key(key)))) = lock.get(Value::Event(Some(deadline - now))) {
            code = match key {
                KeyEvent{code: KeyCode::Left, ..} => {
                    Some(UserInput::Left)
                },
                KeyEvent{code: KeyCode::Right, ..} => {
                    Some(UserInput::Right)
                },
                KeyEvent{code: KeyCode::Char('q'), ..} => {
                    Some(UserInput::Exit)
                },
                _ => code
            };
            if code == Some(UserInput::Exit) {
                break;
            }
        } else {
            break;
        }
    }
    code
}

// enter new screen and hide cursor
fn term_setup<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>) -> error::Result<()> {
    lock.batch(Action::EnterAlternateScreen)?;
    lock.batch(Action::EnableRawMode)?;
    lock.batch(Action::HideCursor)?;
    lock.flush_batch()
}

fn term_get_size<T: std::io::Write>(lock: &terminal::TerminalLock<T>) -> Option<(usize, usize)> {
    if let Ok(Retrieved::TerminalSize(w, h)) = lock.get(Value::TerminalSize) {
        Some((w as usize, h as usize))
    } else {
        None
    }
}

fn term_display<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>, board: &[u8]) -> error::Result<()> {
    lock.act(Action::ClearTerminal(Clear::All))?;
    lock.act(Action::MoveCursorTo(0, 0))?;
    lock.write(board)?;
    lock.flush_batch()
}

// show cursor again and return to old screen
fn term_clean<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>) -> error::Result<()> {
    lock.batch(Action::ShowCursor)?;
    lock.batch(Action::DisableRawMode)?;
    lock.batch(Action::LeaveAlternateScreen)?;
    lock.flush_batch()
}

fn main() {
    let terminal = terminal::stdout();
    let mut lock = terminal.lock_mut().unwrap();

    let args = Args::parse();

    // default is terminal width
    let width = args.width.unwrap_or_else(|| -> usize {term_get_size(&lock).unwrap().0 - 2});
    // default is terminal height
    let height = args.height.unwrap_or_else(|| -> usize {term_get_size(&lock).unwrap().1 - 3});
    // default is chosen so it takes the snake 4 seconds across the board 
    let freq = args.freq.unwrap_or_else(|| -> u64 {(std::cmp::min(width, height) / 4) as u64});

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_height(height) * board_width(width));
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();
    let mut direction = Direction::Right;
    let mut food: (usize, usize);

    term_setup(&mut lock).unwrap();

    // only draw border once
    draw_border(&mut board, width, height);
    term_display(&mut lock, board.as_slice()).unwrap();
    
    // draw snake and food the first time
    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, width, &snake);
    food = random_free_spot(&board, width);
    draw_food(&mut lock, &mut board, width, &food).unwrap();

    loop {
        // input
        if let Some(user_input) = term_user_input(&lock, (1000 * 1000 / freq) as u64) {
            direction = match (user_input, direction) {
                (UserInput::Right, Direction::Right) => Direction::Down,
                (UserInput::Right, Direction::Down) => Direction::Left,
                (UserInput::Right, Direction::Left) => Direction::Up,
                (UserInput::Right, Direction::Up) => Direction::Right,
                (UserInput::Left, Direction::Right) => Direction::Up,
                (UserInput::Left, Direction::Up) => Direction::Left,
                (UserInput::Left, Direction::Left) => Direction::Down,
                (UserInput::Left, Direction::Down) => Direction::Right,
                (UserInput::Exit, _) => break,
            };
        }

        // step: redraw snake and food if eaten
        let (eaten, crashed) = advance_snake(&mut lock, &mut board, width, &mut snake, &direction);
        if eaten {
            food = random_free_spot(&board, width);
            draw_food(&mut lock, &mut board, width, &food).unwrap();
        }

        if crashed {
            break;
        }
    }

    term_clean(&mut lock).unwrap();
}
