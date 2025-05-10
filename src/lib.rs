pub mod config;
pub mod lee_maze;

pub use config::config::Config;
pub use lee_maze::lee_maze::Maze;

use std::path::Path;

static USAGE_MSG: &str = r#"
Usage: cargo run -- <input_file>

Description:
  This program implements the Lee algorithm for maze routing.
  It takes an input file representing the maze layout...

Arguments:
  <input_file>    The input file containing the maze layout.

Example:
  cargo run -- maze.txt
"#;

pub fn usage(args: &[String]) -> Result<&str, &'static str> {
    match args.get(1) {
        Some(filename) => {
            if !filename.ends_with(".txt") {
                return Err("Unsupported file");
            }
            if !Path::new(filename).exists() {
                return Err("File does not exist in the current directory");
            }
            Ok(filename)
        }
        None => Err(USAGE_MSG),
    }
}

#[derive(Debug)]
pub struct Pin {
    pub coord: Coord,
}

#[derive(Debug)]
pub struct Net {
    _net_name: String,
    pins: Vec<Pin>,
}

type Coord = (usize, usize, usize); // layer,x,y
