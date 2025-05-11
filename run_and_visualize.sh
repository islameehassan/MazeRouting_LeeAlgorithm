#!/bin/bash

# Check if user provided input file
if [ -z "$1" ]; then
    echo "Usage: ./run_and_visualize.sh <input_file.txt>"
    exit 1
fi

# Always look for test cases inside test_cases/
INPUT_FILE="$1"
INPUT_PATH="test_cases/$INPUT_FILE"

# Step 1: Run Rust Maze Router
echo "🔧 Compiling and running Rust Maze Router on: $INPUT_PATH"
cargo run -- "$INPUT_PATH"

if [ $? -ne 0 ]; then
    echo "Rust execution failed."
    exit 1
fi

# Step 2: Check for routed_output.csv
if [ ! -f routed_output.csv ]; then
    echo "routed_output.csv not found. Make sure export_paths_csv() is called in your Rust code."
    exit 1
fi

# Step 3: Check for obstacles.csv (optional)
if [ ! -f obstacles.csv ]; then
    echo "obstacles.csv not found. You may not see obstacle blocks in the visualization."
fi

echo "routed_output.csv generated."

# Step 4: Run Python Visualizer
if [ ! -f visualize_routing.py ]; then
    echo "visualize_routing.py not found in the current directory."
    exit 1
fi

echo "Launching Python visualizer..."
python3 visualize_routing.py

if [ $? -ne 0 ]; then
    echo "Python visualization failed."
    exit 1
fi

echo "Visualization complete."
