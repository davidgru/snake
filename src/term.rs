extern crate terminal;

use std::io::Write;
use terminal::{Clear, Action, Value, Retrieved, Event, KeyCode, KeyEvent};

use std::time::Duration;
use std::time::Instant;

#[derive(PartialEq)]
pub enum Input {
    Left, Right, Exit
}


pub struct Terminal {
    terminal: terminal::Terminal<std::io::Stdout>,
}


impl Terminal {

    pub fn new() -> Terminal {
        Terminal{
            terminal: terminal::stdout()
        }
    }

    fn lock(&self) -> terminal::TerminalLock<std::io::Stdout> {
        self.terminal.lock_mut().unwrap()
    }

    // enter new screen and hide cursor
    pub fn setup(&self) {
        let mut lock = self.lock();
        lock.batch(Action::EnterAlternateScreen).unwrap();
        lock.batch(Action::EnableRawMode).unwrap();
        lock.batch(Action::HideCursor).unwrap();
        lock.flush_batch().unwrap();
    }

    // return terminal size in (width, height)
    pub fn get_size(&self) -> Option<(usize, usize)> {
        if let Ok(Retrieved::TerminalSize(w, h)) = self.lock().get(Value::TerminalSize) {
            Some((w as usize, h as usize))
        } else {
            None
        }
    }

    // listen for user input for an interval. return the last entered direction, or exit
    pub fn user_input(&self, interval_us: u64) -> Option<Input> {
        let now = Instant::now();
        let deadline = now + Duration::from_micros(interval_us);
        
        let lock = self.lock();

        let mut code: Option<Input> = None;
        loop {
            let now = Instant::now();
            if let Ok(Retrieved::Event(Some(Event::Key(key)))) = lock.get(Value::Event(Some(deadline - now))) {
                code = match key {
                    KeyEvent{code: KeyCode::Left, ..} => {
                        Some(Input::Left)
                    },
                    KeyEvent{code: KeyCode::Right, ..} => {
                        Some(Input::Right)
                    },
                    KeyEvent{code: KeyCode::Char('q'), ..} => {
                        Some(Input::Exit)
                    },
                    _ => code
                };
                if code == Some(Input::Exit) {
                    break;
                }
            } else {
                break;
            }
        }
        code
    }

    // write board to screen
    pub fn display(&self, board: &[u8]) {
        let mut lock = self.lock();
        lock.act(Action::ClearTerminal(Clear::All)).unwrap();
        lock.act(Action::MoveCursorTo(0, 0)).unwrap();
        lock.write(board).unwrap();
        lock.flush_batch().unwrap()
    }

    // rewrite a single cell
    pub fn write_cell(&self, symbol: u8, x: usize, h: usize) {
        let mut lock = self.lock();
        lock.batch(Action::MoveCursorTo(x as u16, h as u16)).unwrap();
        lock.write(&[symbol]).unwrap();
        lock.flush_batch().unwrap();
    }

    // show cursor again and return to old screen
    pub fn clean(&self) {
        let mut lock = self.lock();
        lock.batch(Action::ShowCursor).unwrap();
        lock.batch(Action::DisableRawMode).unwrap();
        lock.batch(Action::LeaveAlternateScreen).unwrap();
        lock.flush_batch().unwrap()
    }

}
