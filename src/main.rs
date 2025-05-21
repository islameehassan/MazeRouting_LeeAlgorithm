use std::{env, process};
use eframe::{self, App};
use mazerouting_lee::{usage, Config, Maze, Net};
use mazerouting_lee::lee_maze::gui::MazeApp;
use mazerouting_lee::Coord;

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
    println!(" Running: Unordered Routing");
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

    // === RUN 2: Sorted Order with Heuristic ===
    println!("\n============================================");
    println!(" Running: Sorted Routing (Heuristic)");
    println!("============================================");

    let mut sorted_nets = config.nets.clone();
    // Precompute all pins from all nets
	let all_pins: Vec<_> = config
	    .nets
	    .iter()
	    .flat_map(|net| net.pins.iter().map(|p| p.coord))
	    .collect();

	sorted_nets.sort_by(|a, b| {
	    let a_count = count_pins_in_bbox(a, &all_pins);
	    let b_count = count_pins_in_bbox(b, &all_pins);

	    if a_count != b_count {
		return a_count.cmp(&b_count); // ascending → fewer pins in bbox
	    }

	    // Tie-breaker: use Manhattan distance
	    let adist = manhattan_distance(a);
	    let bdist = manhattan_distance(b);
	    adist.cmp(&bdist)
	});


    println!("Routing Order (Sorted):");
	for net in &sorted_nets {
	    let name = &net._net_name;
	    let pin_count = count_pins_in_bbox(net, &all_pins);
	    println!("{} → pins in bbox: {}", name, pin_count);
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

    maze1.export_paths_to_file("output_unordered.txt").unwrap_or_else(|err| {
        eprintln!("Failed to export unordered routing paths: {}", err);
        process::exit(1);
    });

    maze2.export_paths_to_file("output_sorted.txt").unwrap_or_else(|err| {
        eprintln!("Failed to export sorted routing paths: {}", err);
        process::exit(1);
    });

    // === Launch GUI with maze2 (The One With The Heuristic) ===
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Maze Routing Visualizer",
        native_options,
        Box::new(move |_cc| Ok(Box::new(MazeApp::new(maze2)) as Box<dyn App>)),
    ).expect("Failed to start GUI");
}

fn get_bounds(net: &Net) -> (usize, usize, usize, usize) {
    let min_x = net.pins.iter().map(|p| p.coord.1).min().unwrap();
    let max_x = net.pins.iter().map(|p| p.coord.1).max().unwrap();
    let min_y = net.pins.iter().map(|p| p.coord.2).min().unwrap();
    let max_y = net.pins.iter().map(|p| p.coord.2).max().unwrap();
    (min_x, max_x, min_y, max_y)
}

fn count_pins_in_bbox(net: &Net, all_pins: &[Coord]) -> usize {
    let (min_x, max_x, min_y, max_y) = get_bounds(net);
    let layer = net.pins[0].coord.0;

    all_pins
        .iter()
        .filter(|&&(l, x, y)| {
            l == layer && x >= min_x && x <= max_x && y >= min_y && y <= max_y
        })
        .count()
}

fn manhattan_distance(net: &Net) -> usize {
    let first = net.pins.first().unwrap().coord;
    let last = net.pins.last().unwrap().coord;

    let dx = (first.1 as isize - last.1 as isize).abs();
    let dy = (first.2 as isize - last.2 as isize).abs();

    (dx + dy) as usize
}

fn estimated_net_cost(net: &Net, non_pref_cost: i32) -> u32 {
    let first = net.pins.first().unwrap().coord;
    let last = net.pins.last().unwrap().coord;

    let dx = (first.1 as i32 - last.1 as i32).abs();
    let dy = (first.2 as i32 - last.2 as i32).abs();

    let mut cost = dx + dy;

    // Non-preferred direction penalty
    if first.0 % 2 == 0 {
        cost += dy * non_pref_cost;
    } else {
        cost += dx * non_pref_cost;
    }

    cost.max(0) as u32
}
