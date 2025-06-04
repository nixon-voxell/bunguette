use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::player::{PlayerState, PlayerType, QueryPlayers};
use crate::util::PropagateComponentAppExt;

pub(super) struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(
                Update,
                hookup_target_action
                    .run_if(in_state(PlayerState::Possessed)),
            )
            .add_observer(setup_gamepad_index).propagate_component::<TargetAction, Children>();
    }
}

/// Add [`TargetAction`] to [`PlayerType`] that has [`RequireAction`].
fn hookup_target_action(
    mut commands: Commands,
    q_require_actions: Query<
        (&PlayerType, Entity),
        (With<RequireAction>, Without<TargetAction>),
    >,
    q_actions: QueryPlayers<Entity, With<InputMap<PlayerAction>>>,
) {
    // Nothing to do!
    if q_require_actions.is_empty() {
        return;
    }

    for (player_type, entity) in q_require_actions.iter() {
        let Ok(action_entity) = q_actions.get(*player_type) else {
            continue;
        };

        commands.entity(entity).insert(TargetAction(action_entity));
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
    // Inventory actions.
    CycleNext,
    CyclePrev,
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
            .with(Self::CycleNext, GamepadButton::DPadRight)
            .with(Self::CyclePrev, GamepadButton::DPadLeft)
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
            .with(Self::CycleNext, KeyCode::ArrowRight)
            .with(Self::CyclePrev, KeyCode::ArrowLeft)
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

#[derive(Component, Deref, Clone, Copy)]
pub struct TargetAction(Entity);

impl TargetAction {
    pub fn get(&self) -> Entity {
        self.0
    }
}
