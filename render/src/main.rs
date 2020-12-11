use common::{load_last_history, Board, BOARD_HEIGHT, BOARD_WIDTH};
use image::ImageBuffer;
use std::fs;

pub const IMG_SCALE: u32 = 10;
pub const IMG_WIDTH: u32 = BOARD_WIDTH * IMG_SCALE;
pub const IMG_HEIGHT: u32 = BOARD_HEIGHT * IMG_SCALE;

fn render_board(path: &str, board: &Board) {
    let img = ImageBuffer::from_fn(IMG_WIDTH, IMG_HEIGHT, |x, y| {
        let pixel = &board.pixels[(y / IMG_SCALE) as usize][(x / IMG_SCALE) as usize];
        image::Rgb([
            ((pixel.color >> 16) & 255) as u8,
            ((pixel.color >> 8) & 255) as u8,
            ((pixel.color) & 255) as u8,
        ])
    });
    println!("Rendering {}", path);
    img.save(path).unwrap();
}

fn main() {
    let history = load_last_history().expect("Can't load history");
    let _ = fs::create_dir("images");
    for (index, board) in history.boards.iter().enumerate() {
        render_board(format!("images/{:06}.png", index).as_str(), board);
    }
    // To join images into a video use the following command:
    //
    // ffmpeg -r 60 -i images/%06d.png -c:v libx264 -vf "fps=60,format=yuv420p" -crf 3 video_high.mp4
}
