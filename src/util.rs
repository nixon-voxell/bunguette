use bevy::prelude::*;

pub trait PropagateComponentAppExt {
    fn propagate_component<C>(&mut self) -> &mut Self
    where
        C: Component + Clone;
}

impl PropagateComponentAppExt for App {
    fn propagate_component<C>(&mut self) -> &mut Self
    where
        C: Component + Clone,
    {
        self.add_systems(
            PostUpdate,
            propagate_component::<C>.in_set(PropagateComponentSet),
        )
    }
}

/// Propagate component to the children hierarchy.
pub fn propagate_component<C>(
    mut commands: Commands,
    q_children: Query<
        (&C, &Children),
        // Just added or the children changes.
        Or<(Added<C>, Changed<Children>)>,
    >,
) where
    C: Component + Clone,
{
    for (component, children) in q_children.iter() {
        for entity in children.iter() {
            if let Ok(mut cmd) = commands.get_entity(entity) {
                cmd.insert(component.clone());
            }
        }
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PropagateComponentSet;
