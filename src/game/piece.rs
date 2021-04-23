use crate::game::piece_types::{DefinitionBlock, PieceFreezeProperty, PieceDefinition};
use crate::game::tootris::{BlockColor, Point, Orientation, Rotation, GameBlock};
use core::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct Piece {
    definition: Vec<Vec<DefinitionBlock>>,
    pub freeze_property: PieceFreezeProperty,
    pub current_matrix: Vec<Vec<DefinitionBlock>>,
    pub color: BlockColor,
    pub location: Point,
    pub rollback_location: Point,
    pub orientation: Orientation,
    pub rollback_orientation: Orientation,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}:{}\n{})", self.location.x, self.location.y, self.orientation,
               Self::generate_string_representation(&self))
    }
}

impl Piece {
    fn generate_string_representation(s: &Self) -> String {
        let mut output: String = String::new();
        for y in 0..s.current_matrix.len() {
            for x in 0..s.current_matrix[y].len() {
                output.push(s.current_matrix[y][x].get_string_visual());
            }
            output.push('\n');
        }
        output
    }

    fn update_matrix_for(that: &mut Self) {
        match that.orientation {
            Orientation::Normal => that.set_matrix_normal(),
            Orientation::Forward => that.set_matrix_forwards(),
            Orientation::UpsideDown => that.set_matrix_upside_down(),
            Orientation::Backwards => that.set_matrix_backwards(),
        }
    }

    pub fn new(definition: Vec<Vec<DefinitionBlock>>,
               freeze_property: PieceFreezeProperty, color: BlockColor, location: Point)
               -> Self {
        let mut fresh_self = Piece {
            definition,
            freeze_property,
            current_matrix: Vec::new(),
            color,
            location,
            rollback_location: location.clone(),
            orientation: Orientation::Normal,
            rollback_orientation: Orientation::Normal,
        };
        fresh_self.update_current_matrix();

        return fresh_self;
    }

    pub fn of_type(t: &PieceDefinition, color: BlockColor, location: Point) -> Self {
        Self::new(t.def.clone(), t.prop.clone(), color, location)
    }

    pub(crate) fn rotate(&mut self, rot: &Rotation) {
        self.rollback_orientation = self.orientation;
        self.orientation = rot.perform(&self.rollback_orientation);
        self.update_current_matrix();
    }

    pub(crate) fn rollback_rotation(&mut self) {
        self.orientation = self.rollback_orientation;
        self.update_current_matrix();
    }

    pub(crate) fn move_to(&mut self, new_location: Point) {
        self.rollback_location = self.location;
        self.location = new_location;
    }

    pub fn rollback_move(&mut self) {
        self.location = self.rollback_location;
    }

    pub fn update_current_matrix(&mut self) {
        Self::update_matrix_for(self);
    }

    fn set_matrix_normal(&mut self) {
        let mut matrix: Vec<Vec<DefinitionBlock>> =
            vec![vec![DefinitionBlock::Blank; self.definition[0].len()]; self.definition.len()];

        for y in 0..self.definition.len() {
            for x in 0..self.definition[y].len() {
                matrix[y][x] = self.definition[y][x].clone();
            }
        }
        self.current_matrix = matrix;
    }

    fn set_matrix_backwards(&mut self) {
        let mut rotated_matrix: Vec<Vec<DefinitionBlock>> =
            vec![vec![DefinitionBlock::Blank; self.definition.len()]; self.definition[0].len()];
        /*
         * 270 degrees forward, or 90 degrees backwards. This is forward and upside-down combined
         * it is flipped 90 degrees forward and then flipped 180 degrees.
         */
        for y in 0..self.definition.len() {
            for x in 0..self.definition[y].len() {
                rotated_matrix[self.definition[0].len() - 1 - x][y]
                    = self.definition[y][x].clone();
            }
        }
        self.current_matrix = rotated_matrix;
    }
    fn set_matrix_forwards(&mut self) {
        let mut rotated_matrix: Vec<Vec<DefinitionBlock>> =
            vec![vec![DefinitionBlock::Blank; self.definition.len()]; self.definition[0].len()];
        /*
         * Need to rotate the matrix 90 degrees forward.
         * y              y0#
         * 0 ####   ===>   1#
         * x 0123          2#
         *                 3#
         *                 x0
        */
        for y in 0..self.definition.len() {
            for x in 0..self.definition[y].len() {
                rotated_matrix[x][self.definition.len() - 1 - y]
                    = self.definition[y][x].clone();
            }
        }
        self.current_matrix = rotated_matrix;
    }

    fn set_matrix_upside_down(&mut self) {
        let mut rotated_matrix: Vec<Vec<DefinitionBlock>> = Vec::new();
        for _x in 0..self.definition.len() {
            rotated_matrix.push(vec![DefinitionBlock::Blank; self.definition[0].len()]);
        }
        /*
         * Now 180 degrees forward, or upside down
         *  # ==>  ###
         * ### ==>  #
         * Both the y and x vectors should be reversed
         */
        for y in 0..self.definition.len() {
            for x in 0..self.definition[y].len() {
                rotated_matrix[self.definition.len() - 1 - y][self.definition[0].len() - 1 - x]
                    = self.definition[y][x].clone();
            }
        }
        self.current_matrix = rotated_matrix;
    }

