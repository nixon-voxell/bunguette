use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(
    Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug,
)]
pub enum PlayerAction {
    Aim,
    Move,
    Interact,
    Attack,
}

impl PlayerAction {
    /// Define the default bindings to the input
    pub fn _input_map() -> InputMap<Self> {
        InputMap::default()
            // KbM input bindings
            .with_dual_axis(Self::Aim, MouseMove::default())
            .with_dual_axis(Self::Move, VirtualDPad::wasd())
            .with(Self::Interact, KeyCode::KeyE)
            .with(Self::Attack, MouseButton::Left)
            // Gamepad input bindings
            .with_dual_axis(Self::Move, GamepadStick::RIGHT)
            .with_dual_axis(Self::Move, GamepadStick::LEFT)
            .with(Self::Interact, GamepadButton::West)
            .with(Self::Attack, GamepadButton::RightTrigger2)
    }
}
