use eframe::{egui, App};
use crate::lee_maze::lee_maze::{Maze, Cell}; // Adjust to your structure
use egui::{Color32};

pub struct MazeApp {
    maze: Maze,
    cell_size: f32, // dynamic zoomable cell size
}

impl MazeApp {
    pub fn new(maze: Maze) -> Self {
        Self {
            maze,
            cell_size: 30.0, // default zoom level
        }
    }
}

impl App for MazeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Maze Grid Visualization");
            ui.add_space(10.0);

            // === Zoom slider ===
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.cell_size, 10.0..=100.0).text("Cell size"));
            });
            ui.add_space(10.0);

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (layer_index, layer) in self.maze.grid.iter().enumerate() {
                        ui.group(|ui| {
                            ui.label(format!("Layer {}", layer_index));
                            ui.add_space(5.0);

                            egui::Grid::new(format!("layer_{}", layer_index))
                                .spacing([2.0, 2.0])
                                .show(ui, |ui| {
                                    for (r, row) in layer.iter().enumerate() {
                                        for (c, cell) in row.iter().enumerate() {
                                            let coord = (layer_index, r, c);

                                            // Color logic
                                            let base_color = if self.maze.original_sources.contains(&coord) {
                                                Color32::from_rgb(255, 0, 0)
                                            } else if self.maze.vias.contains(&coord) {
                                                Color32::from_rgb(100, 220, 220)
                                            } else {
                                                match cell {
                                                    Cell::Free => Color32::WHITE,
                                                    Cell::Blocked => Color32::BLACK,
                                                    Cell::Routed(net_id) => net_color(*net_id),
                                                    Cell::Start(_) => Color32::DARK_GREEN,
                                                    Cell::Target(_) => Color32::RED,
                                                    Cell::Candidate(_) => Color32::YELLOW,
                                                }
                                            };

                                            let size = egui::vec2(self.cell_size, self.cell_size);
                                            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                                            let painter = ui.painter();

                                            painter.rect_filled(rect, 4.0, base_color);

                                            if let Cell::Routed(net_id) | Cell::Start(net_id) = cell {
                                                painter.text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    format!("{}", net_id),
                                                    egui::FontId::proportional(self.cell_size * 0.4),
                                                    Color32::BLACK,
                                                );
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);
                    }
                });
        });

        ctx.request_repaint(); // for responsiveness
    }
}

fn net_color(net_id: u8) -> Color32 {
    const COLORS: [Color32; 8] = [
        Color32::from_rgb(31, 119, 180),  // blue
        Color32::from_rgb(255, 127, 14),  // orange
        Color32::from_rgb(44, 160, 44),   // green
        Color32::from_rgb(214, 39, 40),   // red
        Color32::from_rgb(148, 103, 189), // purple
        Color32::from_rgb(140, 86, 75),   // brown
        Color32::from_rgb(227, 119, 194), // pink
        Color32::from_rgb(127, 127, 127), // gray
    ];
    COLORS[(net_id as usize) % COLORS.len()]
}
