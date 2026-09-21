use crate::{cpu_grid::CpuGrid, gpu_grid::GpuGrid};

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

pub enum GridType {
    Cpu(CpuGrid),
    Gpu(GpuGrid),
}

impl Grid for GridType {
    #[inline(always)]
    fn width(&self) -> usize {
        match self {
            GridType::Cpu(g) => g.width(),
            GridType::Gpu(g) => g.width(),
        }
    }

    #[inline(always)]
    fn height(&self) -> usize {
        match self {
            GridType::Cpu(g) => g.height(),
            GridType::Gpu(g) => g.height(),
        }
    }

    #[inline(always)]
    fn cell_size(&self) -> usize {
        match self {
            GridType::Cpu(g) => g.cell_size(),
            GridType::Gpu(g) => g.cell_size(),
        }
    }

    #[inline(always)]
    fn draw_mode(&self) -> &DrawMode {
        match self {
            GridType::Cpu(g) => g.draw_mode(),
            GridType::Gpu(g) => g.draw_mode(),
        }
    }

    #[inline(always)]
    fn draw_intensity(&self) -> f64 {
        match self {
            GridType::Cpu(g) => g.draw_intensity(),
            GridType::Gpu(g) => g.draw_intensity(),
        }
    }

    #[inline(always)]
    fn concentrations(&self) -> &[f64] {
        match self {
            GridType::Cpu(g) => g.concentrations(),
            GridType::Gpu(g) => g.concentrations(),
        }
    }

    #[inline(always)]
    fn walls(&self) -> &[u8] {
        match self {
            GridType::Cpu(g) => g.walls(),
            GridType::Gpu(g) => g.walls(),
        }
    }

    #[inline(always)]
    fn set_draw_mode(&mut self) {
        match self {
            GridType::Cpu(g) => g.set_draw_mode(),
            GridType::Gpu(g) => g.set_draw_mode(),
        }
    }

    #[inline(always)]
    fn set_draw_intensity(&mut self, intensity: f64) {
        match self {
            GridType::Cpu(g) => g.set_draw_intensity(intensity),
            GridType::Gpu(g) => g.set_draw_intensity(intensity),
        }
    }

    #[inline(always)]
    fn update(&mut self, diffusion_coefficient: f64, delta: f64) {
        match self {
            GridType::Cpu(g) => g.update(diffusion_coefficient, delta),
            GridType::Gpu(g) => g.update(diffusion_coefficient, delta),
        }
    }

    #[inline(always)]
    fn inject(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        delta: f64,
    ) {
        match self {
            GridType::Cpu(g) => g.inject(start_x, start_y, prev_cell_x, prev_cell_y, delta),
            GridType::Gpu(g) => g.inject(start_x, start_y, prev_cell_x, prev_cell_y, delta),
        }
    }

    #[inline(always)]
    fn clear(&mut self) {
        match self {
            GridType::Cpu(g) => g.clear(),
            GridType::Gpu(g) => g.clear(),
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
