use crate::grid::{DrawMode, Grid};

pub struct CpuGrid {
    width: usize,
    height: usize,
    cell_size: usize,
    draw_mode: DrawMode,
    draw_intensity: f64,
    concentrations: Vec<f64>,
    concentrations_scratch: Vec<f64>,
    sources: Vec<f64>,
    advections: Vec<(f64, f64)>,
    advections_scratch: Vec<(f64, f64)>,
    walls: Vec<u8>,
    pressures: Vec<f64>,
    divergences: Vec<f64>,
}

impl CpuGrid {
    pub fn new(width: usize, height: usize, cell_size: usize) -> Self {
        let grid_width = width / cell_size;
        let grid_height = height / cell_size;
        let total_cells = grid_width * grid_height;

        Self {
            width: grid_width,
            height: grid_height,
            cell_size,
            draw_mode: DrawMode::Gas,
            draw_intensity: 1.0,
            concentrations: vec![0.0; total_cells],
            concentrations_scratch: vec![0.0; total_cells],
            sources: vec![0.0; total_cells],
            advections: vec![(0.0, 0.0); total_cells],
            advections_scratch: vec![(0.0, 0.0); total_cells],
            walls: vec![0; total_cells],
            pressures: vec![0.0; total_cells],
            divergences: vec![0.0; total_cells],
        }
    }

    fn get_value_change(
        cell_size: usize,
        value: f64,
        neighbors: [Option<(f64, usize)>; 4],
        advection_values: (f64, f64),
    ) -> (f64, f64) {
        let concentration_change_x = if advection_values.0 > 0.0
            && let Some(left) = neighbors[0]
        {
            (value - left.0) / cell_size as f64
        } else if advection_values.0 < 0.0
            && let Some(right) = neighbors[1]
        {
            (right.0 - value) / cell_size as f64
        } else {
            0.0
        };

        let concentration_change_y = if advection_values.1 > 0.0
            && let Some(up) = neighbors[2]
        {
            (value - up.0) / cell_size as f64
        } else if advection_values.1 < 0.0
            && let Some(down) = neighbors[3]
        {
            (down.0 - value) / cell_size as f64
        } else {
            0.0
        };

        (concentration_change_x, concentration_change_y)
    }

    fn compute_forward_advections(
        width: usize,
        height: usize,
        cell_size: usize,
        walls: &[u8],
        concentrations: &[f64],
        advections: &[(f64, f64)],
        out: &mut [f64],
        delta: f64,
    ) {
        for idx in 0..out.len() {
            if walls[idx] == 1 {
                out[idx] = concentrations[idx];
                continue;
            }

            let concentration = concentrations[idx];
            let advection_values = advections[idx];
            let neighbors = Self::get_neighbours(idx, width, height, walls, concentrations);
            let value_change =
                Self::get_value_change(cell_size, concentration, neighbors, advection_values);

            out[idx] = concentration
                - delta
                    * (advection_values.0 * value_change.0 + advection_values.1 * value_change.1);
        }
    }

    fn compute_backward_advections(
        width: usize,
        height: usize,
        cell_size: usize,
        walls: &[u8],
        concentrations: &[f64],
        advections: &[(f64, f64)],
        out: &mut [f64],
        forward: &[f64],
        delta: f64,
    ) {
        for idx in 0..out.len() {
            if walls[idx] == 1 {
                out[idx] = concentrations[idx];
                continue;
            }

            let advection_values = advections[idx];
            let forward_val = forward[idx];
            let neighbors = Self::get_neighbours(idx, width, height, walls, forward);
            let value_change = Self::get_value_change(
                cell_size,
                forward_val,
                neighbors,
                (-advection_values.0, -advection_values.1),
            );

            out[idx] = forward_val
                - delta
                    * (-advection_values.0 * value_change.0 + -advection_values.1 * value_change.1);
        }
    }

