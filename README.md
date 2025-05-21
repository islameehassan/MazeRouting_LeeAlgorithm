# Maze Routing Lee Algorithm

This project implements a 3D Maze Routing algorithm based on Lee's algorithm for routing nets in layered grids with obstacles and costs.

---

## How to Run

Run the program with a test case file as an argument:

```bash
cargo run test_cases/test_case19.txt
```
The test case file should specify the grid size, obstacles, and nets as per the input format described below.

## Configuration
Routing costs are controlled via constants defined in src/config/config.rs:
```
static DEFAULT_VIA_COST: i32 = 19;
static DEFAULT_DIRECTION_CHANGE: i32 = 5;
```
DEFAULT_VIA_COST controls the cost of changing layers (vias).
DEFAULT_DIRECTION_CHANGE controls the cost penalty for moving in a non-preferred direction.

## Preferred Movement Directions Logic
Directional penalties are applied as follows in lee_maze.rs:

```
// Directional penalties
if l % 2 == 0 && dr != 0 {
    cost += self.nonpreferred_direction_cost;
} // Vertical movement cost on even layers
if l % 2 != 0 && dc != 0 {
    cost += self.nonpreferred_direction_cost;
} // Horizontal movement cost on odd layers
```
This means:

On even layers (0, 2, 4, ...), vertical moves (dr != 0) are penalized.

On odd layers (1, 3, 5, ...), horizontal moves (dc != 0) are penalized.

This reflects preferred routing directions per layer to optimize routing.

## Output
The program prints the routed maze layers side by side with visual symbols:

. = free cell

\# = obstacle

numbers = routed net IDs

S = Net pins

V = vias (layer transitions)

Routed paths for each net are saved internally and can be exported to a file with export_paths_to_file().

## Notes
Layers stack horizontally in the printout.

Supports up to 2 layers (configurable).

Uses Dijkstra's algorithm with custom costs and penalties.

Via cost and direction penalties help control routing preference and congestion.

Supports large grids but watch performance for very large sizes.


#Presentation: 
https://docs.google.com/presentation/d/1VLQ9Mr4rD6KUDAkzDfa4DGDhjyTWBvuv/edit?usp=sharing&ouid=108143230899188282581&rtpof=true&sd=true
