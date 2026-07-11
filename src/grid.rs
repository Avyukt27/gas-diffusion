pub trait Grid {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cell_size(&self) -> usize;
    fn draw_mode(&self) -> &DrawMode;
    fn draw_intensity(&self) -> f64;
    fn concentrations(&self) -> &[f64];
    fn walls(&self) -> &[u8];

    fn set_draw_mode(&mut self);
    fn set_draw_intensity(&mut self, intensity: f64);

    fn update(&mut self, diffusion_coefficient: f64, delta: f64);
    fn inject(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        delta: f64,
    );

    fn clear(&mut self);
}

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

        Self {
            width: grid_width,
            height: grid_height,
            cell_size,
            draw_mode: DrawMode::Gas,
            draw_intensity: 1.0,

            concentrations: vec![0.0; grid_width * grid_height],
            concentrations_scratch: vec![0.0; grid_width * grid_height],
            sources: vec![0.0; grid_width * grid_height],
            advections: vec![(0.0, 0.0); grid_width * grid_height],
            advections_scratch: vec![(0.0, 0.0); grid_width * grid_height],
            walls: vec![0; grid_width * grid_height],
            pressures: vec![0.0; grid_width * grid_height],
            divergences: vec![0.0; grid_width * grid_height],
        }
    }

    fn get_value_change(
        &self,
        value: f64,
        neighbors: [Option<(f64, usize)>; 4],
        advection_values: (f64, f64),
    ) -> (f64, f64) {
        let concentration_change_x = if advection_values.0 > 0.0
            && let Some(concentration_left) = neighbors[0]
        {
            (value - concentration_left.0) / self.cell_size as f64
        } else if advection_values.0 < 0.0
            && let Some(concentration_right) = neighbors[1]
        {
            (concentration_right.0 - value) / self.cell_size as f64
        } else {
            0.0
        };
        let concentration_change_y = if advection_values.1 > 0.0
            && let Some(concentration_up) = neighbors[2]
        {
            (value - concentration_up.0) / self.cell_size as f64
        } else if advection_values.1 < 0.0
            && let Some(concentration_down) = neighbors[3]
        {
            (concentration_down.0 - value) / self.cell_size as f64
        } else {
            0.0
        };

        (concentration_change_x, concentration_change_y)
    }

    fn get_forward_advections(&self, delta: f64) -> Vec<f64> {
        let mut forward_advections: Vec<f64> = vec![0.0; self.width * self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.walls[idx] == 1 {
                    forward_advections[idx] = self.concentrations[idx];
                    continue;
                }

                let concentration = self.concentrations[idx];
                let advection_values = self.advections[idx];
                let neighbors = self.get_neighbors(idx, &self.concentrations);

                let value_change =
                    self.get_value_change(concentration, neighbors, advection_values);

                let forward_advection = concentration
                    - delta
                        * (advection_values.0 * value_change.0
                            + advection_values.1 * value_change.1);

                forward_advections[idx] = forward_advection;
            }
        }

        forward_advections
    }

    fn get_backward_advections(&self, forward_advections: &Vec<f64>, delta: f64) -> Vec<f64> {
        let mut backward_advections: Vec<f64> = vec![0.0; self.width * self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.walls[idx] == 1 {
                    backward_advections[idx] = self.concentrations[idx];
                    continue;
                }

                let advection_values = self.advections[idx];
                let forward_advection = forward_advections[idx];
                let neighbors = self.get_neighbors(idx, &forward_advections);

                let value_change = self.get_value_change(
                    forward_advection,
                    neighbors,
                    (-advection_values.0, -advection_values.1),
                );

                let backward_advection = forward_advection
                    - delta
                        * (-advection_values.0 * value_change.0
                            + -advection_values.1 * value_change.1);

                backward_advections[idx] = backward_advection;
            }
        }

        backward_advections
    }

    fn get_corrections(&self, delta: f64) -> Vec<f64> {
        let forward_advections = self.get_forward_advections(delta);
        let backward_advections = self.get_backward_advections(&forward_advections, delta);
        let mut corrections: Vec<f64> = vec![0.0; self.width * self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let concentration = self.concentrations[idx];
                let forward_advection = forward_advections[idx];
                let backward_advection = backward_advections[idx];
                corrections[idx] = forward_advection + 0.5 * (concentration - backward_advection);
            }
        }

        corrections
    }

    fn get_advections(&self, delta: f64) -> Vec<f64> {
        let corrections = self.get_corrections(delta);
        let mut advections: Vec<f64> = vec![0.0; self.width * self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let concentration = self.concentrations[idx];
                let correction = corrections[idx];
                let neighbors = self.get_neighbors(idx, &corrections);

                let mut min_advection = concentration;
                let mut max_advection = concentration;
                for neighbor in neighbors {
                    match neighbor {
                        Some(n) => {
                            min_advection = min_advection.min(n.0);
                            max_advection = max_advection.max(n.0);
                        }
                        None => {}
                    }
                }
                advections[idx] = correction.clamp(min_advection, max_advection);
            }
        }

        advections
    }

    fn project(&mut self) {
        let mut divergences = vec![0.0; self.width * self.height];
        let mut pressures = vec![0.0; self.width * self.height];

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                let u = self.advections[idx].0;
                let v = self.advections[idx].1;

                let u_left = if self.walls[y * self.width + (x - 1)] == 0 {
                    self.advections[y * self.width + (x - 1)].0
                } else {
                    0.0
                };
                let v_up = if self.walls[(y - 1) * self.width + x] == 0 {
                    self.advections[(y - 1) * self.width + x].1
                } else {
                    0.0
                };

                let divergence =
                    (u - u_left) / self.cell_size as f64 + (v - v_up) / self.cell_size as f64;

                divergences[idx] = divergence;
            }
        }

        for _ in 0..5 {
            for y in 0..self.height {
                let row = y * self.width;
                for x in 0..self.width {
                    let idx = row + x;
                    if self.walls[idx] == 1 {
                        continue;
                    }
                    let neighbors = self.get_neighbors(idx, &pressures);

                    let mut neighbor_sum = 0.0;
                    let mut fluid_count = 0.0;

                    for neighbor in neighbors.iter() {
                        if let Some((value, idx)) = neighbor {
                            if self.walls[*idx] == 0 {
                                neighbor_sum += value;
                            }
                            fluid_count += 1.0;
                        }
                    }

                    pressures[idx] = (neighbor_sum
                        - self.cell_size as f64 * self.cell_size as f64 * divergences[idx])
                        / fluid_count;
                }
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if x < self.width - 1 {
                    let right = y * self.width + (x + 1);
                    if self.walls[right] == 0 {
                        self.advections[idx].0 -=
                            (pressures[right] - pressures[idx]) / self.cell_size as f64;
                    } else {
                        self.advections[idx].0 = 0.0
                    }
                }
                if y < self.height - 1 {
                    let down = (y + 1) * self.width + x;
                    if self.walls[down] == 0 {
                        self.advections[idx].1 -=
                            (pressures[down] - pressures[idx]) / self.cell_size as f64;
                    } else {
                        self.advections[idx].1 = 0.0
                    }
                }
            }
        }
    }

    fn get_neighbors(&self, idx: usize, values_grid: &Vec<f64>) -> [Option<(f64, usize)>; 4] {
        let x = idx % self.width;
        let y = idx / self.width;

        let left = if x > 0 {
            let neighbor_idx = y * self.width + (x - 1);
            if self.walls[neighbor_idx] == 0 {
                Some((values_grid[neighbor_idx], idx - 1))
            } else {
                None
            }
        } else {
            None
        };
        let right = if x + 1 < self.width {
            let neighbor_idx = y * self.width + (x + 1);
            if self.walls[neighbor_idx] == 0 {
                Some((values_grid[neighbor_idx], idx + 1))
            } else {
                None
            }
        } else {
            None
        };
        let up = if y > 0 {
            let neighbor_idx = (y - 1) * self.width + x;
            if self.walls[neighbor_idx] == 0 {
                Some((values_grid[neighbor_idx], idx - self.width))
            } else {
                None
            }
        } else {
            None
        };
        let down = if y + 1 < self.height {
            let neighbor_idx = (y + 1) * self.width + x;
            if self.walls[neighbor_idx] == 0 {
                Some((values_grid[neighbor_idx], idx + self.width))
            } else {
                None
            }
        } else {
            None
        };
        [left, right, up, down]
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
        match self.draw_mode {
            DrawMode::Gas => self.draw_mode = DrawMode::Source,
            DrawMode::Source => self.draw_mode = DrawMode::Sink,
            DrawMode::Sink => self.draw_mode = DrawMode::Advection,
            DrawMode::Advection => self.draw_mode = DrawMode::Stopper,
            DrawMode::Stopper => self.draw_mode = DrawMode::Gas,
        }
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
        let mut next = self.concentrations.clone();
        let advections = self.get_advections(delta);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if self.walls[idx] == 1 {
                    next[idx] = 0.0;
                    continue;
                }
                let source_rate = self.sources[idx];
                let advection = advections[idx];

                let neighbors = self.get_neighbors(idx, &advections);
                let mut neighbor_sum = 0.0;
                let mut fluid_count = 0.0;

                for neighbor in neighbors.iter() {
                    if let Some((value, idx)) = neighbor {
                        if self.walls[*idx] == 0 {
                            neighbor_sum += value;
                        } else {
                            neighbor_sum += advection;
                        }
                        fluid_count += 1.0;
                    }
                }

                let computed_concentration = advection
                    + diffusion_coefficient * delta * (neighbor_sum - fluid_count * advection)
                        / (self.cell_size * self.cell_size) as f64
                    + source_rate;

                if computed_concentration > 1.0 {
                    next[idx] = 1.0;
                } else if computed_concentration < 1e-4 {
                    next[idx] = 0.0;
                } else {
                    next[idx] = computed_concentration;
                }
            }
        }

        self.concentrations = next;
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
            DrawMode::Gas => {
                self.concentrations[idx] = self.draw_intensity.clamp(0.0, 1.0);
            }
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
                let vel = (dx * strength, dy * strength);
                let max_vel = self.cell_size as f64 / delta * 0.5;

                self.advections[idx].0 = (self.advections[idx].0 + vel.0).clamp(-max_vel, max_vel);
                self.advections[idx].1 = (self.advections[idx].1 + vel.1).clamp(-max_vel, max_vel);
            }
            DrawMode::Stopper => self.walls[idx] = 0,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum DrawMode {
    Gas,
    Source,
    Sink,
    Advection,
    Stopper,
}
