#[macro_use]
extern crate error_chain;

use crossterm::terminal::ClearType;
use rand::{thread_rng, prelude::*};
use arr_macro::arr;

use crossterm::style::Stylize;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, terminal, execute, queue, cursor};
use std::io::{stdout, Write};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }
    }
}
use errors::*;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        exit_display().unwrap();
    }
}

mod board {
    use crossterm::style::Stylize;
    use grid::*;
    use super::errors::*;
    const CHUNK_SIZE:usize = 10;

    enum CellBase {
        Grass,
        Water,
        Sand,
        Stone
    }
    
    enum CellContent {
        Player,
        Food,
        Poison
    }
    
    struct Cell {
        base:CellBase,
        content:Option<CellContent>
    }
    impl Default for Cell {
        fn default() -> Self {
            Self {
                base: CellBase::Grass,
                content: None
            }
        }
    }
    impl Cell {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn to_colored_string(&self) {
            let mut s = match self.content {
                None => " ".to_string(),
                Some(CellContent::Player) => "*".black().bold().to_string(),
                Some(CellContent::Food) => "F".yellow().to_string(),
                Some(CellContent::Poison) => "P".red().to_string()
            };
            match self.base {
                CellBase::Grass => s = s.on_green().to_string(),
                CellBase::Water => s = s.on_blue().to_string(),
                CellBase::Sand => s = s.on_yellow().to_string(),
                CellBase::Stone => s = s.on_grey().to_string()
            }
            s
        }
    }
    struct Chunk {
        content:Grid<Cell>,
        location:(isize, isize)
    }
    impl Chunk {
        pub fn new(location:(isize, isize)) -> Self {
            Self {
                content: Grid::new(CHUNK_SIZE, CHUNK_SIZE),
                location
            }
        }
    }

    pub struct Board {
        chunks:Vec<Chunk>
    }
    impl Board {
        pub fn new() -> Self {
            Self { chunks: vec![] }
        }
        pub fn get_chunk(&mut self, location:(isize, isize)) -> &Chunk {
            self.chunks.iter().find(|c| c.location == location).unwrap_or_else(||{
                self.chunks.push(Chunk::new(location));
                self.get_chunk(location)
                // Possible bug: if a modification is done, it can create an infinite recursion
            })
        }
    }
}

// const BOARD_WIDTH:usize = 10;
// const BOARD_HEIGHT:usize = 10;

// type BasicBoard = [[Cell;10];10];
// type BasicBoardRefRotated<'a> = [[&'a Cell;10];10];

// struct Board {
//     rows:BasicBoard,
//     player_pos:(usize, usize)
// }

// impl Board {
//     pub fn get_rows(&self) -> &BasicBoard {
//         &self.rows
//     }
//     pub fn get_rows_mut(&mut self) -> &mut BasicBoard {
//         &mut self.rows
//     }
//     pub fn get_cols(&self) -> BasicBoardRefRotated {
//         let mut i = 0;
//         let cols:BasicBoardRefRotated = arr![{
//             i += 1;
//             let mut j = 0;
//             arr![{
//                 j += 1;
//                 &self.rows[j-1][i-1]
//             }; 10] // BOARD_HEIGHT
//         }; 10]; // BOARD_WIDTH
//         cols
//     }
//     pub fn get(&self, row:usize, col:usize) -> Result<&Cell> {
//         if row >= BOARD_HEIGHT {
//             Err("row out of range".into())
//         } else if col >= BOARD_WIDTH {
//             Err("col out of range".into())
//         } else {
//             Ok(&self.rows[row][col])
//         }
//     }
//     pub fn set(&mut self, row:usize, col:usize, val:Cell) -> Result<()> {
//         if row >= BOARD_HEIGHT {
//             Err("row out of range".into())
//         } else if col >= BOARD_WIDTH {
//             Err("col out of range".into())
//         } else {
//             self.rows[row][col] = val;
//             Ok(())
//         }
//     }
//     pub fn new() -> Self {
//         let mut rng = thread_rng();
//         Self { rows: arr![{
//                 arr![{
//                     let rand_num: f32 = rng.gen();
//                     if rand_num > 0.9 {
//                         Cell::Food
//                     } else if rand_num < 0.1 {
//                         Cell::Poison
//                     } else {
//                         Cell::Null
//                     }
//                 }; 10] // BOARD_WIDTH
//             }; 10], // BOARD_HEIGHT
//             player_pos: (4, 4)
//         }
//     }
//     pub fn cell_code_to_char(cell_code:&Cell) -> char {
//         match cell_code {
//             Cell::Null => '.',
//             Cell::Food => 'F',
//             Cell::Poison => 'P',
//             _ => '?'
//         }
//     }
//     pub fn cell_code_to_colored(cell_code:&Cell) -> String {
//         match cell_code {
//             Cell::Null => ".".to_string(),
//             Cell::Food => "F".on_green().to_string(),
//             Cell::Poison => "P".on_red().to_string(),
//             _ => "?".to_string()
//         }
//     }
//     pub fn display(&self, up:&i32) -> Result<()> {
//         let mut s = <&Board as Into<String>>::into(self);
//         write!(stdout(), "{}", s)?;
//         Ok(())
//     }
// }
// impl Into<String> for &Board {
//     fn into(self) -> String {
//         let mut char_board = self.get_rows()
//             .map(|row| {
//                 row.map(|x| {
//                     Board::cell_code_to_colored(&x)
//                 })
//             });
        
