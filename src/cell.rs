use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    bio_constants: Rc<BioConstants>,
    params: CellParams,
    state: CellState,
}

impl Cell {
    pub fn new(bio_constants: &Rc<BioConstants>, params: CellParams, state: CellState) -> Self {
        Cell {
            bio_constants: Rc::clone(bio_constants),
            params,
            state,
        }
    }

    pub fn health(&self) -> f32 {
        self.state.health
    }

    pub fn energy(&self) -> f32 {
        self.state.energy
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
        if self.state.energy < self.params.child_threshold_energy
            || environment.food_per_cell < self.params.child_threshold_food
        { return None; }

        let mut child = self.clone();
        self.state.energy -= self.params.child_threshold_energy;
        child.state.energy = self.params.child_threshold_energy - self.bio_constants.create_child_energy;
        Some(child)
    }

    fn eat(&mut self, food_per_cell: f32) -> f32 {
        self.state.energy -= self.params.attempted_eating_energy;
        (self.params.attempted_eating_energy * self.bio_constants.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: f32) {
        self.state.energy += food_amount * self.bio_constants.energy_yield_from_digestion;
    }

    fn maintain(&mut self) {
        self.state.energy -= self.bio_constants.maintenance_energy_use;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellParams {
    pub child_threshold_energy: f32,
    pub child_threshold_food: f32,
    pub attempted_eating_energy: f32,
}

impl CellParams {
    #[allow(dead_code)]
    pub const DEFAULT: CellParams = CellParams {
        child_threshold_energy: f32::MAX,
        child_threshold_food: f32::MAX,
        attempted_eating_energy: 0.0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellState {
    pub health: f32,
    pub energy: f32,
}

impl CellState {
    #[allow(dead_code)]
    pub const DEFAULT: CellState = CellState {
        health: 1.0,
        energy: 0.0,
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
        let cell = Cell::new(&Rc::new(BioConstants::DEFAULT), CellParams::DEFAULT, CellState::DEFAULT);
        assert_eq!(cell.health(), 1.0);
    }

    #[test]
    fn cell_uses_energy() {
        let bio_constants = Rc::new(BioConstants {
            maintenance_energy_use: 5.25,
            ..BioConstants::DEFAULT
        });
        let state = CellState {
            energy: 10.0,
            ..CellState::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, CellParams::DEFAULT, state);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let state = CellState {
            energy: 0.0,
            ..CellState::DEFAULT
        };
        let cell = Cell::new(&Rc::new(BioConstants::DEFAULT), CellParams::DEFAULT, state);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 1.5,
            ..BioConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 1.0,
            ..CellState::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 3.0);
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 1.0,
            ..BioConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 3.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 1.0,
            ..CellState::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 2.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 2.0);
    }

    #[test]
    fn cell_expends_energy_eating() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 0.0,
            ..BioConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 5.0,
            ..CellState::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_expends_energy_eating_even_when_there_is_no_food() {
        let bio_constants = Rc::new(BioConstants {
            food_yield_from_eating: 0.0,
            ..BioConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 5.0,
            ..CellState::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 0.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
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
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 10.0,
            ..CellState::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
        cell.step(&environment);
        assert_eq!(cell.energy(), 11.0);
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 4.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 3.0,
            ..CellState::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(BioConstants::DEFAULT), params, state);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, None);
    }

    #[test]
    fn cell_with_insufficient_food_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 1.0,
            child_threshold_food: 4.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 1.0,
            ..CellState::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(BioConstants::DEFAULT), params, state);
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
        let params = CellParams {
            child_threshold_energy: 4.0,
            child_threshold_food: 0.0,
            attempted_eating_energy: 1.0,
            ..CellParams::DEFAULT
        };
        let state = CellState {
            energy: 10.0,
            ..CellState::DEFAULT
        };
        let mut cell = Cell::new(&bio_constants, params, state);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, Some(Cell {
            bio_constants: Rc::clone(&bio_constants),
            params: CellParams {
                child_threshold_energy: 4.0,
                child_threshold_food: 0.0,
                attempted_eating_energy: 1.0,
            },
            state: CellState {
                health: 1.0,
                energy: 2.5,
            },
        }));
        assert_eq!(5.0, cell.energy());
    }
}