    pub fn place_in_matrix<'a>(&self, matrix: &'a mut [Vec<GameBlock>]) -> &'a mut [Vec<GameBlock>] {
        let origin = self.find_origin_for_zero_block(None);
        for y in 0..self.current_matrix.len() {
            for x in 0..self.current_matrix[y].len() {
                if self.current_matrix[y][x] != DefinitionBlock::Blank {
                    matrix[origin.y + y][origin.x + x]
                        = self.colorize_block_erase_origin(self.current_matrix[y][x].clone());
                }
            }
        }
        matrix
    }

    pub fn remove_from_matrix<'a>(&self, matrix: &'a mut [Vec<GameBlock>]) -> &'a mut [Vec<GameBlock>] {
        let origin = self.find_origin_for_zero_block(None);
        for y in 0..self.current_matrix.len() {
            for x in 0..self.current_matrix[y].len() {
                matrix[origin.y + y][origin.x + x] = GameBlock::Empty;
            }
        }
        matrix
    }

    fn colorize_block_erase_origin(&self, block: DefinitionBlock) -> GameBlock {
        return match block {
            DefinitionBlock::Filled => GameBlock::Filled(self.color.clone()),
            DefinitionBlock::Origin => GameBlock::Filled(self.color.clone()),
            DefinitionBlock::Text(val) => GameBlock::String(val.to_string(), self.color.clone()),
            _ => GameBlock::Empty,
        };
    }

    fn find_origin_for_zero_block(&self, point_override: Option<&Point>) -> Point {
        let mut origin: Option<Point> = None;
        'outer: for y in 0..self.current_matrix.len() {
            for x in 0..self.current_matrix[y].len() {
                if self.current_matrix[y][x] == DefinitionBlock::Origin {
                    if point_override.is_none() {
                        origin = Some(Point {
                            x: self.location.x - x,
                            y: self.location.y - y,
                        });
                    } else {
                        origin = Some(Point {
                            x: point_override.unwrap().x - x,
                            y: point_override.unwrap().y - y,
                        });
                    }
                    break 'outer;
                }
            }
        }
        if origin.is_none() {
            return self.location.clone();
        }
        return origin.unwrap();
    }


    pub fn points(&self, point_override: Option<&Point>) -> Vec<Point> {
        let origin = self.find_origin_for_zero_block(point_override);
        let mut boundaries: Vec<Point> = Vec::new();

        for y in 0..self.current_matrix.len() {
            for x in 0..self.current_matrix[y].len() {
                if self.current_matrix[y][x] != DefinitionBlock::Blank {
                    boundaries.push(Point {
                        x: origin.x + x,
                        y: origin.y + y,
                    });
                }
            }
        }
        return boundaries;
    }

    pub(crate) fn find_xboundaries(&self, point_override: Option<&Point>) -> Vec<Point> {
        let origin = self.find_origin_for_zero_block(point_override);
        let mut boundaries: Vec<Point> = Vec::with_capacity(self.current_matrix.len() * 2);

        /*
         * The x boundaries are the first and last block for each value of y
         * So there may be 1 or 2. We scan the matrix for edge blocks
         */
        for y in 0..self.current_matrix.len() {
            let mut first: Option<Point> = None;
            let mut last: Option<Point> = None;

            for x in 0..self.current_matrix[y].len() {
                if self.current_matrix[y][x] == DefinitionBlock::Blank {
                    continue;
                }
                let current_point = Point { x: origin.x + x, y: origin.y + y };
                if first.is_none() {
                    first = Some(current_point);
                    last = Some(current_point);
                } else {
                    last = Some(current_point);
                }
            }
            if first.is_some() {
                boundaries.push(first.unwrap());
            }
            if last.is_some() {
                boundaries.push(last.unwrap());
            }
        }
        return boundaries;
    }

    pub(crate) fn find_yboundaries(&self, point_override: Option<&Point>) -> Vec<Point> {
        /*
         * We only ever need to check for y collision downwards, so we want one block for each x
         * The highest y value for each x
         */
        let origin = self.find_origin_for_zero_block(point_override);
        let mut boundaries: Vec<Point> = Vec::with_capacity(self.current_matrix[0].len());

        //Assuming a uniform square matrix
        for x in 0..self.current_matrix[0].len() {
            for y in self.current_matrix.len() - 1..=0 {
                if self.current_matrix[y][x] != DefinitionBlock::Blank {
                    boundaries.push(Point {
                        x: origin.x + x,
                        y: origin.y + y,
                    });
                    break;
                }
            }
        }
        return boundaries;
    }
}
