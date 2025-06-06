use std::error::Error;
use std::fs;
use std::result::Result;

static DEFAULT_VIA_COST: i32 = 19;
static DEFAULT_DIRECTION_CHANGE: i32 = 5;

use crate::Coord;
use crate::{Net, Pin};
#[derive(Debug)]
pub struct Config {
    pub grid_width: u16,
    pub grid_height: u16,
    pub obstacles: Vec<Coord>,
    pub nets: Vec<Net>, // each net has a vector of pins
    pub via_cost: i32, // assuming a very high cost (can have a default value if not specified by the user)
    pub nonpreferred_direction_cost: i32,
}

impl Config {
    fn parse_grid_dims(line: &str) -> Result<(u16, u16), &'static str> {
        let dims: Result<Vec<u16>, _> = line.split('x').map(|s| s.trim().parse::<u16>()).collect();

        match dims {
            Ok(vec) if vec.len() == 2 => Ok((vec[0], vec[1])),
            _ => Err("Invalid dimension format. Expected format like 10x20."),
        }
    }

    fn parse_obs(line: &str) -> Result<Coord, &'static str> {
        let content = line
            .strip_prefix("OBS (")
            .and_then(|s| s.strip_suffix(")"))
            .ok_or("Invalid OBS format")?;

        let coordinates: Result<Vec<usize>, _> = content
            .split(",")
            .map(|s| s.trim().parse::<usize>())
            .collect();

        match coordinates {
            Ok(vec) if vec.len() == 3 => Ok((vec[0] - 1, vec[1], vec[2])),
            _ => Err("Invalid OBS format. Expected format like OBS (1,15,32)"),
        }
    }

    fn parse_all_obs<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<Vec<Coord>, &'static str>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut obstacles: Vec<Coord> = Vec::new();

        while let Some(line) = lines.peek() {
            if !line.trim_start().starts_with("OBS") {
                break;
            }

            // If it matches, consume the line
            let obs = Self::parse_obs(lines.next().unwrap())?;
            obstacles.push(obs);
        }
        Ok(obstacles)
    }

    fn parse_net(line: &str) -> Result<Net, &'static str> {
        if !line.contains(" (") {
            return Err("Invalid net format");
        }

        let mut parts = line.trim().split("(");

        let net_name = parts.next().ok_or("Missing net name")?.trim().to_string();

        let mut pins: Vec<Pin> = vec![];
        for part in parts {
            if let Some(tuple) = part.trim().strip_suffix(')') {
                let nums: Vec<&str> = tuple.split(',').map(|s| s.trim()).collect();
                if nums.len() == 3 {
                    let mut layer_num = nums[0].parse::<usize>().map_err(|_| "Invalid int")?;
                    layer_num -= 1; // for ease of indexing later
                    let pin_x = nums[1].parse::<usize>().map_err(|_| "Invalid int")?;
                    let pin_y = nums[2].parse::<usize>().map_err(|_| "Invalid int")?;
                    let coord = (layer_num, pin_x, pin_y);
                    pins.push(Pin { coord: coord });
                } else {
                    return Err("Expected 3 values in the net pin tuple");
                }
            } else {
                return Err("Invalid net tuple format");
            }
        }
        return Ok(Net {
            _net_name: net_name,
            pins: pins,
        });
    }

    fn parse_all_nets<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<Vec<Net>, &'static str>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut nets: Vec<Net> = Vec::new();

        while let Some(line) = lines.peek() {
            if !line.trim_start().starts_with("net") {
                break;
            }
            let net: Net = Self::parse_net(lines.next().unwrap())?;
            nets.push(net);
        }
        Ok(nets)
    }

    fn parse_extra_costs<'a, I>(lines: &mut I) -> (i32, i32)
    where
        I: Iterator<Item = &'a str>,
    {
        let via_cost: i32 = lines
            .next()
            .and_then(|line| line.split_whitespace().nth(1)) // get the second word
            .and_then(|val| val.parse::<i32>().ok())
            .unwrap_or(DEFAULT_VIA_COST);

        let direction_change_cost: i32 = lines
            .next()
            .and_then(|line| line.split_whitespace().nth(1)) // get the second word
            .and_then(|val| val.parse::<i32>().ok())
            .unwrap_or(DEFAULT_DIRECTION_CHANGE);

        (via_cost, direction_change_cost)
    }

    pub fn build(filename: &str) -> Result<Config, Box<dyn Error>> {
        let contents = fs::read_to_string(filename)?; // ? delegates error handling to the caller
        let mut lines = contents.lines().peekable();

        let first_line = lines.next().ok_or("Input file is empty")?;
        let (grid_width, grid_height) = Self::parse_grid_dims(first_line)?;
        let obstacles = Self::parse_all_obs(&mut lines)?;
        let nets = Self::parse_all_nets(&mut lines)?;
        let (via_cost, direction_change_cost) = Self::parse_extra_costs(&mut lines);

        Ok(Config {
            grid_width: grid_width,
            grid_height: grid_height,
            obstacles: obstacles,
            nets: nets,
            via_cost: via_cost,
            nonpreferred_direction_cost: direction_change_cost,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::config::Config;

    #[test]
    fn test_parse_grid_dims_valid() {
        assert_eq!(Config::parse_grid_dims("10x20"), Ok((10, 20)));
        assert_eq!(Config::parse_grid_dims("  30 x 40  "), Ok((30, 40)));
    }

    #[test]
    fn test_parse_grid_dims_invalid() {
        assert!(Config::parse_grid_dims("10-20").is_err());
        assert!(Config::parse_grid_dims("10x").is_err());
        assert!(Config::parse_grid_dims("x20").is_err());
        assert!(Config::parse_grid_dims("abcxdef").is_err());
    }

    #[test]
    fn test_parse_obs_valid() {
        assert_eq!(Config::parse_obs("OBS (1,12, 34)"), Ok((0, 12, 34)));
        assert_eq!(Config::parse_obs("OBS (2, 1 , 2 )"), Ok((1, 1, 2)));
    }

    #[test]
    fn test_parse_obs_invalid() {
        assert!(Config::parse_obs("OBS 12, 34").is_err());
        assert!(Config::parse_obs("OBS ()").is_err());
        assert!(Config::parse_obs("OBS (1,)").is_err());
        assert!(Config::parse_obs("OBS (a,b)").is_err());
    }

    #[test]
    fn test_parse_all_obs_stops_on_non_obs() {
        let mut lines = vec!["OBS (1,1,2)", "OBS (2,3,4)", "net1 (1,2,3)"]
            .into_iter()
            .peekable();
        let result = Config::parse_all_obs(&mut lines).unwrap();
        assert_eq!(result, vec![(0, 1, 2), (1, 3, 4)]);
    }

    #[test]
    fn test_parse_net_valid() {
        let line = "net1 (1, 10, 20) (2, 30, 40)";
        let net = Config::parse_net(line).unwrap();
        assert_eq!(net._net_name, "net1");
        assert_eq!(net.pins.len(), 2);
        assert_eq!(net.pins[0].coord.0, 0);
        assert_eq!(net.pins[0].coord.1, 10);
        assert_eq!(net.pins[0].coord.2, 20);
    }

    #[test]
    fn test_parse_net_invalid_format() {
        let line = "net1 1, 10, 20)";
        assert!(Config::parse_net(line).is_err());
    }

    #[test]
    fn test_parse_all_nets_stops_on_non_net() {
        let mut lines = vec!["net1 (1, 2, 3)", "net2 (2, 3, 4)", "via_cost 100"]
            .into_iter()
            .peekable();
        let nets = Config::parse_all_nets(&mut lines).unwrap();
        assert_eq!(nets.len(), 2);
        assert_eq!(nets[0]._net_name, "net1");
    }

    #[test]
    fn test_parse_extra_costs_with_valid_lines() {
        let mut lines = vec!["via_cost 123", "direction_change_cost 456"].into_iter();
        let (via, dir) = Config::parse_extra_costs(&mut lines);
        assert_eq!(via, 123);
        assert_eq!(dir, 456);
    }

    #[test]
    fn test_build_full_config() {
        let input = "\
10x20
OBS (1,1, 2)
OBS (2,3, 4)
net1 (1, 10, 20) (2, 30, 40)
net2 (1, 5, 5)
via_cost 10
direction_change_cost 5";

        let filename = "test_input.txt";
        std::fs::write(filename, input).unwrap();

        let config = Config::build(filename).unwrap();
        //println!("{:?}",config);
        assert_eq!(config.grid_width, 10);
        assert_eq!(config.grid_height, 20);
        assert_eq!(config.obstacles.len(), 2);
        assert_eq!(config.nets.len(), 2);
        assert_eq!(config.via_cost, 10);
        assert_eq!(config.nonpreferred_direction_cost, 5);

        std::fs::remove_file(filename).unwrap();
    }
}
