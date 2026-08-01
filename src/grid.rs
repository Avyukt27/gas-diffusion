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

#[derive(PartialEq, Eq, Debug)]
pub enum DrawMode {
    Gas,
    Source,
    Sink,
    Advection,
    Stopper,
}
