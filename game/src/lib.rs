pub use direction::CardinalDirection;
pub use grid_2d::{Coord, Grid, Size};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};
use shadowcast::Context as ShadowcastContext;
use std::time::Duration;

mod behaviour;
mod terrain;
mod visibility;
pub mod witness;
mod world;

use behaviour::{Agent, BehaviourContext};
use entity_table::ComponentTable;
pub use entity_table::Entity;
pub use terrain::FINAL_LEVEL;
use terrain::{SpaceStationSpec, Terrain, TerrainState};
pub use visibility::{CellVisibility, EntityTile, Omniscient, VisibilityCell, VisibilityGrid};
use world::{make_player, AnimationContext, AnimationContext_, World, ANIMATION_FRAME_DURATION};
pub use world::{
    player, ActionError, CharacterInfo, Enemy, EntityData, HitPoints, Item, Layer, MeleeWeapon,
    NpcAction, PlayerDied, RangedWeapon, Tile, ToRenderEntity, ToRenderEntityRealtime,
};

pub const MAP_SIZE: Size = Size::new_u16(20, 14);

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Message {
    MaybeThisWasntSuchAGoodIdea,
    EquipWeapon(player::WeaponName),
    PulledByVacuum,
    Descend,
    Suffocating,
    Heal,
    TakeCredit(u32),
    BoomerExplodes,
    EnemyHitPlayer(Enemy),
    PlayerHitEnemy {
        enemy: Enemy,
        weapon: player::WeaponName,
    },
    PlayerDies,
    EnemyDies(Enemy),
    PlayerAdrift,
    EnemyAdrift(Enemy),
    MapTerminal,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub omniscient: Option<Omniscient>,
    pub demo: bool,
    pub debug: bool,
}
impl Config {
    pub const OMNISCIENT: Option<Omniscient> = Some(Omniscient);
}
impl Default for Config {
    fn default() -> Self {
        Self {
            omniscient: None,
            demo: false,
            debug: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Music {
    Gameplay0,
    Gameplay1,
    Gameplay2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SoundEffect {
    Shotgun,
    Rifle,
    Railgun,
    GausCannon,
    LifeStealer,
    Oxidiser,
    Chainsaw,
    Punch,
    DoorOpen,
    Heal,
    Die,
}

/// Events which the game can report back to the io layer so it can
/// respond with a sound/visual effect.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ExternalEvent {
    Explosion(Coord),
    LoopMusic(Music),
    SoundEffect(SoundEffect),
}

#[derive(Debug)]
pub enum GameControlFlow {
    GameOver,
    Win,
    LevelChange,
    Upgrade,
    UnlockMap,
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Walk(CardinalDirection),
    Wait,
    Fire {
        direction: CardinalDirection,
        slot: player::RangedWeaponSlot,
    },
    Upgrade(player::Upgrade),
    EquipMeleeWeapon,
    EquipRangedWeapon(player::RangedWeaponSlot),
    UnlockMap,
}

pub enum WarningLight {
    NoAir,
    Decompression,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Turn {
    Player,
    Npc,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    world: World,
    visibility_grid: VisibilityGrid,
    player: Entity,
    last_player_info: CharacterInfo,
    rng: Isaac64Rng,
    animation_rng: Isaac64Rng,
    events: Vec<ExternalEvent>,
    shadowcast_context: ShadowcastContext<u8>,
    behaviour_context: BehaviourContext,
    animation_context: AnimationContext,
    animation_context_: AnimationContext_,
    agents: ComponentTable<Agent>,
    agents_to_remove: Vec<Entity>,
    since_last_frame: Duration,
    generate_frame_countdown: Option<Duration>,
    after_player_turn_countdown: Option<Duration>,
    before_npc_turn_cooldown: Option<Duration>,
    dead_player: Option<EntityData>,
    turn_during_animation: Option<Turn>,
    gameplay_music: Vec<Music>,
    star_rng_seed: u64,
    won: bool,
    adrift: bool,
    terrain_state: TerrainState,
    message_log: Vec<Message>,
}

impl Game {
    pub fn new<R: Rng>(config: &Config, base_rng: &mut R) -> Self {
        let mut rng = Isaac64Rng::seed_from_u64(base_rng.gen());
        let animation_rng = Isaac64Rng::seed_from_u64(base_rng.gen());
        let star_rng_seed = base_rng.gen();
        let mut terrain_state = TerrainState::new(&mut rng);
        let Terrain {
            mut world,
            agents,
            player,
        } = if config.debug {
            terrain::from_str(include_str!("terrain.txt"), make_player())
        } else {
            terrain::space_station(
                0,
                make_player(),
                &SpaceStationSpec { demo: config.demo },
                &mut terrain_state,
                &mut rng,
            )
        };
        if config.debug {
            /*
            world
                .components
                .player
                .get_mut(player)
                .unwrap()
                .ranged_weapons[0] = Some(player::Weapon::new_life_stealer());
            world
                .components
                .player
                .get_mut(player)
                .unwrap()
                .ranged_weapons[1] = Some(player::Weapon::new_oxidiser());
            world
                .components
                .player
                .get_mut(player)
                .unwrap()
                .melee_weapon = player::Weapon::new_chainsaw();

            let _ = world.apply_upgrade(
                player,
                player::Upgrade {
                    typ: player::UpgradeType::Toughness,
                    level: player::UpgradeLevel::Level1,
                },
            );*/
            world.components.player.get_mut(player).unwrap().credit = 100;
        }
        world.air.init(&world.spatial_table, &world.components);
        let last_player_info = world
            .character_info(player)
            .expect("couldn't get info for player");
        let mut gameplay_music = vec![Music::Gameplay0, Music::Gameplay1, Music::Gameplay2];
        gameplay_music.shuffle(&mut rng);
        let events = vec![ExternalEvent::LoopMusic(gameplay_music[0])];
        let mut game = Self {
            visibility_grid: VisibilityGrid::new(world.size()),
            player,
            last_player_info,
            rng,
            animation_rng,
            events,
            shadowcast_context: ShadowcastContext::default(),
            behaviour_context: BehaviourContext::new(world.size()),
            animation_context: AnimationContext::default(),
            animation_context_: AnimationContext_::default(),
            agents,
            agents_to_remove: Vec::new(),
            world,
            since_last_frame: Duration::from_millis(0),
            generate_frame_countdown: None,
            after_player_turn_countdown: None,
            before_npc_turn_cooldown: None,
            dead_player: None,
            turn_during_animation: None,
            gameplay_music,
            star_rng_seed,
            won: false,
            adrift: false,
            terrain_state,
            message_log: Vec::new(),
        };
        game.update_visibility(config);
        game.prime_npcs();
        game
    }
    pub fn message_log(&self) -> &[Message] {
        &self.message_log
    }
    pub fn player_has_usable_weapon_in_slot(&self, slot: player::RangedWeaponSlot) -> bool {
        let player = self.world.components.player.get(self.player).unwrap();
        if slot.index() >= player.ranged_weapons.len() {
            return false;
        }
        if let Some(weapon) = player.ranged_weapons[slot.index()].as_ref() {
            weapon.ammo.map(|a| a.current > 0).unwrap_or(true)
        } else {
            false
        }
    }
    pub fn player_has_weapon_in_slot(&self, slot: player::RangedWeaponSlot) -> bool {
        let player = self.world.components.player.get(self.player).unwrap();
        if slot.index() >= player.ranged_weapons.len() {
            return false;
        }
        player.ranged_weapons[slot.index()].is_some()
    }
    pub fn player_has_third_weapon_slot(&self) -> bool {
        let player = self.world.components.player.get(self.player).unwrap();
        player.ranged_weapons.len() == 3
    }
    pub fn player_has_melee_weapon_equipped(&self) -> bool {
        let player = self.world.components.player.get(self.player).unwrap();
        player.melee_weapon.is_melee()
    }
    pub fn weapon_under_player(&self) -> Option<&player::Weapon> {
        self.world
            .spatial_table
            .layers_at(self.player_coord())
            .and_then(|layers| {
                layers
                    .item
                    .and_then(|item_entity| self.world.components.weapon.get(item_entity))
            })
    }
    pub fn available_upgrades(&self) -> Vec<player::Upgrade> {
        let player = self
            .world
            .components
            .player
            .get(self.player)
            .expect("no player");
        player.available_upgrades()
    }
    pub fn warning_light(&self, coord: Coord) -> Option<WarningLight> {
        if let Some(layers) = self.world.spatial_table.layers_at(coord) {
            if layers.floor.is_some() {
                if !self.world.air.has_air(coord) {
                    Some(WarningLight::NoAir)
                } else if self.world.air.has_flow(coord) {
                    Some(WarningLight::Decompression)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn is_adrift(&self) -> bool {
        self.adrift
    }
    pub fn star_rng_seed(&self) -> u64 {
        self.star_rng_seed
    }
    pub fn size(&self) -> Size {
        self.world.size()
    }
    fn cleanup(&mut self) {
        if let Some(PlayerDied(player_data)) = self.world.cleanup() {
            self.dead_player = Some(player_data);
        }
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        self.world.is_gameplay_blocked()
    }
    pub fn update_visibility(&mut self, config: &Config) {
        if let Some(player_coord) = self.world.entity_coord(self.player) {
            let mut map = None;
            if let Some(layers) = self.world.spatial_table.layers_at(player_coord) {
                if let Some(feature) = layers.feature {
                    if self.world.components.map.contains(feature) {
                        map = Some(Omniscient);
                    }
                }
            }
            self.visibility_grid.update(
                player_coord,
                &self.world,
                &mut self.shadowcast_context,
                config.omniscient.or(map),
            );
        }
    }
    fn update_behaviour(&mut self) {
        self.behaviour_context.update(self.player, &self.world);
    }

    #[must_use]
    pub fn handle_tick(
        &mut self,
        since_last_tick: Duration,
        config: &Config,
    ) -> Option<GameControlFlow> {
        if let Some(countdown) = self.generate_frame_countdown.as_mut() {
            if countdown.as_millis() == 0 {
                if self.world.level == terrain::FINAL_LEVEL {
                    self.won = true;
                    return Some(GameControlFlow::Win);
                }
                self.generate_level(config);
                self.generate_frame_countdown = None;
                return Some(GameControlFlow::LevelChange);
            } else {
                *countdown = if let Some(remaining) = countdown.checked_sub(since_last_tick) {
                    remaining
                } else {
                    Duration::from_millis(0)
                };
            }
            return None;
        }
        self.since_last_frame += since_last_tick;
        while let Some(remaining_since_last_frame) =
            self.since_last_frame.checked_sub(ANIMATION_FRAME_DURATION)
        {
            self.since_last_frame = remaining_since_last_frame;
            if let Some(game_control_flow) = self.handle_tick_inner(since_last_tick, config) {
                return Some(game_control_flow);
            }
        }
        None
    }
    fn handle_tick_inner(
        &mut self,
        since_last_tick: Duration,
        config: &Config,
    ) -> Option<GameControlFlow> {
        self.world.animation_tick(
            &mut self.animation_context,
            &mut self.animation_context_,
            &mut self.events,
            &mut self.message_log,
            &mut self.animation_rng,
        );
        if !self.is_gameplay_blocked() {
            if let Some(turn_during_animation) = self.turn_during_animation {
                if let Some(countdown) = self.after_player_turn_countdown.as_mut() {
                    if countdown.as_millis() == 0 {
                        self.after_player_turn_countdown = None;
                        self.after_turn();
                    } else {
                        *countdown = if let Some(remaining) = countdown.checked_sub(since_last_tick)
                        {
                            remaining
                        } else {
                            Duration::from_millis(0)
                        }
                    }
                    return None;
                }
                if let Some(countdown) = self.before_npc_turn_cooldown.as_mut() {
                    if countdown.as_millis() == 0 {
                        self.before_npc_turn_cooldown = None;
                    } else {
                        *countdown = if let Some(remaining) = countdown.checked_sub(since_last_tick)
                        {
                            remaining
                        } else {
                            Duration::from_millis(0)
                        }
                    }
                    return None;
                }
                if let Turn::Player = turn_during_animation {
                    self.npc_turn();
                }
                self.turn_during_animation = None;
            }
        }
        self.update_visibility(config);
        self.update_last_player_info();
        if self.is_game_over() {
            Some(GameControlFlow::GameOver)
        } else if self.won {
            Some(GameControlFlow::Win)
        } else {
            None
        }
    }

    #[must_use]
    pub fn handle_input(
        &mut self,
        input: Input,
        config: &Config,
    ) -> Result<Option<GameControlFlow>, ActionError> {
        if let Input::Upgrade(upgrade) = input {
            self.world.apply_upgrade(self.player, upgrade)?;
            return Ok(None);
        }
        if let Input::UnlockMap = input {
            self.world.unlock_map(self.player);
            return Ok(None);
        }
        if self.generate_frame_countdown.is_some() {
            return Ok(None);
        }
        let mut change = false;
        if !self.is_gameplay_blocked() && self.turn_during_animation.is_none() {
            change = true;
            if let Some(control_flow) = self.player_turn(input)? {
                return Ok(Some(control_flow));
            }
        }
        if change {
            self.update_last_player_info();
            self.update_visibility(config);
        }
        if self.is_game_over() {
            Ok(Some(GameControlFlow::GameOver))
        } else if self.won {
            Ok(Some(GameControlFlow::Win))
        } else {
            Ok(None)
        }
    }
    pub fn handle_npc_turn(&mut self) {
        if !self.is_gameplay_blocked() {
            self.npc_turn();
        }
    }
    fn prime_npcs(&mut self) {
        self.update_behaviour();
    }

    fn player_turn(&mut self, input: Input) -> Result<Option<GameControlFlow>, ActionError> {
        let result = match input {
            Input::Walk(direction) => self.world.character_walk_in_direction(
                self.player,
                direction,
                &mut self.rng,
                &mut self.events,
                &mut self.message_log,
            ),
            Input::Wait => Ok(None),
            Input::Fire { direction, slot } => {
                self.world.character_fire_bullet(
                    self.player,
                    self.player_coord() + (direction.coord() * 100),
                    slot,
                    &mut self.events,
                    &mut self.rng,
                );
                Ok(None)
            }
            Input::Upgrade(_upgrade) => Ok(None),
            Input::EquipMeleeWeapon => {
                self.world
                    .equip_melee_weapon_from_ground(self.player, &mut self.message_log);
                Ok(None)
            }
            Input::EquipRangedWeapon(slot) => {
                self.world.equip_ranged_weapon_from_ground(
                    self.player,
                    slot,
                    &mut self.message_log,
                );
                Ok(None)
            }
            Input::UnlockMap => Ok(None),
        };
        if result.is_ok() {
            if self.is_gameplay_blocked() {
                self.after_player_turn_countdown = Some(Duration::from_millis(0));
                self.before_npc_turn_cooldown = Some(Duration::from_millis(100));
            }
            self.turn_during_animation = Some(Turn::Player);
        }
        result
    }

    pub fn resolve_realtime(&mut self) {
        if self.is_gameplay_blocked() {
            self.after_player_turn_countdown = Some(Duration::from_millis(0));
            self.before_npc_turn_cooldown = Some(Duration::from_millis(100));
        }
        self.turn_during_animation = Some(Turn::Player);
    }

    pub fn player_fire_weapon(
        &mut self,
        slot: player::RangedWeaponSlot,
        direction: CardinalDirection,
    ) {
        self.world.character_fire_bullet(
            self.player,
            self.player_coord() + (direction.coord() * 100),
            slot,
            &mut self.events,
            &mut self.rng,
        );
        self.resolve_realtime();
    }

    pub fn player_equip_ranged_weapon_from_ground(&mut self, slot: player::RangedWeaponSlot) {
        self.world
            .equip_ranged_weapon_from_ground(self.player, slot, &mut self.message_log);
        self.world.cleanup();
    }

    pub fn player_equip_melee_weapon_from_ground(&mut self) {
        self.world
            .equip_melee_weapon_from_ground(self.player, &mut self.message_log);
        self.world.cleanup();
    }

    fn npc_turn(&mut self) {
        for i in 0..2 {
            let to_move = self
                .world
                .air
                .update(&self.world.spatial_table, &self.world.components);
            for (entity, direction) in to_move {
                if let Some(player) = self.world.components.player.get(entity) {
                    if i == 0 {
                        self.message_log.push(Message::PulledByVacuum);
                    }
                    if player.traits.half_vacuum_pull && i == 1 {
                        continue;
                    }
                }
                let _ = self
                    .world
                    .character_pull_in_direction(entity, direction, &mut self.rng);
            }
            self.update_last_player_info();
        }
        self.world.process_door_close_countdown();
        self.world.process_oxygen(
            self.player,
            &mut self.rng,
            &mut self.events,
            &mut self.message_log,
        );
        self.world
            .process_skeleton_respawn(&mut self.rng, &mut self.agents, &mut self.events);
        if let Some(layers) = self.world.spatial_table.layers_at(self.player_coord()) {
            if let Some(item_entity) = layers.item {
                if let Some(item) = self.world.components.item.get(item_entity) {
                    match item {
                        Item::Credit(amount) => {
                            if let Some(player) = self.world.components.player.get_mut(self.player)
                            {
                                self.message_log.push(Message::TakeCredit(*amount));
                                player.credit += amount;
                            }
                            self.world.components.to_remove.insert(item_entity, ());
                        }
                        Item::RangedWeapon(_) => {}
                        Item::MeleeWeapon(_) => {}
                        Item::Medkit => {
                            self.world.heal_fully(
                                self.player,
                                &mut self.events,
                                &mut self.message_log,
                            );
                            self.world.components.to_remove.insert(item_entity, ());
                        }
                    }
                }
            }
        }
        self.update_behaviour();
        for (entity, agent) in self.agents.iter_mut() {
            if !self.world.entity_exists(entity) {
                self.agents_to_remove.push(entity);
                continue;
            }
            let input = agent.act(
                entity,
                &self.world,
                self.player,
                &mut self.behaviour_context,
                &mut self.shadowcast_context,
                &mut self.rng,
            );
            match input {
                NpcAction::Walk(direction) => {
                    let _ = self.world.character_walk_in_direction(
                        entity,
                        direction,
                        &mut self.rng,
                        &mut self.events,
                        &mut self.message_log,
                    );
                }
                NpcAction::Wait => (),
            }
        }
        self.update_last_player_info();
        for entity in self.agents_to_remove.drain(..) {
            self.agents.remove(entity);
        }
        self.after_turn();
    }
    fn generate_level(&mut self, config: &Config) {
        self.message_log.push(Message::Descend);
        let mut player_data = self.world.clone_entity_data(self.player);
        for weapon in player_data
            .player
            .as_mut()
            .unwrap()
            .ranged_weapons
            .iter_mut()
        {
            if let Some(weapon) = weapon.as_mut() {
                if let Some(ammo) = weapon.ammo.as_mut() {
                    ammo.current = ammo.max;
                }
            }
        }
        if let Some(ammo) = player_data
            .player
            .as_mut()
            .unwrap()
            .melee_weapon
            .ammo
            .as_mut()
        {
            ammo.current = ammo.max;
        }
        let Terrain {
            mut world,
            agents,
            player,
        } = terrain::space_station(
            self.world.level + 1,
            player_data,
            &SpaceStationSpec { demo: config.demo },
            &mut self.terrain_state,
            &mut self.rng,
        );
        world.air.init(&world.spatial_table, &world.components);
        self.visibility_grid = VisibilityGrid::new(world.size());
        self.world = world;
        self.agents = agents;
        self.player = player;
        self.update_last_player_info();
        self.update_visibility(config);
        self.prime_npcs();
        self.events.push(ExternalEvent::LoopMusic(
            self.gameplay_music[self.world.level as usize % self.gameplay_music.len()],
        ));
    }

    fn after_turn(&mut self) {
        if let Some(layers) = self.world.spatial_table.layers_at(self.player_coord()) {
            if layers.floor.is_none() {
                self.world.components.to_remove.insert(self.player, ());
                if !self.adrift {
                    self.message_log.push(Message::PlayerAdrift);
                }
                self.adrift = true;
            }
        }
        for npc in self.world.components.npc.entities() {
            if let Some(coord) = self.world.spatial_table.coord_of(npc) {
                if let Some(layers) = self.world.spatial_table.layers_at(coord) {
                    if layers.floor.is_none() {
                        self.world.components.to_remove.insert(npc, ());
                        if let Some(enemy) = self.world.components.enemy.get(npc) {
                            self.message_log.push(Message::EnemyAdrift(*enemy));
                        }
                    }
                }
            }
        }
        for item in self.world.components.item.entities() {
            if let Some(coord) = self.world.spatial_table.coord_of(item) {
                if let Some(layers) = self.world.spatial_table.layers_at(coord) {
                    if layers.floor.is_none() {
                        self.world.components.to_remove.insert(item, ());
                    }
                }
            }
        }
        for skeleton_respawn in self.world.components.skeleton_respawn.entities() {
            if let Some(coord) = self.world.spatial_table.coord_of(skeleton_respawn) {
                if let Some(layers) = self.world.spatial_table.layers_at(coord) {
                    if layers.floor.is_none() {
                        self.world.components.to_remove.insert(skeleton_respawn, ());
                    }
                }
            }
        }

        self.cleanup();
        if let Some(player_coord) = self.world.entity_coord(self.player) {
            if let Some(_stairs_entity) = self.world.get_stairs_at_coord(player_coord) {
                self.generate_frame_countdown = Some(Duration::from_millis(200));
            }
        }
        for entity in self.world.components.npc.entities() {
            if !self.agents.contains(entity) {
                self.agents.insert(entity, Agent::new(self.world.size()));
            }
        }
        self.cleanup();
    }
    pub fn is_generating(&self) -> bool {
        if let Some(countdown) = self.generate_frame_countdown {
            countdown.as_millis() == 0
        } else {
            false
        }
    }
    pub fn events(&mut self) -> impl '_ + Iterator<Item = ExternalEvent> {
        self.events.drain(..)
    }
    pub fn player_info(&self) -> &CharacterInfo {
        &self.last_player_info
    }
    pub fn world_size(&self) -> Size {
        self.world.size()
    }
    pub fn to_render_entity(&self, entity: Entity) -> Option<ToRenderEntity> {
        self.world.to_render_entity(entity)
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        self.world.to_render_entities()
    }
    pub fn to_render_entities_realtime<'a>(
        &'a self,
    ) -> impl 'a + Iterator<Item = ToRenderEntityRealtime> {
        self.world.to_render_entities_realtime()
    }
    pub fn visibility_grid(&self) -> &VisibilityGrid {
        &self.visibility_grid
    }
    pub fn contains_wall(&self, coord: Coord) -> bool {
        self.world.is_wall_at_coord(coord)
    }
    pub fn contains_wall_like(&self, coord: Coord) -> bool {
        self.world.is_wall_like_at_coord(coord)
    }
    pub fn contains_floor(&self, coord: Coord) -> bool {
        self.world.is_floor_at_coord(coord)
    }
    fn update_last_player_info(&mut self) {
        if let Some(character_info) = self.world.character_info(self.player) {
            self.last_player_info = character_info;
        }
    }
    fn is_game_over(&self) -> bool {
        self.dead_player.is_some()
    }
    pub fn player(&self) -> &player::Player {
        if let Some(player) = self.world.entity_player(self.player) {
            player
        } else {
            self.dead_player.as_ref().unwrap().player.as_ref().unwrap()
        }
    }
    pub fn player_coord(&self) -> Coord {
        self.last_player_info.coord
    }
    pub fn player_hit_points(&self) -> Coord {
        self.last_player_info.coord
    }
    pub fn current_level(&self) -> u32 {
        self.world.level
    }
}
