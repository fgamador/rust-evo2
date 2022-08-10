use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    bio_constants: Rc<BioConstants>,
    cell_constants: CellConstants,
    health: f32,
    energy: f32,
}

impl Cell {
    pub fn new(bio_constants: &Rc<BioConstants>, energy: f32, child_threshold_energy: f32, child_threshold_food: f32, attempted_eating_energy: f32) -> Self {
        Cell {
            bio_constants: Rc::clone(bio_constants),
            cell_constants: CellConstants {
                child_threshold_energy,
                child_threshold_food,
                attempted_eating_energy,
            },
            health: 1.0,
            energy,
        }
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy() > 0.0
    }

    pub fn step(&mut self, environment: &CellEnvironment) -> (Option<Cell>, f32) {
        let child = self.try_reproduce(environment);
        let food = self.eat(environment.food_per_cell);
        self.digest(food);
        self.maintain();
        (child, food)
    }

    fn try_reproduce(&mut self, environment: &CellEnvironment) -> Option<Cell> {
        if self.energy < self.cell_constants.child_threshold_energy
            || environment.food_per_cell < self.cell_constants.child_threshold_food
        { return None; }

        let mut child = self.clone();
        self.energy -= self.cell_constants.child_threshold_energy;
        child.energy = self.cell_constants.child_threshold_energy - self.bio_constants.create_child_energy;
        Some(child)
    }

    fn eat(&mut self, food_per_cell: f32) -> f32 {
        self.energy -= self.cell_constants.attempted_eating_energy;
        (self.cell_constants.attempted_eating_energy * self.bio_constants.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: f32) {
        self.energy += food_amount * self.bio_constants.energy_yield_from_digestion;
    }

    fn maintain(&mut self) {
        self.energy -= self.bio_constants.maintenance_energy_use;
    }
}

#[derive(Debug, PartialEq)]
pub struct BioConstants {
    pub maintenance_energy_use: f32,
    pub food_yield_from_eating: f32,
    pub energy_yield_from_digestion: f32,
    pub create_child_energy: f32,
}

impl BioConstants {
    #[allow(dead_code)]
    pub const DEFAULT: BioConstants = BioConstants {
        maintenance_energy_use: 0.0,
        food_yield_from_eating: 1.0,
        energy_yield_from_digestion: 1.0,
        create_child_energy: 0.0,
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellConstants {
    child_threshold_energy: f32,
    child_threshold_food: f32,
    attempted_eating_energy: f32,
}

impl CellConstants {
    #[allow(dead_code)]
    pub const DEFAULT: CellConstants = CellConstants {
        child_threshold_energy: f32::MAX,
        child_threshold_food: f32::MAX,
        attempted_eating_energy: 0.0,
    };
}

pub struct CellEnvironment {
    pub food_per_cell: f32,
}

impl CellEnvironment {
    #[allow(dead_code)]
    pub const DEFAULT: CellEnvironment = CellEnvironment { food_per_cell: 0.0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cell_has_full_health() {
        let cell = Cell::new(&Rc::new(BioConstants::DEFAULT), 0.0, f32::MAX, f32::MAX, 0.0);
        assert_eq!(cell.health(), 1.0);
    }

    #[test]
    fn cell_uses_energy() {
        let bio_constants = Rc::new(BioConstants {
            maintenance_energy_use: 5.25,
            ..BioConstants::DEFAULT
        });
        let mut cell = Cell::new(&bio_constants, 10.0, f32::MAX, f32::MAX, 0.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let cell = Cell::new(&Rc::new(BioConstants::DEFAULT), 0.0, f32::MAX, f32::MAX, 0.0);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 1.5,
            ..BioConstants::DEFAULT
        });
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, 1.0, f32::MAX, f32::MAX, 2.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 3.0);
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 1.0,
            ..BioConstants::DEFAULT
        });
        let environment = CellEnvironment {
            food_per_cell: 2.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, 1.0, f32::MAX, f32::MAX, 3.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 2.0);
    }

    #[test]
    fn cell_expends_energy_eating() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 0.0,
            ..BioConstants::DEFAULT
        });
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, 5.0, f32::MAX, f32::MAX, 2.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_expends_energy_eating_even_when_there_is_no_food() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 0.0,
            ..BioConstants::DEFAULT
        });
        let environment = CellEnvironment {
            food_per_cell: 0.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, 5.0, f32::MAX, f32::MAX, 2.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_digests_food() {
        let bio_constants = Rc::new(BioConstants {
            maintenance_energy_use: 0.0,
            food_yield_from_eating: 1.0,
            energy_yield_from_digestion: 1.5,
            ..BioConstants::DEFAULT
        });
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, 10.0, f32::MAX, f32::MAX, 2.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 11.0);
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let mut cell = Cell::new(&Rc::new(BioConstants::DEFAULT), 3.0, 4.0, f32::MAX, 0.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, None);
    }

    #[test]
    fn cell_with_insufficient_food_does_not_reproduce() {
        let mut cell = Cell::new(&Rc::new(BioConstants::DEFAULT), 1.0, 1.0, 4.0, 0.0);
        let environment = CellEnvironment {
            food_per_cell: 3.0,
            ..CellEnvironment::DEFAULT
        };
        let (child, _) = cell.step(&environment);
        assert_eq!(child, None);
    }

    #[test]
    fn cell_passes_energy_to_child() {
        let bio_constants = Rc::new(BioConstants {
            create_child_energy: 1.5,
            ..BioConstants::DEFAULT
        });
        let mut cell = Cell::new(&bio_constants, 10.0, 4.0, 0.0, 1.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, Some(Cell {
            bio_constants: Rc::clone(&bio_constants),
            cell_constants: CellConstants {
                child_threshold_energy: 4.0,
                child_threshold_food: 0.0,
                attempted_eating_energy: 1.0,
            },
            health: 1.0,
            energy: 2.5,
        }));
        assert_eq!(5.0, cell.energy());
    }
}
