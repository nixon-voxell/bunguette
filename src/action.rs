use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_gamepad_index);
    }
}

/// Create a [`InputMap`] for every connected gamepads.
fn setup_gamepad_index(
    trigger: Trigger<OnAdd, Gamepad>,
    mut commands: Commands,
    q_gamepad_indices: Query<(), With<GamepadIndex>>,
    mut count: Local<u8>,
) {
    let entity = trigger.target();

    if q_gamepad_indices.contains(entity) == false {
        commands.entity(entity).insert(GamepadIndex(*count));
        *count += 1;
    }

    info!("Setup `GamepadIndex` input map for gamepad {entity}.");
}

#[derive(
    Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug,
)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    Aim,
    Jump,
    Interact,
    Attack,
}

impl PlayerAction {
    /// Create a new [`InputMap`] for gamepads.
    pub fn new_gamepad() -> InputMap<Self> {
        InputMap::default()
            // Gamepad input bindings.
            .with_dual_axis(Self::Move, GamepadStick::LEFT)
            .with_dual_axis(Self::Aim, GamepadStick::RIGHT)
            .with(Self::Jump, GamepadButton::South)
            .with(Self::Interact, GamepadButton::West)
            .with(Self::Attack, GamepadButton::RightTrigger2)
    }

    /// Create a new [`InputMap`] for keyboard and mouse.
    pub fn new_kbm() -> InputMap<Self> {
        InputMap::default()
            // KbM input bindings.
            .with_dual_axis(Self::Move, VirtualDPad::wasd())
            .with_dual_axis(Self::Aim, MouseMove::default())
            .with(Self::Jump, KeyCode::Space)
            .with(Self::Interact, KeyCode::KeyE)
            .with(Self::Attack, MouseButton::Left)
    }
}

#[derive(Component)]
pub struct GamepadIndex(u8);

impl GamepadIndex {
    pub fn get(&self) -> u8 {
        self.0
    }
}
