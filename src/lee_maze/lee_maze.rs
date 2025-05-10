use std::{collections::{BinaryHeap, HashMap, HashSet, VecDeque}, u32::MAX};
use crate::{Net,Layer,Pin};
use std::cmp::Reverse;

type Coord = (usize, usize, usize); // (layer, row, col)

#[derive(Clone,PartialEq,Debug)]
pub enum Cell {
    Free,
    Blocked,
    Routed,    
    Start,      
    Target(u32),
    Candidate(u32),
}

pub struct Maze {
    grid: Vec<Vec<Vec<Cell>>>, // [layer][row][col]
    start_cords: Vec<Coord>,
    via_cost: u32,
    width: usize,
    height: usize,
    nonpreferred_direction_cost: u32,
    vias: HashSet<Coord>
}

impl Maze {
    // Constructor to initialize the maze grid
    pub fn new(width: usize, height: usize, layers: usize, via_cost: u32, nonpreferred_direction_cost: u32) -> Self {
        Maze {
            grid: vec![vec![vec![Cell::Free; height]; width]; layers],
            start_cords: vec![],
            via_cost,
            width,
            height,
            nonpreferred_direction_cost,
            vias: HashSet::new()
        }
    }

    // Check if a coordinate is within bounds
    fn is_valid(&self, l: isize, r: isize, c: isize) -> bool {
        l >= 0 && r >= 0 && c >= 0 &&
        (l as usize) < self.grid.len() &&
        (r as usize) < self.width &&
        (c as usize) < self.height &&
        // Skip Blocked, Routed, and Start cells
        !matches!(self.grid[l as usize][r as usize][c as usize], Cell::Blocked | Cell::Routed | Cell::Start)
    }

    fn neighbors(&self, l: usize, r: usize, c: usize) -> Vec<(Coord, u32)> {
        let deltas = vec![
            (0, -1, 0), // up (vertical)
            (0, 1, 0),  // down
            (0, 0, -1), // left (horizontal)
            (0, 0, 1),  // right
            (-1, 0, 0), // layer down (via)
            (1, 0, 0),  // layer up
        ];

        let mut result = Vec::new();
        for (dl, dr, dc) in deltas {
            let nl = l as isize + dl;
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            
            if self.is_valid(nl, nr, nc) {
                let nl = nl as usize;
                let nr = nr as usize;
                let nc = nc as usize;
                // Base cost: 1 for horizontal/vertical, via_cost for via
                let mut cost = if dl != 0 {  self.via_cost } else { 1 };

                // Directional penalties
                if l % 2 == 0 && dr != 0 { cost += self.nonpreferred_direction_cost; }  // Vertical cost on even layers
                if l % 2 != 0 && dc != 0 { cost += self.nonpreferred_direction_cost; }  // Horizontal cost on odd layers

                match self.grid[nl][nr][nc] {
                    Cell::Free | Cell::Candidate(_) | Cell::Target(_) => result.push(((nl, nr, nc), cost)),
                    _ => {}
                }
            }
        }
        result
    }

    fn bfs(&mut self) {
        let mut queue = VecDeque::new();
        let mut parent: HashMap<Coord, Coord> = HashMap::new();

        for &source in &self.start_cords {
            queue.push_back(source);
        }

        let mut any_updates = true;

        while any_updates {
            any_updates = false;
            let mut next_queue = VecDeque::new();

            while let Some((l, r, c)) = queue.pop_front() {
                let cost = match self.grid[l][r][c] {
                    Cell::Candidate(existing_cost) => existing_cost,
                    Cell::Target(existing_cost) => existing_cost,
                    _ => 0,
                };

                for ((nl, nr, nc), move_cost) in self.neighbors(l, r, c) {
                    let new_cost = cost + move_cost;

                    match self.grid[nl][nr][nc] {
                        Cell::Free => {
                            self.grid[nl][nr][nc] = Cell::Candidate(new_cost);
                            parent.insert((nl, nr, nc), (l, r, c));
                            next_queue.push_back((nl, nr, nc));
                            any_updates = true;
                        }
                        Cell::Candidate(existing_cost) if new_cost < existing_cost => {
                            self.grid[nl][nr][nc] = Cell::Candidate(new_cost);
                            parent.insert((nl, nr, nc), (l, r, c));
                            next_queue.push_back((nl, nr, nc));
                            any_updates = true;
                        }
                        Cell::Target(existing_cost) if new_cost < existing_cost => {
                            self.grid[nl][nr][nc] = Cell::Target(new_cost);
                            parent.insert((nl, nr, nc), (l, r, c));
                            next_queue.push_back((nl, nr, nc));
                            any_updates = true;
                        }
                        _ => {}
                    }
                }
            }
            queue = next_queue;
        }
    }


