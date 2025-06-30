use std::fmt::{Display, Formatter};
use crate::game::ai::coefficient::Coefficient;
use crate::game::ai::genome::LinearGenome;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinearCoefficients {
    open_holes: Coefficient,
    closed_holes: Coefficient,
    max_stack_height: Coefficient,
    sum_stack_roughness: Coefficient,
    max_stack_roughness: Coefficient,
    line_clear: Coefficient,
    tetris_clear: Coefficient,
    max_tetromino_y: Coefficient,
    pillars: Coefficient,
    hole_cover: Coefficient,
}

impl Default for LinearCoefficients {
    fn default() -> Self {
        LinearCoefficients::from_f64(
            -89.32,
            -104.13,
            -10.12,
            -7.96,
            -7.19,
            -25.65,
            18.87,
            -4.49,
            -57.45,
            0.0,
        )
    }
}

impl Display for LinearCoefficients {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
               self.open_holes, self.closed_holes, self.max_stack_height, self.sum_stack_roughness, self.max_stack_roughness, self.line_clear, self.tetris_clear, self.max_tetromino_y, self.pillars, self.hole_cover)
    }
}

impl LinearCoefficients {
    pub const ZERO: Self = Self {
        open_holes: Coefficient::ZERO,
        closed_holes: Coefficient::ZERO,
        max_stack_height: Coefficient::ZERO,
        sum_stack_roughness: Coefficient::ZERO,
        max_stack_roughness: Coefficient::ZERO,
        line_clear: Coefficient::ZERO,
        tetris_clear: Coefficient::ZERO,
        max_tetromino_y: Coefficient::ZERO,
        pillars: Coefficient::ZERO,
        hole_cover: Coefficient::ZERO,
    };

    pub fn from_f64(
        open_holes: f64,
        closed_holes: f64,
        max_stack_height: f64,
        sum_delta_stack_height: f64,
        max_delta_stack_height: f64,
        line_clear: f64,
        tetris_clear: f64,
        max_tetromino_y: f64,
        pillars: f64,
        hole_cover: f64
    ) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_stack_roughness: sum_delta_stack_height.into(),
            max_stack_roughness: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
            max_tetromino_y: max_tetromino_y.into(),
            pillars: pillars.into(),
            hole_cover: hole_cover.into(),
        }
    }

    pub fn from_i64(
        open_holes: i64,
        closed_holes: i64,
        max_stack_height: i64,
        sum_delta_stack_height: i64,
        max_delta_stack_height: i64,
        line_clear: i64,
        tetris_clear: i64,
        max_tetromino_y: i64,
        pillars: i64,
        hole_cover: i64
    ) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_stack_roughness: sum_delta_stack_height.into(),
            max_stack_roughness: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
            max_tetromino_y: max_tetromino_y.into(),
            pillars: pillars.into(),
            hole_cover: hole_cover.into(),
        }
    }

    pub fn open_holes(&self) -> Coefficient {
        self.open_holes
    }

    pub fn closed_holes(&self) -> Coefficient {
        self.closed_holes
    }

    pub fn max_stack_height(&self) -> Coefficient {
        self.max_stack_height
    }

    pub fn sum_stack_roughness(&self) -> Coefficient {
        self.sum_stack_roughness
    }

    pub fn max_stack_roughness(&self) -> Coefficient {
        self.max_stack_roughness
    }

    pub fn line_clear(&self) -> Coefficient {
        self.line_clear
    }

    pub fn tetris_clear(&self) -> Coefficient {
        self.tetris_clear
    }

    pub fn max_tetromino_y(&self) -> Coefficient {
        self.max_tetromino_y
    }

    pub fn pillars(&self) -> Coefficient {
        self.pillars
    }

    pub fn hole_cover(&self) -> Coefficient {
        self.hole_cover
    }
}

impl Into<LinearGenome> for LinearCoefficients {
    fn into(self) -> LinearGenome {
        LinearGenome::new(
            [
                self.open_holes,
                self.closed_holes,
                self.max_stack_height,
                self.sum_stack_roughness,
                self.max_stack_roughness,
                self.line_clear,
                self.tetris_clear,
                self.max_tetromino_y,
                self.pillars,
                self.hole_cover
            ]
        )
    }
}

impl From<LinearGenome> for LinearCoefficients {
    fn from(genome: LinearGenome) -> Self {
        let [
            open_holes,
            closed_holes,
            max_stack_height,
            sum_stack_roughness,
            max_stack_roughness,
            line_clear,
            tetris_clear,
            max_tetromino_y,
            pillars,
            hole_cover,
        ] = genome.chromosome();
        Self {
            open_holes,
            closed_holes,
            max_stack_height,
            sum_stack_roughness,
            max_stack_roughness,
            line_clear,
            tetris_clear,
            max_tetromino_y,
            pillars,
            hole_cover,
        }
    }
}