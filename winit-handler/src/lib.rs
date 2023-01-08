//! A trait-based event handler for [`winit`][winit].
#![warn(missing_copy_implementations, missing_debug_implementations)]

pub mod extra;

use bitvec::array::BitArray;
#[cfg(feature = "glutin")]
use glutin::{PossiblyCurrent, WindowedContext};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::*;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use std::ops::Index;
use std::path::{Path, PathBuf};
use std::rc::Rc;



/// Handles events emitted by a winit event loop.
#[allow(unused_variables)]
pub trait EventHandler<T = ()>: Sized + 'static {
  /// Called upon [`Event::RedrawRequested`][winit::event::Event::RedrawRequested].
  fn render(&mut self, window_state: &WindowState) {}
  /// Called upon [`Event::MainEventsCleared`][winit::event::Event::MainEventsCleared].
  fn update(&mut self, window_state: &WindowState) {}
  /// Called when an event is sent from [`EventLoopProxy::send_event`][winit::event_loop::EventLoopProxy::send_event].
  fn user_event(&mut self, window_state: &WindowState, event: T) {}
  /// Called when an event from the keyboard has been received.
  fn keyboard_input(&mut self, window_state: &WindowState, state: KeyState, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {}
  /// Called when the window receives a unicode character.
  fn text_input(&mut self, window_state: &WindowState, ch: char) {}
  /// Called when the cursor has moved on the window.
  fn cursor_moved(&mut self, window_state: &WindowState, pos: (f32, f32)) {}
  /// Called when a mouse button press has been received.
  fn mouse_input(&mut self, window_state: &WindowState, state: ElementState, button: MouseButton) {}
  /// Called when a mouse wheel or touchpad scroll occurs.
  fn mouse_scroll(&mut self, window_state: &WindowState, delta: (f32, f32)) {}
  /// Called when the application loses or gains focus.
  fn focused(&mut self, window_state: &WindowState, state: bool) {}
  /// Called when a file is dropped in the application window.
  fn file_dropped(&mut self, window_state: &WindowState, path: PathBuf) {}
  /// Called when either the window has been resized or the scale factor has changed.
  fn resized(&mut self, window_state: &WindowState, window_size: (u32, u32), scale_factor: f64) {}
  /// Called when the user attempts to close the application.
  /// A return value of `true` closes the application, while `false` cancels closing it.
  /// Defaults to an 'always `true`' implementation.
  fn close(&mut self, window_state: &WindowState) -> bool { true }
  /// Instructs the event dispatcher whether the handler wants the application to exit.
  /// Defaults to an 'always `false`' implementation.
  fn should_exit(&self, window_state: &WindowState) -> bool { false }
  /// Called once the event loop has been destroyed and will no longer dispatch any more events.
  /// This is different from the `close` function in that the handler has no choice over the application state.
  fn destroy(self) {}
}

// More than 162 bits, enough space to store the state of every key
const KEYCODE_BITS: usize = 192;
const SCANCODE_BITS: usize = 512;
const SCANCODE_MAX: u32 = 512;

#[inline]
fn element_state_to_bool(state: ElementState) -> bool {
  match state {
    ElementState::Pressed => true,
    ElementState::Released => false
  }
}

