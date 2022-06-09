extern crate terminal;
extern crate rand;

use terminal::{error, Clear, Action, Value, Retrieved, Event, Event::Key, KeyCode, KeyEvent};
use std::io::Write;
use std::time::Duration;
use rand::Rng;
use std::collections::LinkedList;
use std::time::Instant;
use std::sync::{Arc, Mutex};


const EMPTY: u8 = ' ' as u8;
const BORDER: u8 = '#' as u8;
const FOOD: u8 = 'F' as u8;
const HEAD: u8 = '@' as u8;
const BODY: u8 = 'B' as u8;

#[derive(PartialEq)]
enum UserInput {
    Left, Right, Exit
}

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
    width + 4
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

fn advance_snake(board: &mut Vec<u8>, width: usize, snake: &mut LinkedList<(usize, usize)>, direction: &(i32, i32)) -> (bool, bool) {
    if let Some((h, w)) = snake.front() {
        let new_head_h = (*h as i32 + direction.0) as usize;
        let new_head_w = (*w as i32 + direction.1) as usize;

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

fn term_user_input<T: std::io::Write>(lock: &terminal::TerminalLock<T>) -> Option<UserInput> {
    let now = std::time::Instant::now();
    let deadline = now + Duration::from_secs(1);
    
    let mut code: Option<UserInput> = None;
    loop {
        let now = std::time::Instant::now();
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

fn term_setup<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>) -> error::Result<()> {
    lock.batch(Action::EnterAlternateScreen)?;
    lock.batch(Action::EnableRawMode)?;
    lock.batch(Action::HideCursor)?;
    lock.flush_batch()
}

fn term_clean<T: std::io::Write>(lock: &mut terminal::TerminalLock<T>) -> error::Result<()> {
    lock.batch(Action::ShowCursor)?;
    lock.batch(Action::DisableRawMode)?;
    lock.batch(Action::LeaveAlternateScreen)?;
    lock.flush_batch()
}


fn main() {
    if std::env::args().count() != 3 {
        print_usage();
    }

    let width: usize = parse_arg(1);
    let height: usize = parse_arg(2);

    let board_size = board_height(height) * board_width(width);

    let terminal = terminal::stdout();
    let mut lock = terminal.lock_mut().unwrap();

    term_setup(&mut lock).unwrap();

    let mut board : Vec<u8> = std::vec::Vec::with_capacity(board_size);

    let mut food: (usize, usize);
    let mut snake: LinkedList<(usize, usize)> = LinkedList::new();
    let mut direction: (i32, i32) = (0, 1);

    draw_border(&mut board, width, height);
    snake.push_back((height / 2 + 1, width / 2 + 1));
    draw_snake(&mut board, width, &snake);
    food = random_free_spot(&board, width);
    draw_food(&mut board, width, &food);

    loop {
        // user input
        if let Some(user_input) = term_user_input(&lock) {
            direction = match (user_input, direction) {
                (UserInput::Right, (0, 1)) => (1, 0),
                (UserInput::Right, (1, 0)) => (0, -1),
                (UserInput::Right, (0, -1)) => (-1, 0),
                (UserInput::Right, (-1, 0)) => (0, 1),
                (UserInput::Left, (0, 1)) => (-1, 0),
                (UserInput::Left, (-1, 0)) => (0, -1),
                (UserInput::Left, (0, -1)) => (1, 0),
                (UserInput::Left, (1, 0)) => (0, 1),
                (UserInput::Left, _) => (0, 0),
                (_, _) => (0, 0)
            };
        }
        if direction == (0, 0) {
            break;
        }

        let (eaten, crashed) = advance_snake(&mut board, width, &mut snake, &direction);

        draw_border(&mut board, width, height);
        draw_snake(&mut board, width, &snake);
        if eaten {
            food = random_free_spot(&board, width);
        }
        draw_food(&mut board, width, &food);

        // display
        if lock.act(Action::ClearTerminal(Clear::All)).is_err() ||
            lock.act(Action::MoveCursorTo(0, 0)).is_err() ||
            lock.write(board.as_slice()).is_err() {
            std::process::exit(1);
        }

        if crashed {
            break;
        }
    }

    term_clean(&mut lock).unwrap();
}
