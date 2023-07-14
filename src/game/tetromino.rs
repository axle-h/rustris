use std::slice::Iter;
use super::geometry::{Point, Rotation};
use bitflags::{bitflags, Flags};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TetrominoShape {
    /// XXXX
    I,
    /// XX
    /// XX
    O,
    ///  X
    /// XXX
    T,
    ///  XX
    /// XX
    S,
    /// XX
    ///  XX
    Z,
    /// X
    /// XXX
    J,
    ///   X
    /// XXX
    L,
}

impl TetrominoShape {
    pub const ALL: [TetrominoShape; 7] = [
        TetrominoShape::I, TetrominoShape::O, TetrominoShape::T, TetrominoShape::S,
        TetrominoShape::Z, TetrominoShape::J, TetrominoShape::L
    ];

    pub fn meta(&self) -> &TetrominoMeta {
        match self {
            TetrominoShape::I => &I,
            TetrominoShape::O => &O,
            TetrominoShape::T => &T,
            TetrominoShape::S => &S,
            TetrominoShape::Z => &Z,
            TetrominoShape::J => &J,
            TetrominoShape::L => &L,
        }
    }

    fn offsets(&self, rotation: &Rotation) -> TetrominoOffsets {
        match self {
            TetrominoShape::I => match rotation {
                Rotation::North => TETROMINO_OFFSETS_I_NORTH,
                Rotation::East => TETROMINO_OFFSETS_I_EAST,
                Rotation::South => TETROMINO_OFFSETS_I_SOUTH,
                Rotation::West => TETROMINO_OFFSETS_I_WEST,
            },
            TetrominoShape::O => match rotation {
                Rotation::North => TETROMINO_OFFSETS_O_NORTH,
                Rotation::East => TETROMINO_OFFSETS_O_EAST,
                Rotation::South => TETROMINO_OFFSETS_O_SOUTH,
                Rotation::West => TETROMINO_OFFSETS_O_WEST,
            },
            _ => match rotation {
                Rotation::North => TETROMINO_OFFSETS_NORTH,
                Rotation::East => TETROMINO_OFFSETS_EAST,
                Rotation::South => TETROMINO_OFFSETS_SOUTH,
                Rotation::West => TETROMINO_OFFSETS_WEST,
            },
        }
    }
}

#[derive(Copy, Clone)]
struct Offset(i32, i32);

type TetrominoOffsets = [Offset; 5];

/// https://tetris.wiki/Super_Rotation_System
const TETROMINO_OFFSETS_NORTH: TetrominoOffsets = [
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
];
const TETROMINO_OFFSETS_EAST: TetrominoOffsets = [
    Offset(0, 0),
    Offset(1, 0),
    Offset(1, -1),
    Offset(0, 2),
    Offset(1, 2),
];
const TETROMINO_OFFSETS_SOUTH: TetrominoOffsets = [
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 0),
];
const TETROMINO_OFFSETS_WEST: TetrominoOffsets = [
    Offset(0, 0),
    Offset(-1, 0),
    Offset(-1, -1),
    Offset(0, 2),
    Offset(-1, 2),
];

const TETROMINO_OFFSETS_I_NORTH: TetrominoOffsets = [
    Offset(0, 0),
    Offset(-1, 0),
    Offset(2, 0),
    Offset(-1, 0),
    Offset(2, 0),
];
const TETROMINO_OFFSETS_I_EAST: TetrominoOffsets = [
    Offset(-1, 0),
    Offset(0, 0),
    Offset(0, 0),
    Offset(0, 1),
    Offset(0, -2),
];
const TETROMINO_OFFSETS_I_SOUTH: TetrominoOffsets = [
    Offset(-1, 1),
    Offset(1, 1),
    Offset(-2, 1),
    Offset(1, 0),
    Offset(-2, 0),
];
const TETROMINO_OFFSETS_I_WEST: TetrominoOffsets = [
    Offset(0, 1),
    Offset(0, 1),
    Offset(0, 1),
    Offset(0, -1),
    Offset(0, 2),
];

