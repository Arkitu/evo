use rand::{thread_rng, prelude::*};
use arr_macro::arr;

// Also change get_cols() and get_cols_mut() when changing board dimensions
const BOARD_HEIGHT: usize = 10;
const BOARD_WIDTH: usize = 10;

const POPULATION_SIZE: usize = 5;

/* Board :
 * 80% de NULL: 0
 * 10% de FOOD: 1
 * 10% de POISON: 2
*/

type BasicBoard = [[i32;BOARD_HEIGHT];BOARD_WIDTH];
type BasicBoardRotated = [[i32;BOARD_WIDTH];BOARD_HEIGHT];
type BasicBoardRef<'a> = [[&'a i32;BOARD_HEIGHT];BOARD_WIDTH];
type BasicBoardRefMut<'a> = [[&'a mut i32;BOARD_HEIGHT];BOARD_WIDTH];
type BasicBoardRefRotated<'a> = [[&'a i32;BOARD_WIDTH];BOARD_HEIGHT];
type BasicBoardRefMutRotated<'a> = [[&'a mut i32;BOARD_WIDTH];BOARD_HEIGHT];

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
    pub fn get_cols_mut<'a>(&'a mut self) -> BasicBoardRefMutRotated { 
        let mut i = 0;
        let cols:BasicBoardRefMutRotated = arr![{
            i += 1;
            let mut j = 0;
            arr![{
                j += 1;
                &mut self.rows[j-1][i-1]
            }; 10] // BOARD_HEIGHT
        }; 10]; // BOARD_WIDTH
        cols
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
}

fn main() {
    let mut board = Board::new();
    println!("{:?}", board.get_rows());
    println!();
    println!("{:?}", board.get_cols());
}
