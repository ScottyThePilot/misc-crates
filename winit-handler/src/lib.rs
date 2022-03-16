//! A trait-based event handler for [`winit`][winit].
#![warn(missing_copy_implementations, missing_debug_implementations)]

use std::ops::Index;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bitvec::array::BitArray;
use winit::dpi::{PhysicalSize, LogicalPosition};
use winit::event::*;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;



/// Handles events emitted by a winit event loop.
#[allow(unused_variables)]
pub trait EventHandler<T>: Sized + 'static {
  /// Called upon [`Event::RedrawRequested`][winit::event::Event::RedrawRequested].
  fn render(&mut self, state: &State) {}
  /// Called upon [`Event::MainEventsCleared`][winit::event::Event::MainEventsCleared].
  fn update(&mut self, state: &State) {}
  /// Called when an event is sent from [`EventLoopProxy::send_event`][winit::event_loop::EventLoopProxy::send_event].
  fn user_event(&mut self, state: &State, event: T) {}
  /// Called when an event from the keyboard has been received.
  fn keyboard_input(&mut self, state: &State, cond: bool, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {}
  /// Called when the window receives a unicode character.
  fn text_input(&mut self, state: &State, ch: char) {}
  /// Called when the cursor has moved on the window.
  fn cursor_moved(&mut self, state: &State, pos: (f32, f32)) {}
  /// Called when a mouse button press has been received.
  fn mouse_input(&mut self, state: &State, cond: bool, button: MouseButton) {}
  /// Called when a mouse wheel or touchpad scroll occurs.
  fn mouse_scroll(&mut self, state: &State, delta: (f32, f32)) {}
  /// Called when the application loses or gains focus.
  fn focused(&mut self, state: &State, cond: bool) {}
  /// Called when a file is dropped in the application window.
  fn file_dropped(&mut self, state: &State, path: PathBuf) {}
  /// Called when either the window has been resized or the scale factor has changed.
  fn resized(&mut self, state: &State, window_size: (u32, u32), scale_factor: f64) {}
  /// Called when the user attempts to close the application.
  /// A return value of `true` closes the application, while `false` cancels closing it.
  /// Defaults to an 'always `true`' implementation.
  fn close(&mut self, state: &State) -> bool { true }
  /// Instructs the event dispatcher whether the handler wants the application to exit.
  /// Defaults to an 'always `false`' implementation.
  fn should_exit(&self, state: &State) -> bool { false }
  /// Called once the event loop has been destroyed and will no longer dispatch any more events.
  /// This is different from the `close` function in that the handler has no choice over the application state.
  fn destroy(self) {}
}

const KEYCODES: usize = 192;
const SCANCODES: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct InputState {
  cursor_pos: Option<(f32, f32)>,
  cursor_pos_prev: Option<(f32, f32)>,
  mouse_actions: Vec<MouseAction>,
  mouse_left_held: bool,
  mouse_right_held: bool,
  mouse_middle_held: bool,
  key_actions: Vec<KeyAction>,
  // More than 162 bits, enough space to store the state of every key
  keys_held_keycode: BitArray<[u32; KEYCODES / 32]>,
  keys_held_scancode: BitArray<[u32; SCANCODES / 32]>,
  modifiers_state: ModifiersState,
  scroll_rel: (f32, f32),
  text: String
}

impl InputState {
  #[inline]
  pub fn cursor_pos(&self) -> Option<(f32, f32)> {
    self.cursor_pos
  }

  #[inline]
  pub fn cursor_pos_prev(&self) -> Option<(f32, f32)> {
    self.cursor_pos_prev
  }

  #[inline]
  pub fn cursor_pos_rel(&self) -> Option<(f32, f32)> {
    if let (Some(pos), Some(pos_prev)) = (self.cursor_pos, self.cursor_pos_prev) {
      Some((pos.0 - pos_prev.0, pos.1 - pos_prev.1))
    } else {
      None
    }
  }

  #[inline]
  pub fn mouse_actions(&self) -> &[MouseAction] {
    &self.mouse_actions
  }

  #[inline]
  pub fn button_held(&self, button: MouseButton) -> bool {
    self[button]
  }

  #[inline]
  pub fn key_actions(&self) -> &[KeyAction] {
    &self.key_actions
  }

  #[inline]
  pub fn key_held(&self, keycode: VirtualKeyCode) -> bool {
    self[keycode]
  }

  #[inline]
  pub fn key_held_scancode(&self, scancode: ScanCode) -> bool {
    self[scancode]
  }

