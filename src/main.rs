#[macro_use]
extern crate error_chain;
use crossterm::terminal::ClearType;
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
    use std::collections::HashMap;
    use crossterm::style::Stylize;

    pub type Position = (isize, isize);

    pub enum CellBase {
        Grass,
        Water,
        Sand,
        Stone
    }
    
    pub enum CellContent {
        Food,
        Poison,
        None
    }
    
    pub struct Cell {
        base:CellBase,
        content:CellContent,
        height:isize
    }
    impl Default for Cell {
        fn default() -> Self {
            Self {
                base: CellBase::Grass,
                content: CellContent::None,
                height: 0
            }
        }
    }
    impl Cell {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn from_base(base:CellBase) -> Self {
            Self{
                base,
                content: CellContent::None,
                height: 0
            }
        }
        pub fn to_colored_string(&self, contains_player:bool) -> String {
            let mut s = match self.content {
                CellContent::None => " ".to_string(),
                CellContent::Food => "F".yellow().to_string(),
                CellContent::Poison => "P".red().to_string()
            };
            if contains_player {
                s = "*".white().to_string();
            }
            match self.base {
                CellBase::Grass => s = s.on_dark_green().to_string(),
                CellBase::Water => s = s.on_blue().to_string(),
                CellBase::Sand => s = s.on_yellow().to_string(),
                CellBase::Stone => s = s.on_grey().to_string()
            }
            s
        }
    }

    pub struct Board {
        cells:HashMap<Position, Cell>,
        pub player_pos:Position
    }
    impl Board {
        pub fn new() -> Self {
            Self { cells: HashMap::new(), player_pos:(0, 0) }
        }
        pub fn get_cell(&mut self, location:&Position) -> &Cell {
            self.cells.entry(*location).or_insert_with(||{
                // Generate cells here
                let random_nbr: f32 = rand::random();
                if random_nbr < 0.5 {
                    Cell::from_base(CellBase::Grass)
                } else if random_nbr < 0.9 {
                    Cell::from_base(CellBase::Stone)
                } else {
                    Cell::from_base(CellBase::Sand)
                }
            })
        }
        pub fn get_display_at_location(&mut self, center_chunk_location:&Position) -> String {
            let mut s = String::new();
            for y in -15..15 {
                for x in -15..15 {
                    let cell_location = (center_chunk_location.0 + x, center_chunk_location.1 + y);
                    let cell_contains_player = self.player_pos == cell_location;
                    s += &self.get_cell(&cell_location).to_colored_string(cell_contains_player);
                }
                s += "\n\r";
            }
            s.pop();
            s.pop();
            s
        }
        pub fn get_display(&mut self) -> String {
            self.get_display_at_location(&self.player_pos.clone())
        }
    }
}

enum MsgToMain {
    StopProgram,
    InputThreadCrash,
    GameThreadCrash
}

fn input_thread(game_tx:Sender<GameInput>) -> Result<MsgToMain> {
    terminal::enable_raw_mode()?;
    loop {
        if let Event::Key(event) = event::read()? {
            match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => { return Ok(MsgToMain::StopProgram) },
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

fn game_thread(input_rx:Receiver<GameInput>) -> Result<MsgToMain> {
    queue!(stdout(), terminal::Clear(ClearType::All), cursor::Hide)?;
    stdout().flush()?;

    let mut board = board::Board::new();

    let mut current_display = String::new();

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
            cursor::MoveTo(0, 0)
        )?;
        stdout().flush()?;
        let display = board.get_display();
        if display != current_display {
            current_display = display;
            println!("{}", current_display);
            // write!(stdout(), "{}", current_display)?;
        }
        stdout().flush()?;
    }
}

fn exit_display() -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    execute!(stdout(), cursor::Show)?;
    Ok(())
}

fn main() -> Result<()> {
    let _clean_up = CleanUp;

    let (main_tx, main_rx) = mpsc::channel::<MsgToMain>();
    let input_main_tx = main_tx.clone();
    let game_main_tx = main_tx.clone();
    let (input_game_tx, game_input_rx) = mpsc::channel::<GameInput>();
    thread::spawn(move || {
        match input_thread(input_game_tx) {
            Ok(v) => {
                input_main_tx.send(v).unwrap();
            },
            Err(_e) => {
                input_main_tx.send(MsgToMain::InputThreadCrash).unwrap();
            }
        }
    });
    thread::spawn(move || {
        match game_thread(game_input_rx) {
            Ok(v) => {
                game_main_tx.send(v).unwrap();
            },
            Err(_e) => {
                game_main_tx.send(MsgToMain::GameThreadCrash).unwrap();
            }
        }
    });

    match main_rx.recv().unwrap() {
        MsgToMain::StopProgram => { Ok(()) },
        MsgToMain::InputThreadCrash => {
            exit_display()?;
            println!("Error in input thread");
            Ok(())
        },
        MsgToMain::GameThreadCrash => {
            exit_display()?;
            println!("Error in game thread");
            Ok(())
        }
    }
}