    fn compute_advections(
        width: usize,
        height: usize,
        cell_size: usize,
        walls: &[u8],
        concentrations: &[f64],
        advections: &[(f64, f64)],
        out: &mut [f64],
        delta: f64,
        scratch: &mut [f64],
    ) {
        Self::compute_forward_advections(
            width,
            height,
            cell_size,
            walls,
            concentrations,
            advections,
            out,
            delta,
        );
        Self::compute_backward_advections(
            width,
            height,
            cell_size,
            walls,
            concentrations,
            advections,
            scratch,
            out,
            delta,
        );
        for idx in 0..out.len() {
            let val = out[idx] + 0.5 * (concentrations[idx] - scratch[idx]);
            let neighbors = Self::get_neighbours(idx, width, height, walls, out);

            let mut min_adv = concentrations[idx];
            let mut max_adv = concentrations[idx];
            for n in neighbors.iter().flatten() {
                min_adv = min_adv.min(n.0);
                max_adv = max_adv.max(n.0);
            }
            out[idx] = val.clamp(min_adv, max_adv);
        }
    }

    fn get_neighbours(
        idx: usize,
        width: usize,
        height: usize,
        walls: &[u8],
        values_grid: &[f64],
    ) -> [Option<(f64, usize)>; 4] {
        let x = idx % width;
        let y = idx / width;

        let left = if x > 0 && walls[idx - 1] == 0 {
            Some((values_grid[idx - 1], idx - 1))
        } else {
            None
        };
        let right = if x + 1 < width && walls[idx + 1] == 0 {
            Some((values_grid[idx + 1], idx + 1))
        } else {
            None
        };
        let up = if y > 0 && walls[idx - width] == 0 {
            Some((values_grid[idx - width], idx - width))
        } else {
            None
        };
        let down = if y + 1 < height && walls[idx + width] == 0 {
            Some((values_grid[idx + width], idx + width))
        } else {
            None
        };

        [left, right, up, down]
    }

    fn project(&mut self) {
        self.divergences.fill(0.0);
        self.pressures.fill(0.0);

        for y in 1..self.height - 1 {
            let row = y * self.width;
            for x in 1..self.width - 1 {
                let idx = row + x;
                let u = self.advections[idx].0;
                let v = self.advections[idx].1;

                let u_left = if self.walls[idx - 1] == 0 {
                    self.advections[idx - 1].0
                } else {
                    0.0
                };
                let v_up = if self.walls[idx - self.width] == 0 {
                    self.advections[idx - self.width].1
                } else {
                    0.0
                };

                self.divergences[idx] =
                    (u - u_left) / self.cell_size as f64 + (v - v_up) / self.cell_size as f64;
            }
        }

        for _ in 0..5 {
            for idx in 0..self.pressures.len() {
                if self.walls[idx] == 1 {
                    continue;
                }

                let neighbors = Self::get_neighbours(
                    idx,
                    self.width,
                    self.height,
                    &self.walls,
                    &self.pressures,
                );
                let mut neighbor_sum = 0.0;
                let mut fluid_count = 0.0;

                for n in neighbors.iter().flatten() {
                    if self.walls[n.1] == 0 {
                        neighbor_sum += n.0;
                    }
                    fluid_count += 1.0;
                }

                if fluid_count > 0.0 {
                    self.pressures[idx] = (neighbor_sum
                        - (self.cell_size * self.cell_size) as f64 * self.divergences[idx])
                        / fluid_count;
                }
            }
        }

        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                let idx = row + x;
                if x < self.width - 1 {
                    if self.walls[idx + 1] == 0 {
                        self.advections[idx].0 -=
                            (self.pressures[idx + 1] - self.pressures[idx]) / self.cell_size as f64;
                    } else {
                        self.advections[idx].0 = 0.0;
                    }
                }
                if y < self.height - 1 {
                    if self.walls[idx + self.width] == 0 {
                        self.advections[idx].1 -= (self.pressures[idx + self.width]
                            - self.pressures[idx])
                            / self.cell_size as f64;
                    } else {
                        self.advections[idx].1 = 0.0;
                    }
                }
            }
        }
    }
}

