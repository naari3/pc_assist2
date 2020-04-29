extern crate pcf;
extern crate regex;

use enumset::{enum_set, EnumSet, EnumSetType};
use pcf::{Piece, PieceState, Placement, Rotation};
use regex::Regex;

pub type Cells = [(i32, i32, EnumSet<Direction>); 4];

pub trait PlanPlacement {
    fn cells(&self) -> Cells;
}

impl PlanPlacement for Placement {
    fn cells(&self) -> Cells {
        let mut cells = self.kind.cells();

        let min_x = cells.iter().min_by_key(|cell| cell.0).unwrap().0;
        let min_y = cells.iter().min_by_key(|cell| cell.1).unwrap().1;

        let mut offset_x = 0;
        let mut offset_y = 0;

        if min_y < 0 {
            offset_y += -min_y;
        }

        if min_x < 0 {
            offset_x += -min_x;
        }

        for (x, y, _) in &mut cells {
            *y += self.kind.y() as i32 + offset_y;
            *x += self.x as i32 + offset_x;
            println!("x: {}, y: {}", x, y);
        }
        println!("piece x: {}, y: {}", self.x, self.kind.y());
        cells
    }
}

pub trait PlanPiceState {
    fn cells(&self) -> Cells;
}

impl PlanPiceState for PieceState {
    fn cells(&self) -> Cells {
        use Direction::*;

        let mut cells = match self.piece() {
            Piece::I => [
                (-1, 0, enum_set!(Right)),
                (0, 0, enum_set!(Left | Right)),
                (1, 0, enum_set!(Left | Right)),
                (2, 0, enum_set!(Left)),
            ],
            Piece::O => [
                (0, 0, enum_set!(Right | Up)),
                (1, 0, enum_set!(Left | Up)),
                (0, 1, enum_set!(Right | Down)),
                (1, 1, enum_set!(Left | Down)),
            ],
            Piece::L => [
                (-1, 0, enum_set!(Right)),
                (0, 0, enum_set!(Left | Right)),
                (1, 0, enum_set!(Left | Up)),
                (1, 1, enum_set!(Down)),
            ],
            Piece::J => [
                (-1, 0, enum_set!(Right | Up)),
                (0, 0, enum_set!(Left | Right)),
                (1, 0, enum_set!(Left)),
                (-1, 1, enum_set!(Down)),
            ],
            Piece::T => [
                (-1, 0, enum_set!(Right)),
                (0, 0, enum_set!(Left | Right | Up)),
                (1, 0, enum_set!(Left)),
                (0, 1, enum_set!(Down)),
            ],
            Piece::S => [
                (-1, 0, enum_set!(Right)),
                (0, 0, enum_set!(Left | Up)),
                (0, 1, enum_set!(Down | Right)),
                (1, 1, enum_set!(Left)),
            ],
            Piece::Z => [
                (-1, 1, enum_set!(Right)),
                (0, 1, enum_set!(Left | Down)),
                (0, 0, enum_set!(Up | Right)),
                (1, 0, enum_set!(Left)),
            ],
        };

        let enum_name = format!("{:?}", self);
        let rotation: Rotation;
        if enum_name.contains("O") {
            println!("O");
            rotation = Rotation::North;
        } else {
            let re = Regex::new(r"\A[SZTJLOI](.+?)\d+\z").unwrap();
            let matched = re.captures(&enum_name).unwrap().at(1).unwrap();
            println!("matched: {}", matched);
            rotation = match matched.to_string().as_str() {
                "North" => Rotation::North,
                "East" => Rotation::East,
                "South" => Rotation::South,
                "West" => Rotation::West,
                "Vertical" => Rotation::East,
                _ => Rotation::North,
            }
        }

        for (x, y, d) in &mut cells {
            match rotation {
                Rotation::North => {}
                Rotation::East => {
                    *x = -*x;
                    std::mem::swap(x, y);
                    *d = d.iter().map(Direction::cw).collect();
                }
                Rotation::South => {
                    *x = -*x;
                    *y = -*y;
                    *d = d.iter().map(Direction::flip).collect();
                }
                Rotation::West => {
                    *y = -*y;
                    std::mem::swap(x, y);
                    *d = d.iter().map(Direction::ccw).collect();
                }
            }
            // println!("x: {}, y: {}", x, y);
        }
        cells
    }
}

#[derive(EnumSetType, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn cw(self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn ccw(self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
        }
    }

    fn flip(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}
