#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use fastrand;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::SystemTime;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const SCALE_FACTOR: u32 = 4;
const FILL_RATE: f32 = 0.1;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Game of Life")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new(WIDTH / SCALE_FACTOR, HEIGHT / SCALE_FACTOR, FILL_RATE);
    let mut last_update = now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            let now = now();
            if (now - last_update) > 0.5 {
                world.update();
                window.request_redraw();
                last_update = now;
            }
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn now() -> f64 {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!");
    duration.as_secs_f64()
}

struct Cell {
    alive: bool,
}

impl Cell {
    fn update(&mut self, num_neighbours: u8) {
        self.alive = (num_neighbours == 3) || (self.alive && num_neighbours == 2)
    }
}

struct World {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl World {
    fn new(width: u32, height: u32, fill_rate: f32) -> Self {
        let num_cells = (width * height) as usize;
        let mut cells: Vec<Cell> = Vec::with_capacity(num_cells);
        cells.resize_with(num_cells, || Cell {
            alive: fastrand::f32() < fill_rate,
        });

        Self {
            width,
            height,
            cells,
        }
    }

    fn update(&mut self) {
        let mut neighbours: Vec<u8> = Vec::with_capacity(self.cells.len());
        for i in 0..self.cells.len() {
            let w = self.width as usize;
            let h = self.height as usize;
            let x = i % w;
            let y = i / w;
            let mut neighbour_coords: Vec<usize> = Vec::new();

            if y > 0 {
                if x > 0 {
                    neighbour_coords.push(i - w - 1);
                }
                if x < (w - 1) {
                    neighbour_coords.push(i - w + 1);
                }
                neighbour_coords.push(i - w)
            }
            if y < (h - 1) {
                if x > 0 {
                    neighbour_coords.push(i + w - 1);
                }
                if x < (w - 1) {
                    neighbour_coords.push(i + w + 1);
                }
                neighbour_coords.push(i + w)
            }
            if x > 0 {
                neighbour_coords.push(i - 1);
            }
            if x < (w - 1) {
                neighbour_coords.push(i + 1);
            }

            let num_neighbours = neighbour_coords
                .into_iter()
                .filter(|j| self.cells[*j].alive)
                .count();

            neighbours.push(num_neighbours as u8);
        }

        for i in 0..self.cells.len() {
            self.cells[i].update(neighbours[i]);
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as u32;
            let y = (i / WIDTH as usize) as u32;
            let j = ((y / SCALE_FACTOR) * self.width + (x / SCALE_FACTOR)) as usize;
            let rgba = if self.cells[j].alive {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