  pub fn key_pressed(&self, keycode: VirtualKeyCode) -> bool {
    self.key_actions.iter()
      .find(|&&action| match action {
        KeyAction::Pressed(candidate, _) => candidate == keycode,
        _ => false
      })
      .is_some()
  }

  pub fn key_released(&self, keycode: VirtualKeyCode) -> bool {
    self.key_actions.iter()
      .find(|&&action| match action {
        KeyAction::Released(candidate, _) => candidate == keycode,
        _ => false
      })
      .is_some()
  }

  #[inline]
  pub fn scroll_rel(&self) -> (f32, f32) {
    self.scroll_rel
  }

  #[inline]
  pub fn text(&self) -> &str {
    &self.text
  }

  #[inline]
  pub fn modifiers(&self) -> ModifiersState {
    self.modifiers_state
  }

  fn set_button_value(&mut self, button: MouseButton, value: bool) {
    match button {
      MouseButton::Left => self.mouse_left_held = value,
      MouseButton::Right => self.mouse_right_held = value,
      MouseButton::Middle => self.mouse_middle_held = value,
      MouseButton::Other(_) => ()
    };
  }

  fn reset(&mut self) {
    self.cursor_pos_prev = self.cursor_pos;
    self.mouse_actions = Vec::new();
    self.key_actions = Vec::new();
    self.scroll_rel = (0.0, 0.0);
    self.text.clear();
  }

  fn handle_keyboard_input(&mut self, input: KeyboardInput) -> (bool, Option<VirtualKeyCode>, ScanCode) {
    let KeyboardInput { scancode, state, virtual_keycode: keycode, .. } = input;

    match state {
      ElementState::Pressed => {
        if let Some(keycode) = keycode {
          self.keys_held_keycode.set(keycode as usize, true);
          self.key_actions.push(KeyAction::Pressed(keycode, scancode));
        };

        if (scancode as usize) < SCANCODES {
          self.keys_held_scancode.set(scancode as usize, true);
        };

        (true, keycode, scancode)
      },
      ElementState::Released => {
        if let Some(keycode) = keycode {
          self.keys_held_keycode.set(keycode as usize, false);
          self.key_actions.push(KeyAction::Released(keycode, scancode));
        };

        if (scancode as usize) < SCANCODES {
          self.keys_held_scancode.set(scancode as usize, false);
        };

        (false, keycode, scancode)
      }
    }
  }

  fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) -> bool {
    match state {
      ElementState::Pressed => {
        self.set_button_value(button, true);
        self.mouse_actions.push(MouseAction::Pressed(button));
        true
      },
      ElementState::Released => {
        self.set_button_value(button, false);
        self.mouse_actions.push(MouseAction::Released(button));
        false
      }
    }
  }

  fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, scale_factor: f64) -> (f32, f32) {
    let delta = match delta {
      MouseScrollDelta::LineDelta(x, y) => (x, y),
      MouseScrollDelta::PixelDelta(pos) => {
        let LogicalPosition { x, y } = pos.to_logical::<f32>(scale_factor);
        (x, y)
      }
    };

    self.scroll_rel.0 += delta.0;
    self.scroll_rel.1 += delta.1;
    delta
  }
}

impl Default for InputState {
  #[inline]
  fn default() -> Self {
    InputState {
      cursor_pos: None,
      cursor_pos_prev: None,
      mouse_actions: Vec::new(),
      mouse_left_held: false,
      mouse_right_held: false,
      mouse_middle_held: false,
      key_actions: Vec::new(),
      keys_held_keycode: BitArray::default(),
      keys_held_scancode: BitArray::default(),
      modifiers_state: ModifiersState::default(),
      scroll_rel: (0.0, 0.0),
      text: String::new()
    }
  }
}

impl Index<VirtualKeyCode> for InputState {
  type Output = bool;

  #[inline]
  fn index(&self, keycode: VirtualKeyCode) -> &bool {
    &self.keys_held_keycode[keycode as usize]
  }
}

impl Index<ScanCode> for InputState {
  type Output = bool;

  #[inline]
  fn index(&self, scancode: ScanCode) -> &bool {
    if (scancode as usize) < SCANCODES {
      &self.keys_held_scancode[scancode as usize]
    } else {
      &false
    }
  }
}

impl Index<MouseButton> for InputState {
  type Output = bool;

