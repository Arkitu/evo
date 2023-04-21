#[macro_use]
extern crate error_chain;
use crossterm::terminal::ClearType;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::Stylize;
use crossterm::{event, terminal, execute, queue, cursor};
use error_chain::mock::ErrorKind;
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

mod board_mod {
    use std::collections::HashMap;
    use crossterm::style::Stylize;
    use super::errors::*;
    use num_integer::div_floor;

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
        Pointer,
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
        pub fn set_content(&mut self, content:CellContent) -> Result<()> {
            if let CellContent::None = self.content {
                self.content = content;
                Ok(())
            } else {
                bail!("Cell is not empty!")
            }
        }
        pub fn to_colored_string(&self, contains_player:bool) -> String {
            let mut s = match self.content {
                CellContent::None => " ".to_string(),
                CellContent::Food => "F".yellow().to_string(),
                CellContent::Poison => "P".red().to_string(),
                CellContent::Pointer => "#".red().to_string()
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
        /// get the cell or create it if it don't exist yet
        pub fn load_cell(&mut self, pos:Position) -> &Cell {
            self.cells.entry(pos).or_insert_with(||{
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
        pub fn try_get_cell(&self, pos:&Position) -> Option<&Cell> {
            self.cells.get(pos)
        }
        pub fn get_cell(&mut self, pos:Position) -> &Cell {
            self.load_cell(pos)
        }
        pub fn get_cell_mut(&mut self, pos:Position) -> &mut Cell {
            self.load_cell(pos);
            self.cells.get_mut(&pos).unwrap()
        }
        pub fn get_display_at_location(&mut self, size:(isize, isize), center_chunk_location:&Position) -> String {
            let padding: (isize, isize) = (div_floor(size.0,2), size.1.div_floor(2));
            let mut s = String::new();
            for y in -15..15 {
                for x in -15..15 {
                    let cell_location = (center_chunk_location.0 + x, center_chunk_location.1 + y);
                    let cell_contains_player = self.player_pos == cell_location;
                    s += &self.load_cell(cell_location).to_colored_string(cell_contains_player);
                }
                s += "\n\r";
            }
            s.pop();
            s.pop();
            s
        }
        pub fn get_display(&mut self, size:(isize, isize)) -> String {
            self.get_display_at_location(size, &self.player_pos.clone())
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
                    ..
                } => { return Ok(MsgToMain::StopProgram) },
                KeyEvent {
                    code: KeyCode::Up,
                    ..
                } => { game_tx.send(GameInput::MoveUp).unwrap(); },
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => { game_tx.send(GameInput::MoveDown).unwrap(); },
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => { game_tx.send(GameInput::MoveLeft).unwrap(); },
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => { game_tx.send(GameInput::MoveRight).unwrap(); },
                KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                } => { game_tx.send(GameInput::DropPointer).unwrap(); },
                _ => {}
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
    DropPointer,
}

fn game_thread(input_rx:Receiver<GameInput>) -> Result<MsgToMain> {
    queue!(stdout(), terminal::Clear(ClearType::All), cursor::Hide)?;
    stdout().flush()?;

    let mut board = board_mod::Board::new();

    let mut current_display = String::new();
    let mut msg_to_player = String::new();

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
                    GameInput::DropPointer => {
                        if let Err(m) = board.get_cell_mut(board.player_pos).set_content(board_mod::CellContent::Pointer) {
                             msg_to_player = "/!\\".red().to_string() + &m.to_string();
                        }
                    }
                }
            },
            Err(_e) => {}
        }
        queue!(stdout(),
            cursor::MoveTo(0, 0)
        )?;
        stdout().flush()?;
        let display = board.get_display(terminal::size());
        if display != current_display || !msg_to_player.is_empty() {
            current_display = display;
            println!("{}\n\r{}", current_display, msg_to_player);
            msg_to_player = String::new();
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