impl Grid for CpuGrid {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn cell_size(&self) -> usize {
        self.cell_size
    }
    fn draw_mode(&self) -> &DrawMode {
        &self.draw_mode
    }
    fn draw_intensity(&self) -> f64 {
        self.draw_intensity
    }
    fn concentrations(&self) -> &[f64] {
        &self.concentrations
    }
    fn walls(&self) -> &[u8] {
        &self.walls
    }

    fn set_draw_mode(&mut self) {
        self.draw_mode = match self.draw_mode {
            DrawMode::Gas => DrawMode::Source,
            DrawMode::Source => DrawMode::Sink,
            DrawMode::Sink => DrawMode::Advection,
            DrawMode::Advection => DrawMode::Stopper,
            DrawMode::Stopper => DrawMode::Gas,
        };
    }
    fn set_draw_intensity(&mut self, intensity: f64) {
        self.draw_intensity = (self.draw_intensity + intensity).clamp(0.0, 1.0);
    }

    fn clear(&mut self) {
        self.concentrations.fill(0.0);
        self.concentrations_scratch.fill(0.0);
        self.sources.fill(0.0);
        self.advections.fill((0.0, 0.0));
        self.advections_scratch.fill((0.0, 0.0));
        self.walls.fill(0);
    }

    fn update(&mut self, diffusion_coefficient: f64, delta: f64) {
        self.project();
        Self::compute_advections(
            self.width,
            self.height,
            self.cell_size,
            &self.walls,
            &self.concentrations,
            &self.advections,
            &mut self.concentrations_scratch,
            delta,
            &mut self.pressures,
        );

        for idx in 0..self.concentrations.len() {
            if self.walls[idx] == 1 {
                self.concentrations_scratch[idx] = 0.0;
                continue;
            }

            let source_rate = self.sources[idx];
            let advection = self.concentrations_scratch[idx];
            let neighbours = Self::get_neighbours(
                idx,
                self.width,
                self.height,
                &self.walls,
                &self.concentrations_scratch,
            );

            let mut neighbour_sum = 0.0;
            let mut fluid_count = 0.0;

            for neighbour in neighbours.iter().flatten() {
                neighbour_sum += if self.walls[neighbour.1] == 0 {
                    neighbour.0
                } else {
                    advection
                };
                fluid_count += 1.0;
            }

            let computed = advection
                + diffusion_coefficient * delta * (neighbour_sum - fluid_count * advection)
                    / (self.cell_size * self.cell_size) as f64
                + source_rate;

            self.concentrations_scratch[idx] = computed.clamp(0.0, 1.0);
        }

        std::mem::swap(&mut self.concentrations, &mut self.concentrations_scratch);
    }

    fn inject(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        delta: f64,
    ) {
        if start_x >= self.width || start_y >= self.height {
            return;
        }
        let idx = start_y * self.width + start_x;

        match self.draw_mode {
            DrawMode::Gas => self.concentrations[idx] = self.draw_intensity.clamp(0.0, 1.0),
            DrawMode::Source | DrawMode::Sink => {
                let rate = if matches!(self.draw_mode, DrawMode::Source) {
                    self.draw_intensity.abs() / 100.0
                } else {
                    -self.draw_intensity.abs() / 100.0
                };
                self.sources[idx] += rate;
            }
            DrawMode::Advection => {
                let dx = start_x as f64 - prev_cell_x as f64;
                let dy = start_y as f64 - prev_cell_y as f64;
                let strength = 5.0;
                let max_vel = self.cell_size as f64 / delta * 0.5;

                self.advections[idx].0 =
                    (self.advections[idx].0 + dx * strength).clamp(-max_vel, max_vel);
                self.advections[idx].1 =
                    (self.advections[idx].1 + dy * strength).clamp(-max_vel, max_vel);
            }
            DrawMode::Stopper => self.walls[idx] = 1,
        }
    }
}
