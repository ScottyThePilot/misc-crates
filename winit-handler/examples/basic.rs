extern crate winit;
extern crate winit_handler;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, ScanCode, VirtualKeyCode};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_handler::{EventHandler, WindowState};

struct Main;

impl EventHandler<()> for Main {
  fn keyboard_input(&mut self, _: &WindowState, state: ElementState, keycode: Option<VirtualKeyCode>, _: ScanCode) {
    println!("Key {state:?}: {keycode:?}");
  }

  fn should_exit(&self, window_state: &WindowState) -> bool {
    window_state.input().is_key_held(VirtualKeyCode::Q) && window_state.input().modifiers().ctrl()
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
