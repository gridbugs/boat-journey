use crate::world::{
    realtime_periodic::{core::ScheduledRealtimePeriodicState, movement},
    ExternalEvent, World,
};
use crate::Message;
use direction::Direction;
use entity_table::Entity;
use grid_2d::Coord;
use line_2d::LineSegment;
use rand::Rng;
use std::time::Duration;

pub mod spec {
    pub use grid_2d::Coord;
    use serde::{Deserialize, Serialize};
    pub use std::time::Duration;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct ParticleEmitter {
        pub duration: Duration,
        pub num_particles_per_frame: u32,
        pub min_step: Duration,
        pub max_step: Duration,
        pub fade_duration: Duration,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Mechanics {
        pub range: u32,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Explosion {
        pub mechanics: Mechanics,
        pub particle_emitter: ParticleEmitter,
    }
}

fn apply_indirect_hit<R: Rng>(
    world: &mut World,
    character_entity: Entity,
    explosion_to_character: LineSegment,
    rng: &mut R,
    external_events: &mut Vec<ExternalEvent>,
    message_log: &mut Vec<Message>,
) {
    let push_back = 2;
    let damage = 2;
    world.components.realtime.insert(character_entity, ());
    world.realtime_components.movement.insert(
        character_entity,
        ScheduledRealtimePeriodicState {
            state: movement::spec::Movement {
                path: explosion_to_character.delta(),
                repeat: movement::spec::Repeat::Steps(push_back as usize),
                cardinal_step_duration: Duration::from_millis(100),
            }
            .build(),
            until_next_event: Duration::from_millis(0),
        },
    );
    world.damage_character(character_entity, damage, rng, external_events, message_log);
}

fn apply_direct_hit<R: Rng>(
    world: &mut World,
    explosion_coord: Coord,
    character_entity: Entity,
    rng: &mut R,
    external_events: &mut Vec<ExternalEvent>,
    message_log: &mut Vec<Message>,
) {
    let mut solid_neighbour_vector = Coord::new(0, 0);
    for direction in Direction::all() {
        let neighbour_coord = explosion_coord + direction.coord();
        if let Some(spatial_cell) = world.spatial_table.layers_at(neighbour_coord) {
            if spatial_cell.feature.is_some() || spatial_cell.character.is_some() {
                solid_neighbour_vector += direction.coord();
            }
        }
    }
    let push_back = 2;
    let damage = 2;
    if solid_neighbour_vector.is_zero() {
        log::warn!("Direct hit with no solid neighbours shouldn't be possible.");
    } else {
        let travel_vector = -solid_neighbour_vector;
        world.components.realtime.insert(character_entity, ());
        world.realtime_components.movement.insert(
            character_entity,
            ScheduledRealtimePeriodicState {
                state: movement::spec::Movement {
                    path: travel_vector,
                    repeat: movement::spec::Repeat::Steps(push_back as usize),
                    cardinal_step_duration: Duration::from_millis(100),
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
    }
    world.damage_character(character_entity, damage, rng, external_events, message_log);
}

fn is_in_explosion_range(
    explosion_coord: Coord,
    mechanics: &spec::Mechanics,
    coord: Coord,
) -> bool {
    explosion_coord.distance2(coord) <= mechanics.range.pow(2)
}

fn apply_mechanics<R: Rng>(
    world: &mut World,
    explosion_coord: Coord,
    mechanics: &spec::Mechanics,
    rng: &mut R,
    external_events: &mut Vec<ExternalEvent>,
    message_log: &mut Vec<Message>,
) {
    for character_entity in world.components.character.entities().collect::<Vec<_>>() {
        if let Some(character_coord) = world.spatial_table.coord_of(character_entity) {
            if character_coord == explosion_coord {
                apply_direct_hit(
                    world,
                    explosion_coord,
                    character_entity,
                    rng,
                    external_events,
                    message_log,
                );
            } else {
                if !is_in_explosion_range(explosion_coord, mechanics, character_coord) {
                    continue;
                }
                let explosion_to_character = LineSegment::new(explosion_coord, character_coord);
                apply_indirect_hit(
                    world,
                    character_entity,
                    explosion_to_character,
                    rng,
                    external_events,
                    message_log,
                );
            }
        }
    }
    for destructible_entity in world.components.destructible.entities().collect::<Vec<_>>() {
        if let Some(coord) = world.spatial_table.coord_of(destructible_entity) {
            if is_in_explosion_range(explosion_coord, mechanics, coord) {
                world.components.to_remove.insert(destructible_entity, ());
            }
        }
    }
}

pub fn explode<R: Rng>(
    world: &mut World,
    coord: Coord,
    explosion: spec::Explosion,
    external_events: &mut Vec<ExternalEvent>,
    message_log: &mut Vec<Message>,
    rng: &mut R,
) {
    world.spawn_explosion_emitter(coord, &explosion.particle_emitter);
    apply_mechanics(
        world,
        coord,
        &explosion.mechanics,
        rng,
        external_events,
        message_log,
    );
    external_events.push(ExternalEvent::Explosion(coord));
}
