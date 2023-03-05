use crate::{
    world::{
        data::{EntityData, Layer, Location, Tile},
        World,
    },
    Entity,
};
use gridbugs::{coord_2d::Coord, entity_table::entity_data};

pub fn make_player() -> EntityData {
    EntityData {
        tile: Some(Tile::Player),
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

    fn spawn_entity<L: Into<Location>>(&mut self, location: L, entity_data: EntityData) -> Entity {
        let entity = self.entity_allocator.alloc();
        let location @ Location { layer, coord } = location.into();
        if let Err(e) = self.spatial_table.update(entity, location) {
            panic!("{:?}: There is already a {:?} at {:?}", e, layer, coord);
        }
        self.components.insert_entity_data(entity, entity_data);
        entity
    }

    fn spawn_water(&mut self, coord: Coord, tile: Tile) -> Entity {
        self.spawn_entity(
            (coord, Layer::Water),
            entity_data! {
                tile,
            },
        )
    }

    pub fn spawn_water1(&mut self, coord: Coord) -> Entity {
        self.spawn_water(coord, Tile::Water1)
    }

    pub fn spawn_water2(&mut self, coord: Coord) -> Entity {
        self.spawn_water(coord, Tile::Water2)
    }

    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Wall,
                solid: (),
            },
        )
    }

    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::Floor,
            },
        )
    }

    pub fn spawn_rock(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::Rock,
                solid: (),
            },
        )
    }

    pub fn spawn_door(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::DoorClosed,
                solid: (),
            },
        )
    }

    pub fn spawn_boat_floor(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Floor),
            entity_data! {
                tile: Tile::BoatFloor,
                part_of_boat: (),
            },
        )
    }

    pub fn spawn_boat_edge(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Feature),
            entity_data! {
                tile: Tile::BoatEdge,
                solid: (),
                part_of_boat: (),
            },
        )
    }

    pub fn spawn_board(&mut self, coord: Coord) -> Entity {
        self.spawn_entity(
            (coord, Layer::Boat),
            entity_data! {
                tile: Tile::Board,
                part_of_boat: (),
            },
        )
    }
}
