use common::{load_last_history, Board, BOARD_HEIGHT, BOARD_WIDTH};
use image::ImageBuffer;
use std::fs;

pub const IMG_SCALE: u32 = 12;
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
    let _ = fs::create_dir("images_v2");
    for (index, board) in history.boards.iter().enumerate() {
        if index > 86877 {
            render_board(format!("images_v2/{:06}.png", index).as_str(), board);
        }
    }
    // To join images into a video use the following command:
    //
    // ffmpeg -r 60 -i images/%06d.png -c:v libx264 -vf "fps=60,format=yuv420p" -crf 3 video_high.mp4
    // ffmpeg -r 60 -start_number 86878 -i images_v2/%06d.png -c:v libx264 -vf "fps=60,format=yuv420p" -crf 3 video_high_v2.mp4
    //
    // Put video on top of background with scaling
    //
    // ffmpeg -loop 1 -i bg.png -i video_high.mp4 -filter_complex "overlay=679:251:shortest=1,fps=60" -c:v libx264 -crf 3  output.mp4 -y
    // ffmpeg -loop 1 -i bg_v2.png -i video_high_v2.mp4 -filter_complex "overlay=700:180:shortest=1,fps=60" -c:v libx264 -crf 3  output_v2.mp4 -y
    //
    // Make audio
    // # requires bash
    // ffmpeg -f concat -safe 0 -i <(for f in ./*.mp3; do echo "file '$PWD/$f'"; done) -c copy output.mp3
    //
    // Join audio
    //
    // ffmpeg -i output_v2.mp4 -i output_v2.mp3 -shortest -c:v copy -c:a aac -b:a 320k final_v2.mp4
}
