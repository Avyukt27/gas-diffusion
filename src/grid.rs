use crate::colour::Colour;

pub struct Grid {
    pub screen_width: usize,
    pub screen_height: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: usize,
    pub concentrations: Vec<f64>,
}

impl Grid {
    pub fn new(width: usize, height: usize, cell_size: usize) -> Self {
        let grid_width = width / cell_size;
        let grid_height = height / cell_size;

        Self {
            screen_width: width,
            screen_height: height,
            grid_width,
            grid_height,
            cell_size,
            concentrations: vec![0.0; grid_width * grid_height],
        }
    }

    pub fn draw(&self, buffer: &mut Vec<u32>) {
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let concentration = self.concentrations[y * self.grid_width + x];
                let intensity = (concentration.clamp(0.0, 1.0) * 255.0) as u8;
                let colour = Colour::new(intensity, intensity, intensity).to_u32();

                for dy in 0..self.cell_size {
                    for dx in 0..self.cell_size {
                        let pixel_x = x * self.cell_size + dx;
                        let pixel_y = y * self.cell_size + dy;
                        buffer[pixel_y * self.screen_width + pixel_x] = colour;
                    }
                }
            }
        }
    }

    fn get_neighbors(&self, idx: usize) -> [Option<f64>; 4] {
        let x = idx % self.grid_width;
        let y = idx / self.grid_height;

        let left = if x > 0 {
            Some(self.concentrations[idx - 1])
        } else {
            None
        };
        let right = if x + 1 < self.grid_width {
            Some(self.concentrations[idx + 1])
        } else {
            None
        };
        let up = if y > 0 {
            Some(self.concentrations[idx - self.grid_width])
        } else {
            None
        };
        let down = if y + 1 < self.grid_height {
            Some(self.concentrations[idx + self.grid_width])
        } else {
            None
        };
        [left, right, up, down]
    }
}
