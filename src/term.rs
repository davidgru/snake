extern crate terminal;

use std::io::Write;
use terminal::{Clear, Action, Value, Retrieved, Event, KeyCode, KeyEvent};

use std::time::Instant;

#[derive(PartialEq, Clone, Copy)]
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

    // return entered keys until exit is entered or specified deadline is met
    pub fn user_input(&self, until: &Instant) -> Option<Input> {
        let lock = self.lock();

        let mut num_left = 0;
        let mut num_right = 0;

        loop {
            let now = Instant::now();
            if let Ok(Retrieved::Event(Some(Event::Key(key)))) = lock.get(Value::Event(Some(*until - now))) {
                match key {
                    KeyEvent{code: KeyCode::Left, ..} => {
                        num_left += 1;
                    },
                    KeyEvent{code: KeyCode::Right, ..} => {
                        num_right += 1;
                    },
                    KeyEvent{code: KeyCode::Char('q'), ..} => {
                        return Some(Input::Exit);
                    },
                    _ => continue
                };
            } else {
                break;
            }
        }
        return if num_left > num_right {Some(Input::Left)} else if num_left < num_right {Some(Input::Right)} else {None}
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