//         char_board[self.player_pos.1][self.player_pos.0] = '*'.yellow().to_string();

//         let mut s = char_board
//             .map(|row| {
//                 row.iter().fold(String::new(), |mut acc, x| {
//                     acc += x;
//                     acc
//                 })
//             }).iter().fold(String::new(), |mut acc, x| {
//                 acc.push_str(x);
//                 acc += "\n\r";
//                 acc
//             });
//         s.pop();
//         s.pop();
//         s += "\r";
//         s
//     }
// }

/*
 * Code :
 * 0: STOP_PROGRAM (send to main thread to stop all threads and exit)
 * 1: INPUT_THREAD_CRASH (send to main thread to display error and exit)
 * 2: GAME_THREAD_CRASH (send to main thread to display error and exit)
 */
fn input_thread(game_tx:Sender<GameInput>) -> Result<u16> {
    terminal::enable_raw_mode()?;
    loop {
        if let Event::Key(event) = event::read()? {
            match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { return Ok(0) },
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { game_tx.send(GameInput::MoveUp).unwrap(); },
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { game_tx.send(GameInput::MoveDown).unwrap(); },
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { game_tx.send(GameInput::MoveLeft).unwrap(); },
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { game_tx.send(GameInput::MoveRight).unwrap(); },
                _ => {
                    game_tx.send(GameInput::Pause).unwrap();
                }
            }
        };
    }
}

#[derive(Debug)]
enum GameInput {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Pause
}

fn game_thread(input_rx:Receiver<GameInput>) -> Result<u16> {
    execute!(stdout(), terminal::Clear(ClearType::All))?; // Clear screen

    let mut board = board::Board::new();

    let mut in_game_time = 0;

    let mut up = 0;

    loop {
        match input_rx.try_recv() {
            Ok(msg) => {
                match msg {
                    GameInput::MoveUp => {
                        board.player_pos.1 -= 1
                    },
                    GameInput::MoveDown => {board.player_pos.1 += 1},
                    GameInput::MoveLeft => {board.player_pos.0 -= 1},
                    GameInput::MoveRight => {board.player_pos.0 += 1},
                    GameInput::Pause => {
                        println!("Paused\r");
                        
                    }
                }
            },
            Err(_e) => {}
        }
        queue!(stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        board.display(&up)?;
        stdout().flush()?;
    }
}

fn exit_display() -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    Ok(())
}

fn main() -> Result<()> {
    let _clean_up = CleanUp;

    let (main_tx, main_rx) = mpsc::channel::<u16>();
    let input_main_tx = main_tx.clone();
    let game_main_tx = main_tx.clone();
    let (input_game_tx, game_input_rx) = mpsc::channel::<GameInput>();
    thread::spawn(move || {
        match input_thread(input_game_tx) {
            Ok(v) => {
                input_main_tx.send(v).unwrap();
            },
            Err(_e) => {
                input_main_tx.send(2).unwrap();
            }
        }
    });
    thread::spawn(move || {
        match game_thread(game_input_rx) {
            Ok(v) => {
                game_main_tx.send(v).unwrap();
            },
            Err(_e) => {
                game_main_tx.send(1).unwrap();
            }
        }
    });

    loop {
        match main_rx.recv().unwrap() {
            0 => { break },
            1 => {
                exit_display()?;
                println!("Error in input thread");
                break
            },
            2 => {
                exit_display()?;
                println!("Error in game thread");
                break
            },
            _ => {
                exit_display()?;
                println!("Unknown message received");
                break
            }
        }
    }
    Ok(())
}