const TETROMINO_OFFSETS_O_NORTH: TetrominoOffsets = [Offset(0, 0); 5];
const TETROMINO_OFFSETS_O_EAST: TetrominoOffsets = [Offset(0, -1); 5];
const TETROMINO_OFFSETS_O_SOUTH: TetrominoOffsets = [Offset(-1, -1); 5];
const TETROMINO_OFFSETS_O_WEST: TetrominoOffsets = [Offset(-1, 0); 5];

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Perimeter: u8 {
        const Top = 0b00000001;
        const Right = 0b00000010;
        const Bottom = 0b00000100;
        const Left = 0b00001000;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Corner: u8 {
        const None = 0;
        const TopLeft = 0b00000001;
        const TopRight = 0b00000010;
        const BottomRight = 0b00000100;
        const BottomLeft = 0b00001000;
    }
}

pub type Minos = [Point; 4];
pub type MinoPerimeter = [Perimeter; 4];
pub type MinoCorners = [Corner; 4];

const fn perimeter(minos: [u8; 4]) -> MinoPerimeter {
    // error[E0658]: `for` is not allowed in a `const fn`
    let mut result = [Perimeter::Top; 4];
    result[0] = Perimeter::from_bits_truncate(minos[0]);
    result[1] = Perimeter::from_bits_truncate(minos[1]);
    result[2] = Perimeter::from_bits_truncate(minos[2]);
    result[3] = Perimeter::from_bits_truncate(minos[3]);
    return result;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TetrominoMeta {
    shape: TetrominoShape,
    spawn_point: Point,
    minos: Minos,
    perimeter: MinoPerimeter,
    outside_corners: MinoCorners,
    bounding_box: u32,
}

impl TetrominoMeta {
    pub fn wall_kicks(&self, from_rotation: Rotation, to_rotation: Rotation) -> Vec<Point> {
        let offsets_from = self.shape.offsets(&from_rotation);
        let offsets_to = self.shape.offsets(&to_rotation);

        let result = offsets_from
            .iter()
            .zip(offsets_to.iter())
            .map(move |(a, b)| Point::new(a.0 - b.0, a.1 - b.1))
            .collect::<Vec<Point>>();

        if self.shape == TetrominoShape::O {
            vec![result[0]]
        } else {
            result
        }
    }

    pub fn rotated_minos(&self, rotation: Rotation) -> Minos {
        if rotation == Rotation::North {
            return self.minos;
        }

        let center = (self.bounding_box as i32 - 1) / 2;
        let to_origin = -center;

        let (rotations, clockwise) = match rotation {
            Rotation::North => (0, true),
            Rotation::East => (1, true),
            Rotation::South => (2, true),
            Rotation::West => (1, false),
        };

        self.minos
            .iter()
            .map(|p| {
                let mut result = p.translate(to_origin, to_origin);
                for _ in 0..rotations {
                    result = result.rotate(clockwise);
                }
                result.translate(center, center)
            })
            .collect::<Vec<Point>>()
            .try_into()
            .unwrap()
    }

    pub fn normal_minos(&self) -> Minos {
        let normal_offset = Point::new(
            self.minos.iter().map(|p| p.x).min().unwrap(),
            self.minos.iter().map(|p| p.y).min().unwrap()
        );
        return self.minos.map(|p| p - normal_offset)
    }

    pub fn minos(&self) -> Minos {
        self.minos
    }

    pub fn perimeter(&self) -> MinoPerimeter {
        self.perimeter
    }

    pub fn outside_corners(&self) -> MinoCorners {
        self.outside_corners
    }
}

const I: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::I,
    spawn_point: Point::new(2, 18),
    minos: [
        Point::new(1, 2),
        Point::new(2, 2),
        Point::new(3, 2),
        Point::new(4, 2),
    ],
    perimeter: perimeter([
        Perimeter::Top.bits() | Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Bottom.bits(),
    ]),
    outside_corners: [Corner::None, Corner::None, Corner::None, Corner::None],
    bounding_box: 5,
};

