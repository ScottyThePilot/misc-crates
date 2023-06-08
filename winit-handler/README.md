# Winit Handler

Introduces an event-handler style system for winit event loops as well as input state tracking.

`winit-handler` depends on `winit@>=v0.28.6`.

## Behavior

`winit-handler`'s `InputState` tracks key press and release pairs, but in order to avoid situations
where an input pair is broken on lost focus (user releases button/key after unfocusing window), `winit-handler`
completely resets the current input state whenever focus is lost.

If a key down event is recieved while `InputState` already tracks that key as being pressed,
it is reported as a key 'repeat' event.
