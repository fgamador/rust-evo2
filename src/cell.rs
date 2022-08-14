use std::rc::Rc;
use crate::number_types::{F32Positive, F32ZeroToOne};

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

    pub fn with_health(mut self, health: f32) -> Self {
        self.state.health = health;
        self
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
        self.heal();
        (child, food)
    }

    fn try_reproduce(&mut self, environment: &CellEnvironment) -> Option<Cell> {
        if self.state.energy < self.params.child_threshold_energy.value()
            || environment.food_per_cell < self.params.child_threshold_food.value()
        { return None; }

        let mut child = self.clone();
        self.expend_energy(self.params.child_threshold_energy.value());
        child.state.energy = self.params.child_threshold_energy.value() - self.constants.create_child_energy.value();
        Some(child)
    }

    fn eat(&mut self, food_per_cell: f32) -> f32 {
        self.expend_energy(self.params.attempted_eating_energy.value());
        (self.params.attempted_eating_energy.value() * self.constants.food_yield_from_eating.value()).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: f32) {
        self.state.energy += food_amount * self.constants.energy_yield_from_digestion.value();
    }

    fn maintain(&mut self) {
        self.expend_energy(self.constants.maintenance_energy_use.into());
    }

    fn heal(&mut self) {
        self.state.health += self.params.attempted_healing_energy.value() * self.constants.health_increase_per_healing_energy.value();
    }

    fn expend_energy(&mut self, energy: f32) {
        self.state.energy -= energy;
        self.state.energy = self.state.energy.max(0.0);
        self.state.health -= energy * self.constants.health_reduction_per_energy_expended.value();
        self.state.health = self.state.health.max(0.0);
    }
}

#[derive(Debug, PartialEq)]
pub struct CellConstants {
    pub maintenance_energy_use: F32Positive,
    pub food_yield_from_eating: F32Positive,
    pub energy_yield_from_digestion: F32Positive,
    pub create_child_energy: F32Positive,
    pub health_reduction_per_energy_expended: F32ZeroToOne,
    pub health_increase_per_healing_energy: F32ZeroToOne,
}

impl CellConstants {
    #[allow(dead_code)]
    pub const DEFAULT: CellConstants = CellConstants {
        maintenance_energy_use: F32Positive::unchecked(0.0),
        food_yield_from_eating: F32Positive::unchecked(1.0),
        energy_yield_from_digestion: F32Positive::unchecked(1.0),
        create_child_energy: F32Positive::unchecked(0.0),
        health_reduction_per_energy_expended: F32ZeroToOne::unchecked(0.0),
        health_increase_per_healing_energy: F32ZeroToOne::unchecked(0.0),
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellParams {
    pub child_threshold_energy: F32Positive,
    pub child_threshold_food: F32Positive,
    pub attempted_eating_energy: F32Positive,
    pub attempted_healing_energy: F32Positive,
}

impl CellParams {
    #[allow(dead_code)]
    pub const DEFAULT: CellParams = CellParams {
        child_threshold_energy: F32Positive::unchecked(f32::MAX),
        child_threshold_food: F32Positive::unchecked(f32::MAX),
        attempted_eating_energy: F32Positive::unchecked(0.0),
        attempted_healing_energy: F32Positive::unchecked(0.0),
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
            maintenance_energy_use: 5.25.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75);
    }

    #[test]
    fn cell_cannot_expend_energy_below_zero() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 11.0.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 0.0);
    }

    #[test]
    fn expending_maintenance_energy_reduces_health() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 2.0.into(),
            health_reduction_per_energy_expended: 0.125.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let cell = Cell::new(&Rc::new(CellConstants::DEFAULT), CellParams::DEFAULT).with_energy(0.0);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: 1.5.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
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
            food_yield_from_eating: 1.0.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 3.0.into(),
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
            food_yield_from_eating: 0.0.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
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
            food_yield_from_eating: 0.0.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
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
            maintenance_energy_use: 0.0.into(),
            food_yield_from_eating: 1.0.into(),
            energy_yield_from_digestion: 1.5.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
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
            health_reduction_per_energy_expended: 0.125.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75);
    }

    #[test]
    fn expending_eating_energy_cannot_reduce_health_below_zero() {
        let constants = Rc::new(CellConstants {
            health_reduction_per_energy_expended: 1.0.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_eating_energy: 2.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.0);
    }

    #[test]
    fn cell_can_heal() {
        let constants = Rc::new(CellConstants {
            health_increase_per_healing_energy: 0.25.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            attempted_healing_energy: 1.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_health(0.5).with_energy(10.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75);
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 4.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(3.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, None);
    }

    #[test]
    fn cell_with_insufficient_food_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 1.0.into(),
            child_threshold_food: 4.0.into(),
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
            create_child_energy: 1.5.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            child_threshold_energy: 4.0.into(),
            child_threshold_food: 0.0.into(),
            attempted_eating_energy: 1.0.into(),
            attempted_healing_energy: 1.5.into(),
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(child, Some(Cell {
            constants: Rc::clone(&constants),
            params: CellParams {
                child_threshold_energy: 4.0.into(),
                child_threshold_food: 0.0.into(),
                attempted_eating_energy: 1.0.into(),
                attempted_healing_energy: 1.5.into(),
            },
            state: CellState {
                health: 1.0,
                energy: 2.5,
            },
        }));
        assert_eq!(5.0, cell.energy());
    }

    #[test]
    fn expending_reproduction_energy_reduces_health() {
        let constants = Rc::new(CellConstants {
            create_child_energy: 0.0.into(),
            health_reduction_per_energy_expended: 0.125.into(),
            ..CellConstants::DEFAULT
        });
        let params = CellParams {
            child_threshold_energy: 2.0.into(),
            child_threshold_food: 0.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0);
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_ne!(child, None);
        assert_eq!(cell.health(), 0.75);
    }
}
