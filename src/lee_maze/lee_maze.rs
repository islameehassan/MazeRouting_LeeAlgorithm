use std::collections::{VecDeque, HashMap};

type Coord = (usize, usize, usize); // (layer, row, col)

pub enum Cell {
    Free,
    Blocked,
    Routed,
    //Via,     
    Start,      
    Target,     
    Candidate(u32),
}

struct Maze {
    grid: Vec<Vec<Vec<Cell>>>, // [layer][row][col]
    start_cords: Vec<Coord>,
    via_cost: u32,
    width: usize,
    height: usize
}

impl Maze {
    // Constructor to initialize the maze grid
    fn new(width: usize, height: usize, layers: usize, via_cost: u32) -> Self {
        Maze {
            grid: vec![vec![vec![Cell::Free; height]; width]; layers],
            via_cost,
            width,
            height,
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
            // (-1, 0, 0), // layer down (via)
            // (1, 0, 0),  // layer up
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
                let mut cost = if dl != 0 { self.via_cost } else { 1 };

                // Directional penalties
                if l % 2 == 0 && dr != 0 { cost += 5; }  // Vertical cost on even layers
                if l % 2 != 0 && dc != 0 { cost += 5; }  // Horizontal cost on odd layers

                match self.grid[nl][nr][nc] {
                    Cell::Free | Cell::Candidate(_) => result.push(((nl, nr, nc), cost)),
                    _ => {}
                }
            }
        }
        result
    }

    // BFS to find the path between start and target
    fn bfs(&mut self, sources: Vec<Coord>) -> Option<Vec<Coord>> {
        let mut queue = VecDeque::new();
        let mut parent: HashMap<Coord, Coord> = HashMap::new();

        // Mark sources as Start
        for source in &sources {
            self.grid[source.0][source.1][source.2] = Cell::Start; // Start cell
            queue.push_back(*source);
        }

        while let Some((l, r, c)) = queue.pop_front() {
            // Check if we've reached a target cell
            if let Cell::Target = self.grid[l][r][c] {
                return Some(self.reconstruct_path(sources[0], (l, r, c), &parent));
            }

            for ((nl, nr, nc), move_cost) in self.neighbors(l, r, c) {
                let next = (nl, nr, nc);

                match self.grid[nl][nr][nc] {
                    Cell::Free => {
                        // Update the cost in the grid directly if the cell is free
                        self.grid[nl][nr][nc] = Cell::Candidate(move_cost); // Mark as candidate (potential path)
                        parent.insert(next, (l, r, c)); // Set parent for path reconstruction
                        queue.push_back(next);
                    }
                    Cell::Candidate(existing_cost) => {
                        // If it's already a candidate, check the cost and update if necessary
                        if move_cost < existing_cost {
                            self.grid[nl][nr][nc] = Cell::Candidate(move_cost); // Update to new lower cost
                            parent.insert(next, (l, r, c)); // Set parent for path reconstruction
                            queue.push_back(next);
                        }
                    }
                    _ => {} // If the cell is blocked or routed, we skip it
                }
            }
        }

        None // No path found
    }

    fn reconstruct_path(&self, start: Coord, end: Coord, parent: &HashMap<Coord, Coord>) -> Vec<Coord> {
        let mut path = Vec::new();
        let mut current = end;
    
        while current != start {
            path.push(current);
            // Add this cell to start_cords (except for the start)
            if current != start {
                self.start_cords.push(current);  // Add to start_cords
            }
            current = *parent.get(&current).unwrap();
        }
        path.push(start);
        path.reverse(); // Reverse to get the path from start to end
    
        path
    }

    fn mark_path_as_sources(&mut self, path: &Vec<Coord>) {
        for (l, r, c) in path {
            self.grid[*l][*r][*c] = Cell::Start; // Mark path as sources
        }
    }

    fn clear_candidates(&mut self) {
        for layer in &mut self.grid {
            for row in &mut layer {
                for cell in row {
                    if let Cell::Candidate(_) = *cell {
                        *cell = Cell::Free; // Clear candidates
                    }
                }
            }
        }
    }

    fn process_nets(&mut self, nets: &Vec<Net>) {
        let mut all_sources = Vec::new();

        // Start with the first set of sources from the nets
        for net in nets {
            let start_pin = &net.pins[0];
            let start_layer = match start_pin.layer {
                Layer::Layer1 => 0,  // Use layer 0 for Layer1
                Layer::Layer2 => 1,  // Use layer 1 for Layer2
            };

            let start = (start_layer, start_pin.x as usize, start_pin.y as usize);
            all_sources.push(start);
            self.start_cords.push(start);  // Add this source to start_cords
        }

        // Process the nets in rounds
        while !self.start_cords.is_empty() {
            let current_sources = self.start_cords.clone();  // Save the current sources

            for net in nets {
                let start_pin = &net.pins[0];
                let target_pin = &net.pins[1];

                let target_layer = match target_pin.layer {
                    Layer::Layer1 => 0,
                    Layer::Layer2 => 1,
                };

                let target = (target_layer, target_pin.x as usize, target_pin.y as usize);

                // Set the target in the grid
                self.grid[target.0][target.1][target.2] = Cell::Target;

                // Perform BFS to route from current sources
                if let Some(path) = self.bfs(current_sources.clone()) {
                    self.mark_path_as_sources(&path);  // Mark all path cells as sources
                    self.clear_candidates();  // Reset candidate cells
                    
                    // Add new sources (target) to the list for the next round
                    self.start_cords.push(target);
                }
            }
        }
    }


    fn finalize_routing(&mut self, all_sources: Vec<Coord>) {
        for source in all_sources {
            self.grid[source.0][source.1][source.2] = Cell::Routed; // Mark final sources as routed
        }
    }

    fn initialize_obstacles(&mut self, obstacles: &Vec<(i16, i16)>) {
        for (x, y) in obstacles {
            if *x >= 0 && *x < self.width as i16 && *y >= 0 && *y < self.height as i16 {
                self.grid[*x as usize][*y as usize][0] = Cell::Blocked; // Mark as Blocked in Layer 0
            }
        }
    }
}

fn main() {
    // Example input configuration
    let config = Config {
        grid_width: 5,
        grid_height: 5,
        obstacles: vec![(2, 2), (3, 3)],  // Example obstacles
        nets: vec![
            Net { net_name: String::from("Net1"), pins: vec![Pin { x: 0, y: 0, layer: Layer::Layer1 }, Pin { x: 4, y: 4, layer: Layer::Layer2 }] },
            Net { net_name: String::from("Net2"), pins: vec![Pin { x: 0, y: 1, layer: Layer::Layer1 }, Pin { x: 4, y: 1, layer: Layer::Layer2 }] },
        ],
        via_cost: 10,
        nonpreferred_direction_cost: 5,
    };

    // Initialize maze based on the config
    let mut maze = Maze::new(config.grid_width as usize, config.grid_height as usize, 2, config.via_cost as u32);

    // Add obstacles to the maze
    maze.initialize_obstacles(&config.obstacles);

    // Process the nets
    maze.process_nets(&config.nets);
}
