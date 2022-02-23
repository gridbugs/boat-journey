use crate::{
    visibility::Light,
    world::{
        data::{
            Armour, CollidesWith, Disposition, DoorState, Enemy, EntityData, HitPoints, Item,
            Layer, Location, MeleeWeapon, Npc, OnCollision, Oxygen, ProjectileDamage, RangedWeapon,
            Tile,
        },
        explosion,
        player::{self, WeaponAbility},
        realtime, World,
    },
};
use direction::CardinalDirections;
use entity_table::Entity;
use grid_2d::coord_2d::Axis;
use grid_2d::Coord;
use rand::Rng;
use rational::Rational;
use rgb_int::Rgb24;
use shadowcast::vision_distance::Circle;
use std::time::Duration;

pub fn make_player() -> EntityData {
    EntityData {
        tile: Some(Tile::Player),
        character: Some(()),
        player: Some(player::Player::new()),
        light: Some(Light {
            colour: Rgb24::new_grey(200),
            vision_distance: Circle::new_squared(70),
            diminish: Rational {
                numerator: 1,
                denominator: 8,
            },
        }),
        hit_points: Some(HitPoints::new_full(10)),
        oxygen: Some(Oxygen::new_full(20)),
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
            realtime::fade::FadeState::new(Duration::from_millis(100)),
        );

        entity
    }

    pub fn spawn_bullet<R: Rng>(
        &mut self,
        start: Coord,
        target: Coord,
        weapon: &player::Weapon,
        rng: &mut R,
    ) -> Entity {
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
            realtime::movement::spec::Movement {
                path: target - start,
                cardinal_step_duration: Duration::from_millis(50),
                repeat: realtime::movement::spec::Repeat::Once,
            }
            .build(),
        );
        let particle_emitter_ = if let Some(light_colour) = weapon.light_colour {
            use realtime::particle::spec::*;
            if weapon.bright {
                ParticleEmitter {
                    emit_particle_every_period: Duration::from_millis(8),
                    fade_out_duration: None,
                    particle: Particle {
                        tile: None,
                        movement: None,
                        fade_duration: Some(Duration::from_millis(1000)),
                        possible_light: Some(Possible {
                            chance: Rational {
                                numerator: 1,
                                denominator: 1,
                            },
                            value: Light {
                                colour: light_colour,
                                vision_distance: Circle::new_squared(50),
                                diminish: Rational {
                                    numerator: 10,
                                    denominator: 1,
                                },
                            },
                        }),
                        ..Default::default()
                    },
                }
            } else {
                ParticleEmitter {
                    emit_particle_every_period: Duration::from_millis(1),
                    fade_out_duration: None,
                    particle: Particle {
                        tile: None,
                        movement: None,
                        fade_duration: Some(Duration::from_millis(100)),
                        possible_light: Some(Possible {
                            chance: Rational {
                                numerator: 1,
                                denominator: 1,
                            },
                            value: Light {
                                colour: light_colour,
                                vision_distance: Circle::new_squared(7),
                                diminish: Rational {
                                    numerator: 100,
                                    denominator: 1,
                                },
                            },
                        }),
                        ..Default::default()
                    },
                }
            }
        } else {
            use realtime::particle::spec::*;
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
                    possible_light: None,
                    ..Default::default()
                },
            }
        }
        .build(rng);
        self.realtime_components
            .particle_emitter
            .insert(entity, particle_emitter_);
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
                hit_points: weapon.dmg,
                push_back: weapon
                    .abilities
                    .iter()
                    .any(|a| *a == player::WeaponAbility::KnockBack),
                pen: weapon.pen,
                hull_pen_percent: weapon.hull_pen_percent,
                oxidise: weapon
                    .abilities
                    .iter()
                    .any(|a| *a == WeaponAbility::Oxidise),
                life_steal: weapon
                    .abilities
                    .iter()
                    .any(|a| *a == WeaponAbility::LifeSteal),
                weapon_name: Some(weapon.name),
            },
        );
        entity
    }

    pub fn spawn_explosion_emitter<R: Rng>(
        &mut self,
        coord: Coord,
        spec: &explosion::spec::ParticleEmitter,
        rng: &mut R,
    ) -> Entity {
        let emitter_entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(emitter_entity, Location { coord, layer: None })
            .unwrap();
        self.realtime_components.fade.insert(
            emitter_entity,
            realtime::fade::FadeState::new(spec.duration),
        );
        self.components.realtime.insert(emitter_entity, ());
        self.realtime_components
            .particle_emitter
            .insert(emitter_entity, {
                use realtime::particle::spec::*;
                ParticleEmitter {
                    emit_particle_every_period: realtime::data::period_per_frame(
                        spec.num_particles_per_frame,
                    ),
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
                .build(rng)
            });
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
            realtime::light_colour_fade::LightColourFadeState {
                fade_state: realtime::fade::FadeState::new(spec.fade_duration),
                from: Rgb24::new(255, 187, 63),
                to: Rgb24::new(0, 0, 0),
            },
        );
        emitter_entity
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
        self.components.solid.insert(entity, ());
        entity
    }

    pub fn spawn_map(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::MapLocked);
        self.components.map.insert(entity, true);
        self.components.solid.insert(entity, ());
        entity
    }

    pub fn spawn_zombie(&mut self, coord: Coord) -> Entity {
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
        self.components.enemy.insert(entity, Enemy::Zombie);
        entity
    }

    pub fn spawn_skeleton(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Skeleton);
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
        self.components.armour.insert(entity, Armour::new(4));
        self.components.damage.insert(entity, 1);
        self.components.skeleton.insert(entity, ());
        self.components.enemy.insert(entity, Enemy::Skeleton);
        entity
    }

    pub fn spawn_skeleton_respawn(&mut self, mut coord: Coord) -> Option<Entity> {
        if self.spatial_table.layers_at_checked(coord).item.is_some() {
            let mut new_coord = None;
            for direction in CardinalDirections {
                let nei_coord = coord + direction.coord();
                if let Some(layers) = self.spatial_table.layers_at(nei_coord) {
                    if layers.item.is_none() {
                        new_coord = Some(nei_coord);
                        break;
                    }
                }
            }
            if let Some(new_coord) = new_coord {
                coord = new_coord;
            } else {
                return None;
            }
        }
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
        self.components.tile.insert(entity, Tile::SkeletonRespawn);
        self.components.skeleton_respawn.insert(entity, 11);
        Some(entity)
    }

    pub fn spawn_tank(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Tank);
        self.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Hostile,
            },
        );
        self.components.character.insert(entity, ());
        self.components
            .hit_points
            .insert(entity, HitPoints::new_full(10));
        self.components.armour.insert(entity, Armour::new(10));
        self.components.damage.insert(entity, 2);
        self.components.push_back.insert(entity, ());
        self.components.enemy.insert(entity, Enemy::Tank);
        entity
    }

    pub fn spawn_boomer(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Boomer);
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
        self.components.armour.insert(entity, Armour::new(4));
        self.components.damage.insert(entity, 1);
        self.components.expoodes_on_death.insert(entity, ());
        self.components.enemy.insert(entity, Enemy::Boomer);
        entity
    }

    pub fn spawn_ranged_weapon(&mut self, coord: Coord, ranged_weapon: RangedWeapon) -> Entity {
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
        self.components.tile.insert(entity, ranged_weapon.tile());
        self.components
            .item
            .insert(entity, Item::RangedWeapon(ranged_weapon));
        self.components
            .weapon
            .insert(entity, ranged_weapon.new_weapon());
        entity
    }

    pub fn spawn_melee_weapon(&mut self, coord: Coord, melee_weapon: MeleeWeapon) -> Entity {
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
        self.components.tile.insert(entity, melee_weapon.tile());
        self.components
            .item
            .insert(entity, Item::MeleeWeapon(melee_weapon));
        self.components
            .weapon
            .insert(entity, melee_weapon.new_weapon());
        entity
    }

    pub fn spawn_medkit(&mut self, coord: Coord) -> Entity {
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
        self.components.tile.insert(entity, Tile::Medkit);
        self.components.item.insert(entity, Item::Medkit);
        entity
    }
}
