use std::env;
use std::process;
use std::path::Path;

pub mod lee_maze;
pub mod config;
use crate::lee_maze::lee_maze::Maze;
use crate::config::config::Config;


static USAGE_MSG: &'static str = r#"
Usage: cargo run -- <input_file>

Description:
  This program implements the Lee algorithm for maze routing.
  It takes an input file representing the maze layout, including the grid's height and width, 
  net connections, and costs for non-preferred directions or vias.

Arguments:
  <input_file>    The input file containing the maze layout. Only one file is allowed.

Example:
  cargo run -- maze.txt
"#;


fn usage(args: &[String]) -> Result<&str, &'static str> {
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
pub enum Layer {
    Layer1,
    Layer2,
}

#[derive(Debug)]
pub struct Pin {
    x: i16,
    y: i16,
    layer: Layer,
}

#[derive(Debug)]
pub struct Net {
    _net_name: String,
    pins: Vec<Pin>,
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = match usage(&args) {
        Ok(fname) => fname,
        Err(errmsg) => {
            eprint!("{}", errmsg);
            process::exit(1);
        }
    };

    let config = Config::build(filename).unwrap_or_else(|err_msg| {
        eprint!("{}", err_msg);
        process::exit(1);
    });

    // Initialize maze based on the config
    let mut maze = Maze::new(config.grid_width as usize, config.grid_height as usize, 2, config.via_cost as u32, config.nonpreferred_direction_cost as u32);

    // Add obstacles to the maze
    maze.initialize_obstacles(&config.obstacles);

    // Process the nets
    maze.process_nets(&config.nets);

}