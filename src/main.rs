use std::env;
use std::path::Path;
use std::process;

use mazerouting_lee::{usage, Config, Maze};
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
    let mut maze = Maze::new(
        config.grid_width as usize,
        config.grid_height as usize,
        2,
        config.via_cost as u32,
        config.nonpreferred_direction_cost as u32,
    );

    // Add obstacles to the maze
    maze.initialize_obstacles(&config.obstacles);

    // Process the nets
    maze.process_nets(&config.nets);

    // Export routed paths to .txt
    if let Err(e) = maze.export_paths_txt("routed_output.txt") {
        eprintln!("Failed to write routed_output.txt: {}", e);
    } else {
        println!("Routed paths written to routed_output.txt");
    }

    // Export routed paths to .csv
    if let Err(e) = maze.export_paths_csv("routed_output.csv") {
        eprintln!("Failed to write routed_output.csv: {}", e);
    } else {
        println!("CSV output written to routed_output.csv");
    }

}
