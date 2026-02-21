pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cell_size: usize,
    pub concentrations: Vec<f64>,
}

impl Grid {
    pub fn new(width: usize, height: usize, cell_size: usize) -> Self {
        let grid_width = width / cell_size;
        let grid_height = height / cell_size;

        Self {
            width,
            height,
            cell_size,
            concentrations: vec![0.0; grid_width * grid_height],
        }
    }

    pub fn draw(buffer: &mut Vec<u32>) {}
}