macro_rules! find_key_action {
  ($window_state:expr, $keycode:ident, $KeyState:pat) => {
    $window_state.key_actions.iter()
      .find(|&&action| match action {
        KeyAction {
          keycode: candidate,
          state: $KeyState,
          ..
        } => candidate == $keycode,
        _ => false
      })
      .is_some()
  };
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputState {
  cursor_pos: Option<(f32, f32)>,
  cursor_pos_prev: Option<(f32, f32)>,
  mouse_actions: Vec<MouseAction>,
  mouse_left_held: bool,
  mouse_right_held: bool,
  mouse_middle_held: bool,
  has_not_moved: bool,
  key_actions: Vec<KeyAction>,
  keys_held_keycode: BitArray<[u32; KEYCODE_BITS / 32]>,
  keys_held_scancode: BitArray<[u32; SCANCODE_BITS / 32]>,
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

  /// Returns a list of mouse button actions performed during the current frame.
  #[inline]
  pub fn mouse_actions(&self) -> &[MouseAction] {
    &self.mouse_actions
  }

  /// Checks whether or not the given mouse button is currently pressed.
  #[inline]
  pub fn is_button_held(&self, button: MouseButton) -> bool {
    self[button]
  }

  /// Returns a list of key actions performed during the current frame.
  #[inline]
  pub fn key_actions(&self) -> &[KeyAction] {
    &self.key_actions
  }

  /// Checks whether or not the given key is currently pressed.
  #[inline]
  pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
    self[keycode]
  }

  /// Checks whether or not the given key is currently pressed, takes a scancode.
  #[inline]
  pub fn is_key_held_scancode(&self, scancode: ScanCode) -> bool {
    self[scancode]
  }

  /// Checks whether or not the given key was pressed during the current frame.
  pub fn was_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
    find_key_action!(self, keycode, KeyState::Pressed)
  }

  /// Checks whether or not the given key was repeated during the current frame.
  pub fn was_key_repeating(&self, keycode: VirtualKeyCode) -> bool {
    find_key_action!(self, keycode, KeyState::Repeating)
  }

  /// Checks whether or not the given key was released during the current frame.
  pub fn was_key_released(&self, keycode: VirtualKeyCode) -> bool {
    find_key_action!(self, keycode, KeyState::Released)
  }

  /// Whether the mouse moved during the current frame.
  pub fn was_moving(&self) -> bool {
    self.cursor_pos != self.cursor_pos_prev
  }

  /// Whether the mouse has remained stationary since the mouse was pressed.
  /// This can be useful for filtering between user intention in the event you
  /// want to have different functionality between click + drag and click.
  pub fn has_not_moved(&self) -> bool {
    self.has_not_moved && !self.was_moving()
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
    self.mouse_actions = Vec::with_capacity(4);
    self.key_actions = Vec::with_capacity(4);
    self.has_not_moved = false;
    self.scroll_rel = (0.0, 0.0);
    self.text.clear();
  }

  fn handle_keyboard_input(&mut self, input: KeyboardInput) -> (KeyState, Option<VirtualKeyCode>, ScanCode) {
    let KeyboardInput { scancode, state, virtual_keycode: keycode, .. } = input;

    let cond = element_state_to_bool(state);
    let state = match state {
      ElementState::Pressed if self[scancode] => KeyState::Repeating,
      ElementState::Pressed => KeyState::Pressed,
      ElementState::Released => KeyState::Released
    };

    if let Some(keycode) = keycode {
      self.keys_held_keycode.set(keycode as usize, cond);
      self.key_actions.push(KeyAction { keycode, scancode, state });
    };

    if scancode < SCANCODE_MAX {
      self.keys_held_scancode.set(scancode as usize, cond);
    };

    (state, keycode, scancode)
  }

  fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
    let cond = element_state_to_bool(state);
    self.set_button_value(button, cond);
    self.mouse_actions.push(MouseAction { button, state });
    self.has_not_moved = cond;
  }

  fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta, scale_factor: f64) -> (f32, f32) {
    let delta: (f32, f32) = match delta {
      MouseScrollDelta::LineDelta(x, y) => (x, y),
      MouseScrollDelta::PixelDelta(pos) => pos.to_logical::<f32>(scale_factor).into()
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
      has_not_moved: false,
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
    if scancode < SCANCODE_MAX {
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyAction {
  pub keycode: VirtualKeyCode,
  pub scancode: ScanCode,
  pub state: KeyState
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MouseAction {
  pub button: MouseButton,
  pub state: ElementState
}

/// Equivalent to [`ElementState`][winit::event::ElementState] but with an additional `Repeating` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KeyState {
  Pressed,
  Repeating,
  Released
}

impl From<KeyState> for ElementState {
  #[inline]
  fn from(state: KeyState) -> Self {
    match state {
      KeyState::Pressed | KeyState::Repeating => ElementState::Pressed,
      KeyState::Released => ElementState::Released
    }
  }
}

#[derive(Debug, Clone)]
pub enum WindowStateContext {
  #[cfg(feature = "glutin")]
  Glutin(Rc<WindowedContext<PossiblyCurrent>>),
  Winit(Rc<Window>)
}

impl WindowStateContext {
  pub fn window(&self) -> &Window {
    match self {
      #[cfg(feature = "glutin")]
      Self::Glutin(windowed_context) => windowed_context.window(),
      Self::Winit(window) => window
    }
  }
}

impl From<Window> for WindowStateContext {
  fn from(window: Window) -> Self {
    WindowStateContext::Winit(Rc::new(window))
  }
}

impl From<Rc<Window>> for WindowStateContext {
  fn from(window: Rc<Window>) -> Self {
    WindowStateContext::Winit(window)
  }
}

#[cfg(feature = "glutin")]
impl From<WindowedContext<PossiblyCurrent>> for WindowStateContext {
  fn from(window_context: WindowedContext<PossiblyCurrent>) -> Self {
    WindowStateContext::Glutin(Rc::new(window_context))
  }
}

#[cfg(feature = "glutin")]
impl From<Rc<WindowedContext<PossiblyCurrent>>> for WindowStateContext {
  fn from(window_context: Rc<WindowedContext<PossiblyCurrent>>) -> Self {
    WindowStateContext::Glutin(window_context)
  }
}

#[derive(Debug, Clone)]
pub struct WindowState {
  input_state: InputState,
  dropped_file: Option<PathBuf>,
  scale_factor: f64,
  window_size: PhysicalSize<u32>,
  context: WindowStateContext
}

impl WindowState {
  fn new(context: WindowStateContext) -> Self {
    let window = context.window();
    WindowState {
      input_state: InputState::default(),
      dropped_file: None,
      scale_factor: window.scale_factor(),
      window_size: window.inner_size().into(),
      context
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
    self.context.window()
  }

  #[inline]
  pub fn context(&self) -> &WindowStateContext {
    &self.context
  }

  #[cfg(feature = "glutin")]
  pub fn context_glutin(&self) -> &WindowedContext<PossiblyCurrent> {
    match &self.context {
      WindowStateContext::Glutin(windowed_context) => &windowed_context,
      WindowStateContext::Winit(..) => panic!("window state context does not contain a glutin windowed context")
    }
  }

  /// Only returns `Some` when the given cursor position is within frame.
  fn clip_cursor_pos(&self, position: PhysicalPosition<f64>) -> Option<(f32, f32)> {
    let PhysicalSize { width, height } = self.window_size.cast::<f64>();
    if position.x >= 0.0 && position.x <= width && position.y >= 0.0 && position.y <= height {
      Some((position.x as f32, position.y as f32))
    } else {
      None
    }
  }

  /// Dispatches an event to a handler given an [`Event`][winit::event::Event].
  fn handle_event<T, H: EventHandler<T>>(&mut self, handler: &mut H, event: Event<T>, cf: &mut ControlFlow) {
    match event {
      Event::NewEvents(_) => self.reset(),
      Event::WindowEvent { event, window_id } if self.window().id() == window_id => match event {
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
          let (state, keycode, scancode) = self.input_state.handle_keyboard_input(input);
          handler.keyboard_input(self, state, keycode, scancode);
        },
        WindowEvent::ReceivedCharacter(ch) => {
          self.input_state.text.push(ch);
          handler.text_input(self, ch);
        },
        WindowEvent::CursorMoved { position, .. } => {
          if let Some(position) = self.clip_cursor_pos(position) {
            self.input_state.cursor_pos = Some(position);
            self.input_state.has_not_moved = false;
            handler.cursor_moved(self, position);
          };
        },
        WindowEvent::MouseInput { state, button, .. } => {
          self.input_state.handle_mouse_input(state, button);
          handler.mouse_input(self, state, button);
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
      Event::RedrawRequested(window_id) if self.window().id() == window_id => {
        handler.render(self);
      },
      Event::MainEventsCleared => {
        handler.update(self);
        self.window().request_redraw();
      },
      Event::UserEvent(t) => handler.user_event(self, t),
      _ => ()
    }
  }
}

pub fn run<T, W, H>(event_loop: EventLoop<T>, source: W, handler: H) -> !
where W: Into<WindowStateContext>, H: EventHandler<T> {
  let mut window_state = WindowState::new(source.into());
  let mut handler = Some(handler);
  event_loop.run(move |event, _, cf| {
    if let Event::LoopDestroyed = event {
      handler.take().unwrap().destroy();
    } else {
      let handler = handler.as_mut().unwrap();
      window_state.handle_event(handler, event, cf);
      if handler.should_exit(&window_state) {
        *cf = ControlFlow::Exit;
      };
    };
  });
}
