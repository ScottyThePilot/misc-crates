extern crate winit;
extern crate winit_handler;

use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_handler::{EventHandler, State};

struct Main;

impl EventHandler<()> for Main {
  fn should_exit(&self, state: &State) -> bool {
    state.input().key_held(VirtualKeyCode::Q) && state.input().modifiers().ctrl()
  }
}

fn main() {
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_title("winit-handler basic example")
    .with_inner_size(PhysicalSize::<u32>::from((384, 256)))
    .build(&event_loop).unwrap();
  winit_handler::run(event_loop, window, Main);
}