  #[inline]
  fn index(&self, button: MouseButton) -> &bool {
    match button {
      MouseButton::Left => &self.mouse_left_held,
      MouseButton::Right => &self.mouse_right_held,
      MouseButton::Middle => &self.mouse_middle_held,
      MouseButton::Other(_) => &false
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
  Pressed(VirtualKeyCode, ScanCode),
  Released(VirtualKeyCode, ScanCode)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
  Pressed(MouseButton),
  Released(MouseButton)
}



#[derive(Debug, Clone)]
pub struct State {
  input_state: InputState,
  dropped_file: Option<PathBuf>,
  scale_factor: f64,
  window_size: PhysicalSize<u32>,
  window: Rc<Window>
}

impl State {
  fn from_window(window: Rc<Window>) -> Self {
    State {
      input_state: InputState::default(),
      dropped_file: None,
      scale_factor: window.scale_factor(),
      window_size: window.inner_size().into(),
      window
    }
  }

  fn reset(&mut self) {
    self.input_state.reset();
    self.dropped_file = None;
  }

  #[inline]
  pub fn input(&self) -> &InputState {
    &self.input_state
  }

  #[inline]
  pub fn dropped_file(&self) -> Option<&Path> {
    self.dropped_file.as_deref()
  }

  #[inline]
  pub fn scale_factor(&self) -> f64 {
    self.scale_factor
  }

  #[inline]
  pub fn window_size(&self) -> (u32, u32) {
    self.window_size.into()
  }

  #[inline]
  pub fn window(&self) -> &Window {
    &self.window
  }

  #[inline]
  pub fn window_ptr(&self) -> Rc<Window> {
    Rc::clone(&self.window)
  }

  fn handle_event<T, H: EventHandler<T>>(&mut self, handler: &mut H, event: Event<T>, cf: &mut ControlFlow) {
    match event {
      Event::NewEvents(_) => self.reset(),
      Event::WindowEvent { event, window_id } if self.window.id() == window_id => match event {
        WindowEvent::CloseRequested => {
          if handler.close(self) {
            *cf = ControlFlow::Exit;
          };
        },
        WindowEvent::Destroyed => {
          *cf = ControlFlow::Exit;
        },
        WindowEvent::Focused(false) => {
          self.input_state = InputState::default();
          handler.focused(self, false);
        },
        WindowEvent::Focused(true) => {
          handler.focused(self, true);
        },
        WindowEvent::DroppedFile(path) => {
          self.dropped_file = Some(path.clone());
          handler.file_dropped(self, path);
        },
        WindowEvent::Resized(new_inner_size) => {
          self.window_size = new_inner_size;
          handler.resized(self, self.window_size.into(), self.scale_factor);
        },
        WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
          self.window_size = *new_inner_size;
          self.scale_factor = scale_factor;
          handler.resized(self, self.window_size.into(), self.scale_factor);
        },
        WindowEvent::KeyboardInput { input, .. } => {
          let (cond, keycode, scancode) = self.input_state.handle_keyboard_input(input);
          handler.keyboard_input(self, cond, keycode, scancode);
        },
        WindowEvent::ReceivedCharacter(ch) => {
          self.input_state.text.push(ch);
          handler.text_input(self, ch);
        },
        WindowEvent::CursorMoved { position, .. } => {
          let position = (position.x as f32, position.y as f32);
          self.input_state.cursor_pos = Some(position);
          handler.cursor_moved(self, position);
        },
        WindowEvent::MouseInput { state, button, .. } => {
          let cond = self.input_state.handle_mouse_input(state, button);
          handler.mouse_input(self, cond, button);
        },
        WindowEvent::MouseWheel { delta, .. } => {
          let delta = self.input_state.handle_mouse_wheel(delta, self.scale_factor);
          handler.mouse_scroll(self, delta);
        },
        WindowEvent::ModifiersChanged(modifiers_state) => {
          self.input_state.modifiers_state = modifiers_state;
        },
        _ => ()
      },
      Event::RedrawRequested(window_id) if self.window.id() == window_id => {
        handler.render(self);
      },
      Event::MainEventsCleared => {
        handler.update(self);
        self.window.request_redraw();
      },
      Event::UserEvent(t) => handler.user_event(self, t),
      _ => ()
    }
  }
}

pub fn run<T, H: EventHandler<T>>(
  event_loop: EventLoop<T>,
  window: impl Into<Rc<Window>>,
  handler: H
) -> ! {
  let mut state = State::from_window(window.into());
  let mut handler = Some(handler);
  event_loop.run(move |event, _, cf| {
    if let Event::LoopDestroyed = event {
      handler.take().unwrap().destroy();
    } else {
      let handler = handler.as_mut().unwrap();
      state.handle_event(handler, event, cf);
      if handler.should_exit(&state) {
        *cf = ControlFlow::Exit;
      };
    };
  });
}
