use std::cmp::Reverse;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    u32::MAX,
};

use crate::{Coord, Net, Pin};

#[derive(Clone, PartialEq, Debug)]
pub enum Cell {
    Free,
    Blocked,
    Routed(u32),     // indicate which net
    Start(u32),      // indicate which net
    Target(u32,u32),    // cost
    Candidate(u32), // cost
}

pub struct Maze {
    pub grid: Vec<Vec<Vec<Cell>>>, // [layer][row][col]
    pub start_cords: Vec<Coord>,
    pub via_cost: u32,
    pub width: usize,
    pub height: usize,
    pub nonpreferred_direction_cost: u32,
    pub vias: HashSet<Coord>,
    pub original_sources: HashSet<Coord>,
    pub current_net_processed: u32,
    pub routed_paths: Vec<(String, Vec<Coord>)>,
}

impl Maze {
    // Constructor to initialize the maze grid
    pub fn new(
        width: usize,
        height: usize,
        layers: usize,
        via_cost: u32,
        nonpreferred_direction_cost: u32,
    ) -> Self {
        Maze {
            grid: vec![vec![vec![Cell::Free; height]; width]; layers],
            start_cords: vec![],
            via_cost,
            width,
            height,
            nonpreferred_direction_cost,
            vias: HashSet::new(),
            original_sources: HashSet::new(),
            current_net_processed: 1, // temporary
            routed_paths: Vec::new(),
        }
    }

