use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResponse<T> {
    pub id: String,
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: i32,
    pub data: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewStateResponse {
    block_hash: String,
    block_height: u64,
    proof: Vec<String>,
    values: Vec<StateKeyValues>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateKeyValues {
    key: String,
    proof: Vec<String>,
    value: String,
}

pub const BOARD_WIDTH: u32 = 50;
pub const BOARD_HEIGHT: u32 = 50;
pub const TOTAL_NUM_PIXELS: u32 = BOARD_WIDTH * BOARD_HEIGHT;
pub const BERRY_GENESIS_BLOCK: u64 = 21793900;

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
    pixels: Vec<Vec<Pixel>>,
    block_height: u64,
}

fn fetch_board(block_height: u64) -> io::Result<Board> {
    let client = reqwest::blocking::Client::new();
    let body = json!({
        "id": "123",
        "jsonrpc": "2.0",
        "method": "query",
        "params": {
            "account_id": "berryclub.ek.near",
            "block_id": block_height,
            "prefix_base64": "cA==",
            "request_type": "view_state"
        }
    });
    let resp = client
        .post("https://rpc.mainnet.internal.near.org/")
        .json(&body)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .json::<JsonResponse<ViewStateResponse>>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(error) = resp.error {
        return Err(io::Error::new(io::ErrorKind::NotFound, error.data));
    }

    let res = resp.result.expect("No error, so got to be data");

    let mut board = Board {
        pixels: vec![Default::default(); BOARD_HEIGHT as usize],
        block_height: res.block_height,
    };
    for kv in res.values {
        let key =
            base64::decode(&kv.key).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row = u64::try_from_slice(&key[1..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if row > u64::from(BOARD_HEIGHT) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Row out of range",
            ));
        }
        let value =
            base64::decode(&kv.value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        board.pixels[row as usize] = BorshDeserialize::try_from_slice(&value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }

    Ok(board)
}

fn fetch_board_with_retries(block_height: u64) -> Option<Board> {
    for iter in 0..5 {
        match fetch_board(block_height) {
            Ok(board) => return Some(board),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return None,
                _ => {
                    println!("Error: {}", err);
                    std::thread::sleep(std::time::Duration::from_secs(1 << iter))
                }
            },
        }
    }
    None
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
pub struct History {
    pub boards: Vec<Board>,
    pub last_fetched_block: u64,
}

pub const FAST_JUMP_SIZE: u64 = 60;

fn main() {
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

    let mut history = if let Some(last_history_file) = entries.last() {
        println!(
            "Recovering history from {}",
            last_history_file.to_str().unwrap()
        );
        History::try_from_slice(&fs::read(last_history_file).unwrap()).unwrap()
    } else {
        History {
            boards: vec![fetch_board_with_retries(BERRY_GENESIS_BLOCK).unwrap()],
            last_fetched_block: BERRY_GENESIS_BLOCK,
        }
    };
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut block_height = history.last_fetched_block + 1;
    let final_block_height = 24852466;
    while running.load(Ordering::SeqCst) && block_height <= final_block_height {
        println!(
            "#{} Fast search. History has {} boards",
            block_height,
            history.boards.len()
        );
        let fast_jump_block_height = block_height + FAST_JUMP_SIZE;
        let mut fast_jump_board = None;
        if fast_jump_block_height <= final_block_height {
            if let Some(board) = fetch_board_with_retries(fast_jump_block_height) {
                if &history.boards.last().unwrap().pixels == &board.pixels {
                    block_height = fast_jump_block_height + 1;
                    history.last_fetched_block = fast_jump_block_height;
                    continue;
                } else {
                    fast_jump_board = Some(board);
                }
            }
        }
        println!(
            "Fetching blocks from {} to {}",
            block_height, fast_jump_block_height
        );
        while running.load(Ordering::SeqCst) && block_height <= fast_jump_block_height {
            println!(
                "#{} Slow search. History has {} boards",
                block_height,
                history.boards.len()
            );
            if let Some(board) = fetch_board_with_retries(block_height) {
                if &history.boards.last().unwrap().pixels != &board.pixels {
                    history.boards.push(board);
                }
                if let Some(fast_jump_board) = &fast_jump_board {
                    if &history.boards.last().unwrap().pixels == &fast_jump_board.pixels {
                        block_height = fast_jump_block_height + 1;
                        history.last_fetched_block = fast_jump_block_height;
                        break;
                    }
                }
            }
            history.last_fetched_block = block_height;
            block_height += 1;
        }
    }
    println!("Got it! Exiting...");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let path = format!("history/{}.borsh", now);
    println!("Saving history to {}", path);
    let mut file = File::create(path).unwrap();
    file.write_all(&history.try_to_vec().unwrap()).unwrap();
}