    // Dijkstra to find the path between start and target
    fn dijkstra(&mut self) {
        let mut queue = BinaryHeap::new(); // Min-heap via Reverse
        let mut parent: HashMap<Coord, Coord> = HashMap::new();
    
        for &source in &self.start_cords {
            queue.push((Reverse(0), source));
        }
        
        while let Some((Reverse(cost), (l, r, c))) = queue.pop() {
            if let Cell::Target(_) = self.grid[l][r][c] {
                self.reconstruct_path((l, r, c), &parent);
                return;
            }
            for ((nl, nr, nc), move_cost) in self.neighbors(l, r, c) {
                let new_cost = cost + move_cost;
            
                match self.grid[nl][nr][nc] {
                    Cell::Free => {
                        self.grid[nl][nr][nc] = Cell::Candidate(new_cost);
                        parent.insert((nl, nr, nc), (l, r, c));
                        queue.push((Reverse(new_cost), (nl, nr, nc)));
                    }
                    Cell::Candidate(existing_cost) => {
                        if new_cost < existing_cost {
                            self.grid[nl][nr][nc] = Cell::Candidate(new_cost);
                            parent.insert((nl, nr, nc), (l, r, c));
                            queue.push((Reverse(new_cost), (nl, nr, nc)));
                        }
                    }
                    Cell::Target(existing_cost) => {
                        if new_cost < existing_cost {
                            self.grid[nl][nr][nc] = Cell::Target(new_cost);
                            parent.insert((nl, nr, nc), (l, r, c));
                            queue.push((Reverse(new_cost), (nl, nr, nc)));
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn reconstruct_path(&mut self, end: Coord, parent: &HashMap<Coord, Coord>) {
        let mut current = end;

        while self.grid[current.0][current.1][current.2] != Cell::Start {
            let prev = *parent.get(&current).unwrap();

            // If changing layer, mark as Via
            if current.0 != prev.0 {
                self.vias.insert(current);
                self.vias.insert(prev);
            }

            self.grid[current.0][current.1][current.2] = Cell::Start;

            self.start_cords.push(current);
            current = prev;
        }
    }


    fn clear_candidates(&mut self) {
        for layer in &mut self.grid {
            for row in layer {
                for cell in row {
                    if let Cell::Candidate(_) = *cell {
                        *cell = Cell::Free; // Clear candidates
                    }
                }
            }
        }
    }

    pub fn process_nets(&mut self, nets: &Vec<Net>) {
        for net in nets {
            // insert the start pin for this net
            self.set_as_target(&net.pins);
            let start_pin: &Pin = &net.pins[0];   // &net.pins[0]; TODO: to be replaced by a function that gets the closest pin to a corner
            let start_layer = match start_pin.layer {
                Layer::Layer1 => 0,  // Use layer 0 for Layer1
                Layer::Layer2 => 1,  // Use layer 1 for Layer2
            };

            let start = (start_layer, start_pin.x as usize, start_pin.y as usize);
            self.grid[start_layer][start_pin.x as usize][start_pin.y as usize] = Cell::Start;
            //all_sources.push(start);
            self.start_cords.clear();
            self.start_cords.push(start);  // Add this source to start_cords
            
            // 3
            for _ in 0..net.pins.len() - 1 {
                // Perform Dijkstra to route from current sources
                self.dijkstra();
                self.clear_candidates();  // Reset candidate cells
                self.print_layers_side_by_side();

            }
            self.finalize_routing();
            println!("\nFinal Layout");
            self.print_layers_side_by_side();
        }
    }


    fn finalize_routing(&mut self) {
        for source in &self.start_cords {
            self.grid[source.0][source.1][source.2] = Cell::Routed; // Mark final sources as routed
        }
    }

    fn set_as_target(&mut self, pins: &Vec<Pin>) {
        for pin in pins {
            let layer = match pin.layer {
                Layer::Layer1 => 0,  // Use layer 0 for Layer1
                Layer::Layer2 => 1,  // Use layer 1 for Layer2
            };
            self.grid[layer][pin.x as usize][pin.y as usize] = Cell::Target(MAX);
        }
    }

    pub fn initialize_obstacles(&mut self, obstacles: &Vec<(i16, i16)>) {
        for (x, y) in obstacles {
            if *x >= 0 && *x < self.width as i16 && *y >= 0 && *y < self.height as i16 {
                self.grid[0][*x as usize][*y as usize] = Cell::Blocked; // Mark as Blocked in Layer 1
            }
        }
    }

    pub fn print_layers_side_by_side(&self) {
        println!("Maze Layers 1 & 2 (Side by Side)");
        let max_rows = self.grid[0].len().max(self.grid[1].len());
        
        for r in 0..max_rows {
            // Layer 0
            if r < self.grid[0].len() {
                for c in 0..self.grid[0][r].len() {
                    let coord = (0, r, c);
                    let symbol = if self.vias.contains(&coord) {
                        " V ".to_string()
                    } else {
                        match self.grid[0][r][c] {
                            Cell::Free => " . ".to_string(),
                            Cell::Blocked => " # ".to_string(),
                            Cell::Routed => " * ".to_string(),
                            Cell::Start => " S ".to_string(),
                            Cell::Target(_) => " T ".to_string(),
                            Cell::Candidate(cost) => format!("{:>2} ", cost),
                        }
                    };
                    print!("{}", symbol);
                }
            } else {
                print!("{}", "   ".repeat(self.grid[0][0].len()));
            }

            // Layer separation with vertical bar
            print!("  │  ");

            // Layer 1
            if r < self.grid[1].len() {
                for c in 0..self.grid[1][r].len() {
                    let coord = (1, r, c);
                    let symbol = if self.vias.contains(&coord) {
                        " V ".to_string()
                    } else {
                        match self.grid[1][r][c] {
                            Cell::Free => " . ".to_string(),
                            Cell::Blocked => " # ".to_string(),
                            Cell::Routed => " * ".to_string(),
                            Cell::Start => " S ".to_string(),
                            Cell::Target(_) => " T ".to_string(),
                            Cell::Candidate(cost) => format!("{:>2} ", cost),
                        }
                    };
                    print!("{}", symbol);
                }
            }

            println!("| Row {}", r);
        }
    }

}