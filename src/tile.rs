use bevy::prelude::*;
use pathfinding::prelude::*;

pub(super) struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>()
            .add_observer(setup_tile)
            .add_observer(on_placed)
            .add_observer(on_freed);

        app.register_type::<Tile>();
    }
}

/// The half size of the map, this should be
/// greater or equal to the half size of the map.
const HALF_MAP_SIZE: usize = 20;

/// Setup tile inside the [`TileMap`].
fn setup_tile(
    trigger: Trigger<OnAdd, Tile>,
    q_transforms: Query<&Transform>,
    mut tile_map: ResMut<TileMap>,
) -> Result {
    let entity = trigger.target();

    let transform = q_transforms.get(entity)?;

    *tile_map.get_mut(transform).ok_or(format!(
        "Unable to get tile for {entity}, {transform:?}"
    ))? = Some(TileMeta::new(entity));

    Ok(())
}

fn on_placed(
    trigger: Trigger<OnAdd, PlacedBy>,
    q_transforms: Query<&Transform>,
    mut tile_map: ResMut<TileMap>,
) -> Result {
    let entity = trigger.target();

    let transform = q_transforms.get(entity)?;

    if let Some(tile) = tile_map
        .get_mut(transform)
        .ok_or(format!(
            "Unable to get tile for {entity}, {transform:?}"
        ))?
        .as_mut()
    {
        tile.occupied = true;
    }

    Ok(())
}

fn on_freed(
    trigger: Trigger<OnRemove, PlacedBy>,
    q_transforms: Query<&Transform>,
    mut tile_map: ResMut<TileMap>,
) -> Result {
    let entity = trigger.target();

    let transform = q_transforms.get(entity)?;

    if let Some(tile) = tile_map
        .get_mut(transform)
        .ok_or(format!(
            "Unable to get tile for {entity}, {transform:?}"
        ))?
        .as_mut()
    {
        tile.occupied = false;
    }

    Ok(())
}

#[derive(Resource, Deref)]
pub struct TileMap(Vec<Option<TileMeta>>);

impl TileMap {
    pub fn validate_ivec2(coordinate: &IVec2) -> bool {
        const MAP_SIZE: i32 = HALF_MAP_SIZE as i32 * 2;

        if coordinate.x < 0 || coordinate.y < 0 {
            warn!("Attempt to obtain negative coordinate!");
            return false;
        } else if coordinate.x >= MAP_SIZE || coordinate.y >= MAP_SIZE
        {
            warn!("Attempt to obtain out of bounds coordinate!");
            return false;
        }

        true
    }

    pub fn transform_to_uvec2(
        transform: &Transform,
    ) -> Option<UVec2> {
        let coordinate = transform.translation.xz().round().as_ivec2()
                // Prevent going negative.
                + HALF_MAP_SIZE as i32;

        if TileMap::validate_ivec2(&coordinate) == false {
            return None;
        }

        Some(coordinate.as_uvec2())
    }

    pub fn uvec2_to_tile_idx(coordinate: &UVec2) -> usize {
        let map_size = HALF_MAP_SIZE as u32 * 2;
        (coordinate.x + coordinate.y * map_size) as usize
    }

    pub fn transform_to_tile_idx(
        transform: &Transform,
    ) -> Option<usize> {
        TileMap::transform_to_uvec2(transform)
            .map(|coord| TileMap::uvec2_to_tile_idx(&coord))
    }

    fn get_mut(
        &mut self,
        transform: &Transform,
    ) -> Option<&mut Option<TileMeta>> {
        TileMap::transform_to_tile_idx(transform)
            .and_then(|index| self.0.get_mut(index))
    }

    pub fn pathfind_to(
        &self,
        start_transform: &Transform,
        end_transform: &Transform,
    ) -> Option<Vec<IVec2>> {
        let start =
            TileMap::transform_to_uvec2(start_transform)?.as_ivec2();
        let end =
            TileMap::transform_to_uvec2(end_transform)?.as_ivec2();

        Some(
            astar(
                &start,
                |&IVec2 { x, y }| {
                    [
                        // Bottom row.
                        IVec2::new(x - 1, y - 1),
                        IVec2::new(x, y - 1),
                        IVec2::new(x + 1, y - 1),
                        // Center row.
                        IVec2::new(x - 1, y),
                        IVec2::new(x + 1, y),
                        // Top row.
                        IVec2::new(x - 1, y + 1),
                        IVec2::new(x, y + 1),
                        IVec2::new(x + 1, y + 1),
                    ]
                    .into_iter()
                    .filter(|p| {
                        // Must be a valid coordinate
                        if TileMap::validate_ivec2(p) {
                            let tile_meta = self
                                [TileMap::uvec2_to_tile_idx(
                                    &p.as_uvec2(),
                                )];

                            let Some(tile_meta) = tile_meta else {
                                return false;
                            };

                            // Must not be occupied.
                            return tile_meta.occupied == false;
                        }
                        false
                    })
                    .map(|p| (p, 1))
                },
                |potential| potential.distance_squared(end),
                |coord| *coord == end,
            )?
            .0,
        )
    }
}

impl Default for TileMap {
    fn default() -> Self {
        let map_size = HALF_MAP_SIZE * 2;
        Self(vec![None; map_size * map_size])
    }
}

#[derive(Clone, Copy)]
pub struct TileMeta {
    target: Entity,
    occupied: bool,
}

impl TileMeta {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            occupied: false,
        }
    }
}

/// Tag component for tiles that can be placed on.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct Tile;

/// Attached to a [`PlacementTile`] when it's being placed on.
#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = PlacedOn)]
pub struct PlacedBy(Vec<Entity>);

/// Attached to the item that is being placed on a [`PlacementTile`].
#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = PlacedBy)]
pub struct PlacedOn(pub Entity);
