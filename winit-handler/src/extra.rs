use super::*;

use winit::event::*;

use std::error::Error;



/// Handles events emitted by a winit event loop, but with built-in error handling.
/// If any functions return an error, the application will close on the next frame, and the
/// `destroy` function will handle any errors that occurred.
#[allow(unused_variables)]
pub trait EventHandlerFallible<E: Error>: Sized + 'static {
  /// Called upon [`Event::RedrawRequested`][winit::event::Event::RedrawRequested].
  fn render(&mut self, window_state: &WindowState) -> Result<(), E> { Ok(()) }
  /// Called upon [`Event::MainEventsCleared`][winit::event::Event::MainEventsCleared].
  fn update(&mut self, window_state: &WindowState) -> Result<(), E> { Ok(()) }
  /// Called when an event from the keyboard has been received.
  fn keyboard_input(&mut self, window_state: &WindowState, state: KeyState, keycode: Option<VirtualKeyCode>, scancode: ScanCode) -> Result<(), E> { Ok(()) }
  /// Called when the window receives a unicode character.
  fn text_input(&mut self, window_state: &WindowState, ch: char) -> Result<(), E> { Ok(()) }
  /// Called when the cursor has moved on the window.
  fn cursor_moved(&mut self, window_state: &WindowState, pos: (f32, f32)) -> Result<(), E> { Ok(()) }
  /// Called when a mouse button press has been received.
  fn mouse_input(&mut self, window_state: &WindowState, state: ElementState, button: MouseButton) -> Result<(), E> { Ok(()) }
  /// Called when a mouse wheel or touchpad scroll occurs.
  fn mouse_scroll(&mut self, window_state: &WindowState, delta: (f32, f32)) -> Result<(), E> { Ok(()) }
  /// Called when the application loses or gains focus.
  fn focused(&mut self, window_state: &WindowState, state: bool) -> Result<(), E> { Ok(()) }
  /// Called when a file is dropped in the application window.
  fn file_dropped(&mut self, window_state: &WindowState, path: PathBuf) -> Result<(), E> { Ok(()) }
  /// Called when either the window has been resized or the scale factor has changed.
  fn resized(&mut self, window_state: &WindowState, window_size: (u32, u32), scale_factor: f64) -> Result<(), E> { Ok(()) }
  /// Called when the user attempts to close the application.
  /// A return value of `Ok(true)` closes the application, while `Ok(false)` cancels closing it.
  /// Defaults to an 'always `Ok(true)`' implementation.
  fn close(&mut self, window_state: &WindowState) -> Result<bool, E> { Ok(true) }
  /// Instructs the event dispatcher whether the handler wants the application to exit.
  /// This function does not return a result because a program should (logically) be able to determine whether it should close without fail.
  /// Defaults to an 'always `false`' implementation.
  fn should_exit(&self, window_state: &WindowState) -> bool { false }
  /// Called once the event loop has been destroyed and will no longer dispatch any more events.
  /// This is different from the `close` function in that the handler has no choice over the application state.
  /// It is mandatory to implement this function, as it serves as the error handling mechanism.
  fn destroy(self, errors: impl IntoIterator<Item = E>);
}



#[derive(Debug)]
pub struct FallibleWrapper<H, E> {
  handler: H,
  errors: Vec<E>
}

impl<H, E> FallibleWrapper<H, E>
where H: EventHandlerFallible<E>, E: Error + 'static {
  #[inline]
  pub fn new(handler: H) -> Self {
    FallibleWrapper { handler, errors: Vec::new() }
  }
}

macro_rules! handler_functions {
  ($(fn $function:ident(&mut self $(, $arg:ident: $type:ty)*);)*) => (
    $(#[inline] fn $function(&mut self, $($arg: $type),*) {
      match self.handler.$function($($arg),*) {
        Ok(()) => (),
        Err(error) => self.errors.push(error)
      };
    })*
  );
}

impl<H, E> EventHandler for FallibleWrapper<H, E>
where H: EventHandlerFallible<E>, E: Error + 'static {
  handler_functions!{
    fn render(&mut self, window_state: &WindowState);
    fn update(&mut self, window_state: &WindowState);
    fn keyboard_input(&mut self, window_state: &WindowState, state: KeyState, keycode: Option<VirtualKeyCode>, scancode: ScanCode);
    fn text_input(&mut self, window_state: &WindowState, ch: char);
    fn cursor_moved(&mut self, window_state: &WindowState, pos: (f32, f32));
    fn mouse_input(&mut self, window_state: &WindowState, state: ElementState, button: MouseButton);
    fn mouse_scroll(&mut self, window_state: &WindowState, delta: (f32, f32));
    fn focused(&mut self, window_state: &WindowState, state: bool);
    fn file_dropped(&mut self, window_state: &WindowState, path: PathBuf);
    fn resized(&mut self, window_state: &WindowState, window_size: (u32, u32), scale_factor: f64);
  }

  fn close(&mut self, window_state: &WindowState) -> bool {
    match self.handler.close(window_state) {
      Ok(close) => close || !self.errors.is_empty(),
      Err(error) => {
        self.errors.push(error);
        true
      }
    }
  }

  #[inline]
  fn should_exit(&self, window_state: &WindowState) -> bool {
    !self.errors.is_empty() || self.handler.should_exit(window_state)
  }

  #[inline]
  fn destroy(self) {
    self.handler.destroy(self.errors)
  }
}
