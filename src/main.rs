#[macro_use]
extern crate error_chain;

use rand::{thread_rng, prelude::*};
use arr_macro::arr;

use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, terminal};
use std::time::Duration;
use std::thread;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }
    }
}
use errors::*;

// Also change get_cols() and get_cols_mut() when changing board dimensions
const BOARD_HEIGHT: usize = 10;
const BOARD_WIDTH: usize = 10;

const POPULATION_SIZE: usize = 5;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

/* Board :
 * 80% de NULL: 0
 * 10% de FOOD: 1
 * 10% de POISON: 2
*/

type BasicBoard = [[u8;BOARD_HEIGHT];BOARD_WIDTH];
type BasicBoardRefRotated<'a> = [[&'a u8;BOARD_WIDTH];BOARD_HEIGHT];

struct Board {
    rows:BasicBoard
}

impl Board {
    pub fn get_rows(&self) -> &BasicBoard {
        &self.rows
    }
    pub fn get_rows_mut(&mut self) -> &mut BasicBoard {
        &mut self.rows
    }
    pub fn get_cols(&self) -> BasicBoardRefRotated {
        let mut i = 0;
        let cols:BasicBoardRefRotated = arr![{
            i += 1;
            let mut j = 0;
            arr![{
                j += 1;
                &self.rows[j-1][i-1]
            }; 10] // BOARD_HEIGHT
        }; 10]; // BOARD_WIDTH
        cols
    }
    pub fn get(&self, row:usize, col:usize) -> Result<&u8> {
        if row >= BOARD_HEIGHT {
            Err("row out of range".into())
        } else if col >= BOARD_WIDTH {
            Err("col out of range".into())
        } else {
            Ok(&self.rows[row][col])
        }
    }
    pub fn set(&mut self, row:usize, col:usize, val:u8) -> Result<()> {
        if row >= BOARD_HEIGHT {
            Err("row out of range".into())
        } else if col >= BOARD_WIDTH {
            Err("col out of range".into())
        } else {
            self.rows[row][col] = val;
            Ok(())
        }
    }
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self { rows: arr![{
            arr![{
                let rand_num: f32 = rng.gen();
                if rand_num > 0.9 {
                    1
                } else if rand_num < 0.1 {
                    2
                } else {
                    0
                }
            }; 10] // BOARD_WIDTH
        }; 10] } // BOARD_HEIGHT
    }
    pub fn display() -> Result<()> {
        Ok(())
    }
}

struct Output;

impl Output {

}

fn main() -> Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    thread::spawn(|| -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(event) = event::read()? {
                    match event {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: event::KeyModifiers::NONE,
                            ..
                        } => break,
                        _ => {}
                    }
                    println!("{:?}\r", event);
                };
            } else {
                println!("No input yet\r");
            }
        }
        Ok(())
    });
    let mut board = Board::new();
    println!("{:?}", board.get_rows());
    println!();
    println!("{:?}", board.get_cols());
    Ok(())
}
