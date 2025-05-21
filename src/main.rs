use std::{env, process};
use eframe::{self, App};
use mazerouting_lee::{usage, Config, Maze};
use mazerouting_lee::lee_maze::gui::MazeApp;

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

    // === RUN 1: Original Order ===
    println!("\n==============================");
    println!("🧪 Running: Unordered Routing");
    println!("==============================");
    println!("Routing Order (Original):");
    for net in &config.nets {
        let name = &net._net_name;
        let first = net.pins.first().unwrap().coord;
        let last = net.pins.last().unwrap().coord;
        let dist = (first.1 as isize - last.1 as isize).abs()
                 + (first.2 as isize - last.2 as isize).abs();
        println!("{} → distance: {}", name, dist);
    }

    let mut maze1 = Maze::new(
        config.grid_width as usize,
        config.grid_height as usize,
        2,
        config.via_cost as u32,
        config.nonpreferred_direction_cost as u32,
    );
    maze1.initialize_obstacles(&config.obstacles);
    maze1.process_nets(&config.nets);

    // === RUN 2: Longest Net First Order ===
    println!("\n============================================");
    println!("🏁 Running: Sorted Routing (Longest Net First)");
    println!("============================================");

    let mut sorted_nets = config.nets.clone();
    sorted_nets.sort_by_key(|net| {
        let first = net.pins.first().unwrap().coord;
        let last = net.pins.last().unwrap().coord;
        let dist = (first.1 as isize - last.1 as isize).abs()
                 + (first.2 as isize - last.2 as isize).abs();
        std::cmp::Reverse(dist)
    });

    println!("Routing Order (Sorted):");
    for net in &sorted_nets {
        let name = &net._net_name;
        let first = net.pins.first().unwrap().coord;
        let last = net.pins.last().unwrap().coord;
        let dist = (first.1 as isize - last.1 as isize).abs()
                 + (first.2 as isize - last.2 as isize).abs();
        println!("{} → distance: {}", name, dist);
    }

    let mut maze2 = Maze::new(
        config.grid_width as usize,
        config.grid_height as usize,
        2,
        config.via_cost as u32,
        config.nonpreferred_direction_cost as u32,
    );
    maze2.initialize_obstacles(&config.obstacles);
    maze2.process_nets(&sorted_nets);

    // === Launch GUI with maze2 ===
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Maze Routing Visualizer",
        native_options,
        Box::new(move |_cc| Ok(Box::new(MazeApp::new(maze2)) as Box<dyn App>)),
    ).expect("Failed to start GUI");

}
