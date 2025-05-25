use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::player::{PlayerA, PlayerB, PlayerState, PlayerType};

pub(super) struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(
                Update,
                hookup_target_action
                    .run_if(in_state(PlayerState::Possessed)),
            )
            .add_observer(setup_gamepad_index);
    }
}

/// Add [`TargetAction`] to [`PlayerType`] that has [`RequireAction`].
fn hookup_target_action(
    mut commands: Commands,
    q_require_actions: Query<
        (&PlayerType, Entity),
        (With<RequireAction>, Without<TargetAction>),
    >,
    q_action_a: Query<
        Entity,
        (With<InputMap<PlayerAction>>, With<PlayerA>),
    >,
    q_action_b: Query<
        Entity,
        (With<InputMap<PlayerAction>>, With<PlayerB>),
    >,
) {
    // Nothing to do!
    if q_require_actions.is_empty() {
        return;
    }

    let Ok(action_a) = q_action_a.single() else {
        return;
    };
    let Ok(action_b) = q_action_b.single() else {
        return;
    };

    for (player_type, entity) in q_require_actions.iter() {
        let mut cmd = commands.entity(entity);
        match player_type {
            PlayerType::A => cmd.insert(TargetAction(action_a)),
            PlayerType::B => cmd.insert(TargetAction(action_b)),
        };
    }
}

/// Create a [`GamepadIndex`] for every connected gamepads.
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
            .with_dual_axis(
                Self::Move,
                GamepadStick::LEFT.with_deadzone_symmetric(0.1),
            )
            .with_dual_axis(
                Self::Aim,
                GamepadStick::RIGHT.with_deadzone_symmetric(0.1),
            )
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

/// Tag component for entities that requires action.
#[derive(Component, Default)]
pub struct RequireAction;

#[derive(Component, Deref)]
pub struct TargetAction(Entity);

impl TargetAction {
    pub fn get(&self) -> Entity {
        self.0
    }
}
