use crate::{
    visibility::Light,
    world::{
        data::{
            Armour, CollidesWith, Disposition, DoorState, EntityData, HitPoints, Item, Layer,
            Location, MoveHalfSpeed, Npc, OnCollision, Oxygen, ProjectileDamage, Tile,
        },
        explosion, player,
        realtime_periodic::{
            core::ScheduledRealtimePeriodicState,
            data::{period_per_frame, FadeState, LightColourFadeState},
            flicker, movement, particle,
        },
        World,
    },
};
use entity_table::Entity;
use grid_2d::coord_2d::Axis;
use grid_2d::Coord;
use rand::Rng;
use rational::Rational;
use rgb24::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

pub fn make_player<R: Rng>(rng: &mut R) -> EntityData {
    EntityData {
        tile: Some(Tile::Player),
        character: Some(()),
        player: Some(player::Player::new()),
        light: Some(Light {
            colour: Rgb24::new_grey(200),
            vision_distance: Circle::new_squared(120),
            diminish: Rational {
                numerator: 1,
                denominator: 8,
            },
        }),
        hit_points: Some(HitPoints::new_full(10)),
        oxygen: Some(Oxygen::new_full(10)),
        ..Default::default()
    }
}

impl World {
    pub fn insert_entity_data(&mut self, location: Location, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table.update(entity, location).unwrap();
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Wall);
        self.components.solid.insert(entity, ());
        self.components.opacity.insert(entity, 255);
        self.components.destructible.insert(entity, ());
        entity
    }

    pub fn spawn_invisible_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.solid.insert(entity, ());
        self.components.opacity.insert(entity, 255);
        entity
    }

    pub fn spawn_human(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Afraid,
            },
        );
        self.components.character.insert(entity, ());
        self.components
            .hit_points
            .insert(entity, HitPoints::new_full(20));
        panic!("missing tile")
    }

    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Floor);
        entity
    }

    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(entity, Location { coord, layer: None })
            .unwrap();
        self.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(200),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        entity
    }

    pub fn spawn_flickering_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(entity, Location { coord, layer: None })
            .unwrap();
        self.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(200),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        self.components.realtime.insert(entity, ());
        self.realtime_components.flicker.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use flicker::spec::*;
                    Flicker {
                        colour_hint: None,
                        light_colour: Some(UniformInclusiveRange {
                            low: Rgb24::new(0, 0, 0),
                            high: colour,
                        }),
                        tile: None,
                        until_next_event: UniformInclusiveRange {
                            low: Duration::from_millis(17),
                            high: Duration::from_millis(51),
                        },
                    }
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        entity
    }

    pub fn spawn_flash(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(entity, Location { coord, layer: None })
            .unwrap();
        self.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new_grey(100),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 20,
                },
            },
        );
        self.components.realtime.insert(entity, ());
        self.realtime_components.fade.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: FadeState::new(Duration::from_millis(100)),
                until_next_event: Duration::from_millis(0),
            },
        );
        entity
    }

    pub fn spawn_bullet(&mut self, start: Coord, target: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord: start,
                    layer: None,
                },
            )
            .unwrap();
        self.components.realtime.insert(entity, ());
        self.components.blocks_gameplay.insert(entity, ());
        self.components
            .on_collision
            .insert(entity, OnCollision::Remove);
        self.realtime_components.movement.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: movement::spec::Movement {
                    path: target - start,
                    cardinal_step_duration: Duration::from_millis(50),
                    repeat: movement::spec::Repeat::Once,
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.realtime_components.particle_emitter.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use particle::spec::*;
                    ParticleEmitter {
                        emit_particle_every_period: Duration::from_micros(2000),
                        fade_out_duration: None,
                        particle: Particle {
                            tile: None,
                            movement: Some(Movement {
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: Duration::from_millis(200),
                                    high: Duration::from_millis(500),
                                },
                            }),
                            fade_duration: Some(Duration::from_millis(1000)),
                            ..Default::default()
                        },
                    }
                    .build()
                },
                until_next_event: Duration::from_millis(0),
            },
        );
        self.components.collides_with.insert(
            entity,
            CollidesWith {
                solid: true,
                character: false,
            },
        );
        self.components.tile.insert(entity, Tile::Bullet);
        self.components.projectile_damage.insert(
            entity,
            ProjectileDamage {
                hit_points: 2,
                push_back: false,
                pen: 3,
                hull_pen_percent: 50,
            },
        );
        entity
    }

    pub fn spawn_rocket(&mut self, start: Coord, target: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord: start,
                    layer: None,
                },
            )
            .unwrap();
        self.components.realtime.insert(entity, ());
        self.components.blocks_gameplay.insert(entity, ());
        self.realtime_components.movement.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: movement::spec::Movement {
                    path: target - start,
                    cardinal_step_duration: Duration::from_millis(16),
                    repeat: movement::spec::Repeat::Once,
                }
                .build(),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.realtime_components.particle_emitter.insert(
            entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use particle::spec::*;
                    ParticleEmitter {
                        emit_particle_every_period: Duration::from_micros(500),
                        fade_out_duration: None,
                        particle: Particle {
                            tile: None,
                            movement: Some(Movement {
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: Duration::from_millis(200),
                                    high: Duration::from_millis(500),
                                },
                            }),
                            fade_duration: Some(Duration::from_millis(1000)),
                            ..Default::default()
                        },
                    }
                    .build()
                },
                until_next_event: Duration::from_millis(0),
            },
        );
        self.components.on_collision.insert(
            entity,
            OnCollision::Explode({
                use explosion::spec::*;
                Explosion {
                    mechanics: Mechanics { range: 10 },
                    particle_emitter: ParticleEmitter {
                        duration: Duration::from_millis(250),
                        num_particles_per_frame: 50,
                        min_step: Duration::from_millis(10),
                        max_step: Duration::from_millis(30),
                        fade_duration: Duration::from_millis(250),
                    },
                }
            }),
        );
        self.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new(255, 187, 63),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        self.components.collides_with.insert(
            entity,
            CollidesWith {
                solid: true,
                character: true,
            },
        );
        panic!("missing tiles")
    }

    pub fn spawn_explosion_emitter(
        &mut self,
        coord: Coord,
        spec: &explosion::spec::ParticleEmitter,
    ) -> Entity {
        let emitter_entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(emitter_entity, Location { coord, layer: None })
            .unwrap();
        self.realtime_components.fade.insert(
            emitter_entity,
            ScheduledRealtimePeriodicState {
                state: FadeState::new(spec.duration),
                until_next_event: Duration::from_millis(0),
            },
        );
        self.components.realtime.insert(emitter_entity, ());
        self.realtime_components.particle_emitter.insert(
            emitter_entity,
            ScheduledRealtimePeriodicState {
                state: {
                    use particle::spec::*;
                    ParticleEmitter {
                        emit_particle_every_period: period_per_frame(spec.num_particles_per_frame),
                        fade_out_duration: Some(spec.duration),
                        particle: Particle {
                            tile: None,
                            movement: Some(Movement {
                                angle_range: Radians::uniform_range_all(),
                                cardinal_period_range: UniformInclusiveRange {
                                    low: spec.min_step,
                                    high: spec.max_step,
                                },
                            }),
                            fade_duration: Some(spec.fade_duration),
                            colour_hint: Some(UniformInclusiveRange {
                                low: Rgb24::new(255, 17, 0),
                                high: Rgb24::new(255, 255, 63),
                            }),
                            possible_particle_emitter: Some(Possible {
                                chance: Rational {
                                    numerator: 1,
                                    denominator: 20,
                                },
                                value: Box::new(ParticleEmitter {
                                    emit_particle_every_period: spec.min_step,
                                    fade_out_duration: None,
                                    particle: Particle {
                                        tile: None,
                                        movement: Some(Movement {
                                            angle_range: Radians::uniform_range_all(),
                                            cardinal_period_range: UniformInclusiveRange {
                                                low: Duration::from_millis(200),
                                                high: Duration::from_millis(500),
                                            },
                                        }),
                                        fade_duration: Some(Duration::from_millis(1000)),
                                        ..Default::default()
                                    },
                                }),
                            }),
                            ..Default::default()
                        },
                    }
                    .build()
                },
                until_next_event: Duration::from_millis(0),
            },
        );
        self.components.light.insert(
            emitter_entity,
            Light {
                colour: Rgb24::new(255, 187, 63),
                vision_distance: Circle::new_squared(420),
                diminish: Rational {
                    numerator: 1,
                    denominator: 100,
                },
            },
        );
        self.realtime_components.light_colour_fade.insert(
            emitter_entity,
            ScheduledRealtimePeriodicState {
                state: LightColourFadeState {
                    fade_state: FadeState::new(spec.fade_duration),
                    from: Rgb24::new(255, 187, 63),
                    to: Rgb24::new(0, 0, 0),
                },
                until_next_event: Duration::from_millis(0),
            },
        );
        panic!("missing tiles")
    }

    pub fn spawn_door(&mut self, coord: Coord, axis: Axis) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::DoorClosed(axis));
        self.components.opacity.insert(entity, 255);
        self.components.solid.insert(entity, ());
        self.components.door_state.insert(entity, DoorState::Closed);
        self.components.destructible.insert(entity, ());
        entity
    }

    pub fn spawn_window(&mut self, coord: Coord, axis: Axis) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Window(axis));
        self.components.solid.insert(entity, ());
        self.components.destructible.insert(entity, ());
        entity
    }

    pub fn spawn_stairs(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Stairs);
        self.components.stairs.insert(entity, ());
        entity
    }

    pub fn spawn_credit(&mut self, coord: Coord, value: u32) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Item),
                },
            )
            .unwrap();
        let tile = if value == 1 {
            Tile::Credit1
        } else if value == 2 {
            Tile::Credit2
        } else {
            panic!()
        };
        self.components.tile.insert(entity, tile);
        self.components.item.insert(entity, Item::Credit(value));
        entity
    }

    pub fn spawn_upgrade(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Upgrade);
        self.components.upgrade.insert(entity, ());
        entity
    }

    pub fn spawn_zombie<R: Rng>(&mut self, coord: Coord, rng: &mut R) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Zombie);
        self.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Hostile,
            },
        );
        self.components.character.insert(entity, ());
        self.components
            .hit_points
            .insert(entity, HitPoints::new_full(4));
        self.components.armour.insert(entity, Armour::new(2));
        self.components.damage.insert(entity, 1);
        entity
    }
}
