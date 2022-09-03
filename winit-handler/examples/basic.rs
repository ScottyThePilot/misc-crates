extern crate winit;
extern crate winit_handler;

use winit::dpi::PhysicalSize;
use winit::event::{ScanCode, VirtualKeyCode};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_handler::{EventHandler, KeyState, WindowState};

struct Main;

impl EventHandler<()> for Main {
  fn keyboard_input(&mut self, window_state: &WindowState, state: KeyState, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {
    println!("Key {:<10} {:<20} {scancode:#08x}", format!("{state:?}:"), format!("{keycode:?}"));

    match (state, keycode) {
      (KeyState::Pressed, Some(keycode)) => assert!(window_state.input().was_key_pressed(keycode)),
      (KeyState::Repeating, Some(keycode)) => assert!(window_state.input().was_key_repeating(keycode)),
      (KeyState::Released, Some(keycode)) => assert!(window_state.input().was_key_released(keycode)),
      (_, None) => ()
    };
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