const J: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::J,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(0, 1),
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(0, 2),
    ],
    perimeter: perimeter([
        Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Left.bits(),
    ]),
    outside_corners: [Corner::TopRight, Corner::None, Corner::None, Corner::None],
    bounding_box: 3,
};

const L: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::L,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(0, 1),
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(2, 2),
    ],
    perimeter: perimeter([
        Perimeter::Top.bits() | Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Bottom.bits(),
        Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Left.bits(),
    ]),
    outside_corners: [Corner::None, Corner::None, Corner::TopLeft, Corner::None],
    bounding_box: 3,
};

const O: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::O,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(1, 2),
        Point::new(2, 2),
    ],
    perimeter: perimeter([
        Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits(),
    ]),
    outside_corners: [Corner::None, Corner::None, Corner::None, Corner::None],
    bounding_box: 3,
};

const S: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::S,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(0, 1),
        Point::new(1, 1),
        Point::new(1, 2),
        Point::new(2, 2),
    ],
    perimeter: perimeter([
        Perimeter::Top.bits() | Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Bottom.bits(),
    ]),
    outside_corners: [Corner::None, Corner::TopLeft, Corner::BottomRight, Corner::None],
    bounding_box: 3,
};

const T: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::T,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(0, 1),
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(1, 2),
    ],
    perimeter: perimeter([
        Perimeter::Top.bits() | Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Left.bits(),
    ]),
    outside_corners: [Corner::None, Corner::from_bits_truncate(Corner::TopLeft.bits() | Corner::TopRight.bits()), Corner::None, Corner::None],
    bounding_box: 3,
};

