use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::player::{
    PlayerState, PlayerType, QueryPlayerA, QueryPlayerB,
};
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
            .add_observer(setup_gamepad_index).propagate_component::<TargetAction>();
    }
}

/// Add [`TargetAction`] to [`PlayerType`] that has [`RequireAction`].
fn hookup_target_action(
    mut commands: Commands,
    q_require_actions: Query<
        (&PlayerType, Entity),
        (With<RequireAction>, Without<TargetAction>),
    >,
    q_action_a: QueryPlayerA<Entity, With<InputMap<PlayerAction>>>,
    q_action_b: QueryPlayerB<Entity, With<InputMap<PlayerAction>>>,
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
    // Inventory actions.
    Pickup,
    Drop,
    Consume,
    CycleNext,
    CyclePrev,
    MoveItem,
    ToggleInventory,
    #[actionlike(Button)]
    InventoryModifier,
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
            .with(Self::Pickup, GamepadButton::West)
            .with(Self::Drop, GamepadButton::North)
            .with(Self::Consume, GamepadButton::LeftTrigger2)
            .with(Self::CycleNext, GamepadButton::East)
            .with(Self::CyclePrev, GamepadButton::West)
            .with(Self::MoveItem, GamepadButton::RightTrigger)
            .with(Self::ToggleInventory, GamepadButton::Select)
            .with(Self::InventoryModifier, GamepadButton::LeftTrigger)
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
            .with(Self::Pickup, KeyCode::KeyE)
            .with(Self::Drop, KeyCode::KeyQ)
            .with(Self::Consume, KeyCode::KeyC)
            .with(Self::CycleNext, KeyCode::ArrowRight)
            .with(Self::CyclePrev, KeyCode::ArrowLeft)
            .with(Self::MoveItem, KeyCode::KeyM)
            .with(Self::ToggleInventory, KeyCode::Tab)
            .with(Self::InventoryModifier, KeyCode::AltLeft)
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
