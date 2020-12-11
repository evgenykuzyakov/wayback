use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::{fs, io};

pub const BOARD_WIDTH: u32 = 50;
pub const BOARD_HEIGHT: u32 = 50;
pub const TOTAL_NUM_PIXELS: u32 = BOARD_WIDTH * BOARD_HEIGHT;

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize,
    Copy,
    Eq,
    PartialEq,
    Clone,
    Default,
    Debug,
)]
pub struct Pixel {
    pub color: u32,
    pub owner_id: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Board {
    pub pixels: Vec<Vec<Pixel>>,
    pub block_height: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
pub struct History {
    pub boards: Vec<Board>,
    pub last_fetched_block: u64,
}

pub fn load_last_history() -> Option<History> {
    let _ = fs::create_dir("history");

    let mut entries = fs::read_dir("history")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();
    if entries.len() > 3 {
        let first_entry = entries.first().unwrap();
        println!("Deleting oldest history {}", first_entry.to_str().unwrap());
        fs::remove_file(first_entry).unwrap();
    }

    entries.last().map(|last_history_file| {
        println!(
            "Recovering history from {}",
            last_history_file.to_str().unwrap()
        );
        let history = History::try_from_slice(&fs::read(last_history_file).unwrap()).unwrap();
        println!("History contains {} boards", history.boards.len());
        history
    })
}
