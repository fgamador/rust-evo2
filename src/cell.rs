use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    constants: Rc<CellConstants>,
    params: CellParams,
    state: CellState,
}

impl Cell {
    pub fn new(constants: &Rc<CellConstants>, params: CellParams) -> Self {
        Cell {
            constants: Rc::clone(constants),
            params,
            state: CellState::DEFAULT,
        }
    }

    pub fn with_energy(mut self, energy: f32) -> Self {
        self.state.energy = energy;
        self
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
        child.state.energy = self.params.child_threshold_energy - self.constants.create_child_energy;
        Some(child)
    }

    fn eat(&mut self, food_per_cell: f32) -> f32 {
        self.expend_energy(self.params.attempted_eating_energy);
        (self.params.attempted_eating_energy * self.constants.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: f32) {
        self.state.energy += food_amount * self.constants.energy_yield_from_digestion;
    }

    fn maintain(&mut self) {
        self.expend_energy(self.constants.maintenance_energy_use);
    }

    fn expend_energy(&mut self, energy: f32) {
        self.state.energy -= energy;
        self.state.health -= energy * self.constants.health_reduction_per_energy_used;
        self.state.health = self.state.health.max(0.0);
    }
}

#[derive(Debug, PartialEq)]
pub struct CellConstants {
    pub maintenance_energy_use: f32,
    pub food_yield_from_eating: f32,
    pub energy_yield_from_digestion: f32,
    pub create_child_energy: f32,
    pub health_reduction_per_energy_used: f32,
}

impl CellConstants {
    #[allow(dead_code)]
    pub const DEFAULT: CellConstants = CellConstants {
        maintenance_energy_use: 0.0,
        food_yield_from_eating: 1.0,
        energy_yield_from_digestion: 1.0,
        create_child_energy: 0.0,
        health_reduction_per_energy_used: 0.0,
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
        let cell = Cell::new(&Rc::new(CellConstants::DEFAULT), CellParams::DEFAULT);
        assert_eq!(cell.health(), 1.0);
    }

    #[test]
    fn cell_uses_energy() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 5.25,
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75);
    }

    #[test]
    fn expending_maintenance_energy_reduces_health() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 2.0,
            health_reduction_per_energy_used: 0.125,
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75);
    }

    #[test]
    fn cannot_reduce_health_below_zero() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 2.0,
            health_reduction_per_energy_used: 1.0,
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.0);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let cell = Cell::new(&Rc::new(CellConstants::DEFAULT), CellParams::DEFAULT).with_energy(0.0);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: 1.5,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(1.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 3.0);
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: 1.0,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 3.0,
            ..CellParams::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 2.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(1.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 2.0);
    }

    #[test]
    fn cell_expends_energy_eating() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: 0.0,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(5.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_expends_energy_eating_even_when_there_is_no_food() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: 0.0,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 0.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(5.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_digests_food() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 0.0,
            food_yield_from_eating: 1.0,
            energy_yield_from_digestion: 1.5,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 11.0);
    }

    #[test]
    fn expending_eating_energy_reduces_health() {
        let constants = Rc::new(CellConstants {
            health_reduction_per_energy_used: 0.125,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0,
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75);
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 4.0,
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(3.0);
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
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(1.0);
        let environment = CellEnvironment {
            food_per_cell: 3.0,
            ..CellEnvironment::DEFAULT
        };
        let (child, _) = cell.step(&environment);
        assert_eq!(child, None);
    }

    #[test]
    fn cell_passes_energy_to_child() {
        let constants = Rc::new(CellConstants {
            create_child_energy: 1.5,
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            child_threshold_energy: 4.0,
            child_threshold_food: 0.0,
            attempted_eating_energy: 1.0,
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, Some(Cell {
            constants: Rc::clone(&constants),
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
