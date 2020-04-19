use crate::GRID_SIDE;

use crate::agent::Id;
use crate::rand::Rng;

use std::cmp;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn random() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            x: rng.gen_range(0, GRID_SIDE),
            y: rng.gen_range(0, GRID_SIDE),
        }
    }
}

#[derive(Copy, Clone)]
pub struct PositionChange {
    pub id: Id,
    pub before: Position,
    pub after: Position,
}

/// Provides 2D grid and subgrid agent detection
pub struct Grid {
    pub val: Vec<Vec<Id>>,
}

impl Grid {
    pub fn set(&mut self, position: Position, id: Id) {
        self.val[position.x][position.y] = id;
    }

    pub fn update(&mut self, changes: Vec<PositionChange>) {
        for change in &changes {
            self.set(change.before, 0);
            self.set(change.after, change.id);
        }
    }

    pub fn is_subgrid_free(
        &self,
        position: Position,
        subgrid_size_x: usize,
        subgrid_size_y: usize,
        excluded_ids: Vec<Id>,
        maybe_excluded_fn: Option<&dyn Fn(Id) -> bool>,
    ) -> bool {
        let subgrid_center: Position = self.get_subgrid_center(subgrid_size_x, subgrid_size_y);
        let mut occupier_iter: SubgridSearch = SubgridSearch::new(
            position,
            subgrid_center,
            subgrid_size_x,
            subgrid_size_y,
            excluded_ids,
            &self.val,
        );

        if let Some(excluded_fn) = maybe_excluded_fn {
            while let Some(occupier) = occupier_iter.next() {
                if !excluded_fn(occupier) {
                    return false;
                }
            }
            return true;
        } else {
            return occupier_iter.next().is_none();
        }
    }

    pub fn is_subgrid_occupied(
        &self,
        position: Position,
        subgrid_size_x: usize,
        subgrid_size_y: usize,
        excluded_ids: Vec<Id>,
        maybe_excluded_fn: Option<&dyn Fn(Id) -> bool>,
    ) -> bool {
        let subgrid_center: Position = self.get_subgrid_center(subgrid_size_x, subgrid_size_y);
        let mut occupier_iter: SubgridSearch = SubgridSearch::new(
            position,
            subgrid_center,
            subgrid_size_x,
            subgrid_size_y,
            excluded_ids,
            &self.val,
        );

        if let Some(excluded_fn) = maybe_excluded_fn {
            while let Some(occupier) = occupier_iter.next() {
                if !excluded_fn(occupier) {
                    return true;
                }
            }
            return false;
        } else {
            occupier_iter.next().is_some()
        }
    }

    pub fn get_subgrid_occupiers(
        &self,
        position: Position,
        subgrid_size_x: usize,
        subgrid_size_y: usize,
        excluded_ids: Vec<Id>,
        maybe_excluded_fn: Option<&dyn Fn(Id) -> bool>,
    ) -> Vec<Id> {
        let subgrid_center: Position = self.get_subgrid_center(subgrid_size_x, subgrid_size_y);
        let mut occupier_iter: SubgridSearch = SubgridSearch::new(
            position,
            subgrid_center,
            subgrid_size_x,
            subgrid_size_y,
            excluded_ids,
            &self.val,
        );

        if let Some(excluded_fn) = maybe_excluded_fn {
            let mut v: Vec<Id> = vec![];
            while let Some(occupier) = occupier_iter.next() {
                if !excluded_fn(occupier) {
                    v.push(occupier);
                }
            }
            v
        } else {
            occupier_iter.collect()
        }
    }

    fn get_subgrid_center(&self, subgrid_size_x: usize, subgrid_size_y: usize) -> Position {
        Position {
            x: ((subgrid_size_x - 1) as f32 / 2.0).round() as usize,
            y: ((subgrid_size_y - 1) as f32 / 2.0).round() as usize,
        }
    }
}

struct SubgridSearch<'a> {
    x_start: usize,
    y_start: usize,
    y_start_: usize,
    x_end: usize,
    y_end: usize,
    ignore: Vec<Id>,
    val: &'a Vec<Vec<Id>>,
}

impl<'a> SubgridSearch<'a> {
    fn new(
        position: Position,
        subgrid_center: Position,
        subgrid_size_x: usize,
        subgrid_size_y: usize,
        ignore: Vec<Id>,
        val: &Vec<Vec<Id>>,
    ) -> SubgridSearch {
        let x = position.x as i64 - subgrid_center.x as i64;
        let y = position.y as i64 - subgrid_center.y as i64;
        let x_start = cmp::max(x, 0) as usize;
        let y_start = cmp::max(y, 0) as usize;
        let y_start_ = y_start;
        let x_end = cmp::min(subgrid_size_x as i64 + x, GRID_SIDE as i64) as usize;
        let y_end = cmp::min(subgrid_size_y as i64 + y, GRID_SIDE as i64) as usize;
        SubgridSearch {
            x_start,
            y_start,
            y_start_,
            x_end,
            y_end,
            ignore,
            val,
        }
    }
}

impl<'a> Iterator for SubgridSearch<'a> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if self.x_start >= self.x_end {
            None
        } else {
            if self.y_start != self.y_start_ {
                let x = self.x_start;
                for y in self.y_start_..self.y_end {
                    let val: Id = self.val[x][y];
                    if val > 0 && !self.ignore.contains(&val) {
                        self.x_start = if y == self.y_end - 1 { x + 1 } else { x };
                        self.y_start_ = if y < self.y_end - 1 {
                            y + 1
                        } else {
                            self.y_start
                        };
                        return Some(val);
                    }
                }
                self.x_start += 1;
                self.y_start_ = self.y_start;
            }

            for x in self.x_start..self.x_end {
                for y in self.y_start..self.y_end {
                    let val: Id = self.val[x][y];
                    if val > 0 && !self.ignore.contains(&val) {
                        self.x_start = if y == self.y_end - 1 { x + 1 } else { x };
                        self.y_start_ = if y < self.y_end - 1 {
                            y + 1
                        } else {
                            self.y_start
                        };
                        return Some(val);
                    }
                }
            }
            self.x_start = self.x_end;
            None
        }
    }
}