    // Check if a coordinate is within bounds
    fn is_valid(&self, l: isize, r: isize, c: isize) -> bool {
        if l < 0 || r < 0 || c < 0 {
            return false;
        }
    
        let (l, r, c) = (l as usize, r as usize, c as usize);
    
        if l >= self.grid.len() || r >= self.width || c >= self.height {
            return false;
        }
        
        if l == 0 && r == 0 && c == 1{
            println!("{:?}",self.grid[l][r][c]);
        }
        match self.grid[l][r][c] {
            Cell::Blocked | Cell::Routed(_) => false,
            Cell::Start(net) | Cell::Target(_,net) => net == self.current_net_processed,
            _ => true,
        }
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
                println!("{},{},{}",nl,nr,nc);
                let nl = nl as usize;
                let nr = nr as usize;
                let nc = nc as usize;
                // Base cost: 1 for horizontal/vertical, via_cost for via
                let mut cost = if dl != 0 { self.via_cost } else { 1 };

                // Directional penalties
                if l % 2 == 0 && dr != 0 {
                    cost += self.nonpreferred_direction_cost;
                } // Vertical cost on even layers
                if l % 2 != 0 && dc != 0 {
                    cost += self.nonpreferred_direction_cost;
                } // Horizontal cost on odd layers

                match self.grid[nl][nr][nc] {
                    Cell::Free | Cell::Candidate(_) | Cell::Target(_,_) => {
                        result.push(((nl, nr, nc), cost))
                    }
                    _ => {}
                }
            }
        }
        result
    }

    // Dijkstra to find the path between start and target
    fn dijkstra(&mut self) {
        let mut queue = BinaryHeap::new(); // Min-heap via Reverse
        let mut parent: HashMap<Coord, Coord> = HashMap::new();

        for &source in &self.start_cords {
            queue.push((Reverse(0), source));
        }

        while let Some((Reverse(cost), (l, r, c))) = queue.pop() {
            if let Cell::Target(_,_) = self.grid[l][r][c] {
                // print!("\nNet {} Cost: {}\n",self.current_net_processed, cost);
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
                    Cell::Target(existing_cost,_) => {
                        if new_cost < existing_cost {
                            self.grid[nl][nr][nc] = Cell::Target(new_cost,self.current_net_processed);
                            parent.insert((nl, nr, nc), (l, r, c));
                            queue.push((Reverse(new_cost), (nl, nr, nc)));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn reconstruct_path(&mut self, end: Coord, parent: &HashMap<Coord, Coord>) {
        let mut current = end;
        let mut path = vec![current];

        while !matches!(self.grid[current.0][current.1][current.2], Cell::Start(_)) {
            let prev = *parent.get(&current).unwrap();

            if current.0 != prev.0 {
                self.vias.insert(current);
                self.vias.insert(prev);
            }

            self.grid[current.0][current.1][current.2] = Cell::Start(self.current_net_processed);
            path.push(prev);
            current = prev;
        }

        path.reverse();
        self.start_cords = path.clone();
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
            let net_num = net
                ._net_name
                .strip_prefix("net")
                .unwrap()
                .parse::<u32>()
                .unwrap();
    
            for pin in &net.pins {
                let coord = pin.coord;
                self.grid[coord.0][coord.1][coord.2] = Cell::Start(net_num);
            }
        }
        
        for net in nets {
            let net_num = net
                ._net_name
                .strip_prefix("net")
                .unwrap()
                .parse::<u32>()
                .unwrap();

            self.current_net_processed = net_num;
            // insert the start pin for this net
            self.set_as_target(&net.pins);
            let start_pin: &Pin = &net.pins[0]; // &net.pins[0]; TODO: to be replaced by a function that gets the closest pin to a corner
            
            for pin in &net.pins {
                self.original_sources.insert(pin.coord);
            }
            self.grid[start_pin.coord.0][start_pin.coord.1][start_pin.coord.2] = Cell::Start(net_num);
            
            //all_sources.push(start);
            self.start_cords.clear();
            self.start_cords.push(start_pin.coord); // Add this source to start_cords

            // 3
        let mut full_path: Vec<Coord> = vec![self.start_cords[0]]; // initial pin
        for _ in 0..net.pins.len() - 1 {
            self.dijkstra();
            self.clear_candidates();
            full_path.extend(self.start_cords.iter().skip(1)); // append newly routed segment
        }
        self.routed_paths.push((format!("net{}", self.current_net_processed), full_path.clone()));
        self.finalize_routing();
        }
        println!("\nFinal Layout");
        self.print_layers_side_by_side();
    }

    fn finalize_routing(&mut self) {
        for source in &self.start_cords {
            match self.grid[source.0][source.1][source.2] {
                Cell::Start(net_num) => {
                    self.grid[source.0][source.1][source.2] = Cell::Routed(net_num)
                } // Mark final sources as routed
                _ => panic!("Routing on non-source") // impossible
            }
        }
    }

    fn set_as_target(&mut self, pins: &Vec<Pin>) {
        for pin in pins {
            self.grid[pin.coord.0][pin.coord.1][pin.coord.2] = Cell::Target(MAX,self.current_net_processed);
        }
    }

    pub fn initialize_obstacles(&mut self, obstacles: &Vec<Coord>) {
        for (layer, x, y) in obstacles {
            if *x < self.width && *y < self.height {
                self.grid[*layer][*x as usize][*y as usize] = Cell::Blocked; // Mark as Blocked in Layer 1
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
                    }else if self.original_sources.contains(&coord){
                        " S ".to_string()
                    }else {
                        match self.grid[0][r][c] {
                            Cell::Free => " . ".to_string(),
                            Cell::Blocked => " # ".to_string(),
                            Cell::Routed(net_num) => format!("{:^3}", net_num),
                            Cell::Start(net_num) => {
                                if self.original_sources.contains(&coord) {
                                    " S ".to_string()
                                } else {
                                    format!("{:^3}", net_num)
                                }
                            }
                            Cell::Target(_,_) => " T ".to_string(),
                            Cell::Candidate(cost) => format!("{:^3}", cost),
                        }
                    };
                    print!("{}", symbol);
                }
            } else {
                print!("{}", "   ".repeat(self.grid[0][0].len()));
            }

            // Layer separation with vertical bar
            print!(" │ ");

            // Layer 1
            if r < self.grid[1].len() {
                for c in 0..self.grid[1][r].len() {
                    let coord = (1, r, c);
                    let symbol = if self.vias.contains(&coord) {
                        " V ".to_string()
                    }else if self.original_sources.contains(&coord){
                        " S ".to_string()
                    }else {
                        match self.grid[1][r][c] {
                            Cell::Free => " . ".to_string(),
                            Cell::Blocked => " # ".to_string(),
                            Cell::Routed(net_num) => format!("{:^3}", net_num),
                            Cell::Start(_) => " S ".to_string(),
                            Cell::Target(_,_) => " T ".to_string(),
                            Cell::Candidate(cost) => format!("{:^3}", cost),
                        }
                    };
                    print!("{}", symbol);
                }
            }

            println!(" │ Row {}", r);
        }
    }
    pub fn export_paths_to_file(&self, filename: &str) -> std::io::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        for (net_name, path) in &self.routed_paths {
            write!(writer, "{} ", net_name)?;
            for &(l, x, y) in path {
                write!(writer, "({},{},{}) ", l + 1, x, y)?; // l + 1 for 1-based layer
            }
            writeln!(writer)?;
        }

        Ok(())
    }

}