const Z: TetrominoMeta = TetrominoMeta {
    shape: TetrominoShape::Z,
    spawn_point: Point::new(3, 19),
    minos: [
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(0, 2),
        Point::new(1, 2),
    ],
    perimeter: perimeter([
        Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits() | Perimeter::Bottom.bits(),
        Perimeter::Top.bits() | Perimeter::Bottom.bits() | Perimeter::Left.bits(),
        Perimeter::Top.bits() | Perimeter::Right.bits(),
    ]),
    outside_corners: [Corner::TopRight, Corner::None, Corner::None, Corner::BottomLeft],
    bounding_box: 3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tetromino {
    meta: TetrominoMeta,
    position: Point,
    rotation: Rotation,
    minos: Minos,
    lock_placements: u32,
    y_min: i32,
}

impl Tetromino {
    pub fn new(shape: TetrominoShape) -> Self {
        let meta = shape.meta();
        Self {
            meta: *meta,
            position: meta.spawn_point,
            rotation: Rotation::North,
            minos: meta
                .rotated_minos(Rotation::North)
                .map(|p| p + meta.spawn_point),
            lock_placements: 0,
            y_min: meta.spawn_point.y,
        }
    }

    pub fn shape(&self) -> TetrominoShape {
        self.meta.shape
    }

    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    pub fn minos(&self) -> Minos {
        self.minos
    }

    pub fn translate(&mut self, x: i32, y: i32) {
        self.translate_point(Point::new(x, y));
    }

    pub fn possible_minos_after_rotation(&self, clockwise: bool) -> Vec<Minos> {
        let to_rotation = self.rotation.rotate(clockwise);
        let basic_rotation_minos = self.meta.rotated_minos(to_rotation);
        return self
            .meta
            .wall_kicks(self.rotation, to_rotation)
            .iter()
            .map(|kick| basic_rotation_minos.map(|p| p + self.position + *kick))
            .collect::<Vec<Minos>>();
    }

    pub fn rotate(&mut self, clockwise: bool, wall_kick_id: usize) {
        let to_rotation = self.rotation.rotate(clockwise);
        let wall_kick = self.meta.wall_kicks(self.rotation, to_rotation)[wall_kick_id];
        self.rotation = to_rotation;
        self.translate_point(wall_kick);
    }

    fn translate_point(&mut self, p: Point) {
        self.position += p;
        self.minos = self
            .meta
            .rotated_minos(self.rotation)
            .map(|p| p + self.position);
        if self.position.y < self.y_min {
            self.y_min = self.position.y;
            // lock placements are reset every time a tetromino falls
            self.lock_placements = 0;
        }
    }

    pub fn register_lock_placement(&mut self) -> u32 {
        self.lock_placements += 1;
        self.lock_placements
    }

    pub fn lock_placements(&self) -> u32 {
        self.lock_placements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_kicks_j() {
        assert_eq!(
            TetrominoShape::J
                .meta()
                .wall_kicks(Rotation::North, Rotation::East),
            [
                Point::new(0, 0),
                Point::new(-1, 0),
                Point::new(-1, 1),
                Point::new(0, -2),
                Point::new(-1, -2)
            ]
        );
    }

    #[test]
    fn clockwise_rotation_with_wall_kicks() {
        let tetromino = Tetromino::new(TetrominoShape::J);
        let observed = tetromino.possible_minos_after_rotation(true);

        assert_eq!(
            observed,
            vec![
                [
                    Point::new(4, 21),
                    Point::new(4, 20),
                    Point::new(4, 19),
                    Point::new(5, 21)
                ],
                [
                    Point::new(3, 21),
                    Point::new(3, 20),
                    Point::new(3, 19),
                    Point::new(4, 21)
                ],
                [
                    Point::new(3, 22),
                    Point::new(3, 21),
                    Point::new(3, 20),
                    Point::new(4, 22)
                ],
                [
                    Point::new(4, 19),
                    Point::new(4, 18),
                    Point::new(4, 17),
                    Point::new(5, 19)
                ],
                [
                    Point::new(3, 19),
                    Point::new(3, 18),
                    Point::new(3, 17),
                    Point::new(4, 19)
                ]
            ]
        );
    }

    #[test]
    fn minos() {
        let tetromino = Tetromino::new(TetrominoShape::L);
        assert_eq!(
            tetromino.minos(),
            [
                Point::new(3, 20),
                Point::new(4, 20),
                Point::new(5, 20),
                Point::new(5, 21)
            ]
        );
    }

    #[test]
    fn translate() {
        let mut tetromino = Tetromino::new(TetrominoShape::L);
        tetromino.translate(1, -1);
        assert_eq!(
            tetromino.minos(),
            [
                Point::new(4, 19),
                Point::new(5, 19),
                Point::new(6, 19),
                Point::new(6, 20)
            ]
        );
    }

    #[test]
    fn lock_placements_initial() {
        let tetromino = Tetromino::new(TetrominoShape::L);
        assert_eq!(tetromino.lock_placements(), 0);
    }

    #[test]
    fn register_lock_placement() {
        let mut tetromino = Tetromino::new(TetrominoShape::L);
        assert_eq!(tetromino.register_lock_placement(), 1);
    }

    #[test]
    fn lock_placement_reset_on_translation_below_y_min() {
        let mut tetromino = Tetromino::new(TetrominoShape::L);
        tetromino.register_lock_placement();
        tetromino.translate(0, -1);
        assert_eq!(tetromino.lock_placements(), 0);
    }

    #[test]
    fn lock_placement_not_reset_on_translation_above_y_min() {
        let mut tetromino = Tetromino::new(TetrominoShape::L);
        tetromino.translate(0, -2); // set y_min = -2
        tetromino.translate(0, 1); // translate above y_min
        tetromino.register_lock_placement();
        tetromino.translate(0, -1); // translate back to but not below y_min
        assert_eq!(tetromino.lock_placements(), 1);
    }

    #[test]
    fn lock_placement_not_reset_on_x_translation() {
        let mut tetromino = Tetromino::new(TetrominoShape::L);
        tetromino.register_lock_placement();
        tetromino.translate(1, 0);
        assert_eq!(tetromino.lock_placements(), 1);
    }

    #[test]
    fn normal_minos() {
        assert_eq!(TetrominoShape::I.meta().normal_minos(), [
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(2, 0),
            Point::new(3, 0),
        ]);
    }
}
