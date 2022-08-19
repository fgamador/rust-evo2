use std::rc::Rc;
use crate::number_types::{F32Positive, F32ZeroToOne, F32ZeroToOnePerF32Positive};

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

    pub fn with_health(mut self, health: F32ZeroToOne) -> Self {
        self.state.health = health;
        self
    }

    pub fn with_energy(mut self, energy: F32Positive) -> Self {
        self.state.energy = energy;
        self
    }

    pub fn health(&self) -> F32ZeroToOne {
        self.state.health
    }

    pub fn energy(&self) -> F32Positive {
        self.state.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy().value() > 0.0
    }

    pub fn step(&mut self, environment: &CellEnvironment) -> (Option<Cell>, F32Positive) {
        let [reproduction_energy, eating_energy, maintenance_energy, healing_energy ] =
            Self::budget(self.state.energy,
                         &[self.params.child_threshold_energy,
                             self.params.attempted_eating_energy,
                             self.constants.maintenance_energy_use,
                             self.params.attempted_healing_energy]);

        let child = self.try_reproduce(reproduction_energy, environment);

        self.expend_energy(eating_energy);
        let food = self.eat(eating_energy, environment.food_per_cell);
        self.digest(food);

        self.expend_energy(maintenance_energy);
        self.maintenance(maintenance_energy);

        self.expend_energy(healing_energy);
        self.heal(healing_energy);

        (child, food)
    }

    fn budget<const N: usize>(_available: F32Positive, desired: &[F32Positive; N]) -> [F32Positive; N] {
        *desired
    }

    fn try_reproduce(&mut self, reproduction_energy: F32Positive, environment: &CellEnvironment) -> Option<Cell> {
        if self.cannot_reproduce(reproduction_energy, environment)
        { return None; }

        self.reproduce(reproduction_energy)
    }

    fn cannot_reproduce(&self, reproduction_energy: F32Positive, environment: &CellEnvironment) -> bool {
        self.state.energy < reproduction_energy
            || environment.food_per_cell < self.params.child_threshold_food
    }

    fn reproduce(&mut self, reproduction_energy: F32Positive) -> Option<Cell> {
        let mut child = self.clone();
        self.expend_energy(reproduction_energy);
        child.state.health = 1.0.into();
        child.state.energy = reproduction_energy - self.constants.create_child_energy;
        Some(child)
    }

    fn eat(&mut self, eating_energy: F32Positive, food_per_cell: F32Positive) -> F32Positive {
        (eating_energy * self.constants.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: F32Positive) {
        self.state.energy += food_amount * self.constants.energy_yield_from_digestion;
    }

    fn maintenance(&mut self, _maintenance_energy: F32Positive) {}

    fn heal(&mut self, healing_energy: F32Positive) {
        self.state.health += healing_energy * self.constants.health_increase_per_healing_energy;
    }

    fn expend_energy(&mut self, energy: F32Positive) {
        self.state.energy -= energy;
        self.state.health -= energy * self.constants.health_reduction_per_energy_expended;
    }
}

#[derive(Debug, PartialEq)]
pub struct CellConstants {
    pub maintenance_energy_use: F32Positive,
    pub food_yield_from_eating: F32Positive,
    pub energy_yield_from_digestion: F32Positive,
    pub create_child_energy: F32Positive,
    pub health_reduction_per_energy_expended: F32ZeroToOnePerF32Positive,
    pub health_increase_per_healing_energy: F32ZeroToOnePerF32Positive,
}

impl CellConstants {
    #[allow(dead_code)]
    pub const DEFAULT: CellConstants = CellConstants {
        maintenance_energy_use: F32Positive::unchecked(0.0),
        food_yield_from_eating: F32Positive::unchecked(1.0),
        energy_yield_from_digestion: F32Positive::unchecked(1.0),
        create_child_energy: F32Positive::unchecked(0.0),
        health_reduction_per_energy_expended: F32ZeroToOnePerF32Positive::unchecked(0.0),
        health_increase_per_healing_energy: F32ZeroToOnePerF32Positive::unchecked(0.0),
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
    pub health: F32ZeroToOne,
    pub energy: F32Positive,
}

impl CellState {
    #[allow(dead_code)]
    pub const DEFAULT: CellState = CellState {
        health: F32ZeroToOne::unchecked(1.0),
        energy: F32Positive::unchecked(0.0),
    };
}

pub struct CellEnvironment {
    pub food_per_cell: F32Positive,
}

impl CellEnvironment {
    #[allow(dead_code)]
    pub const DEFAULT: CellEnvironment = CellEnvironment { food_per_cell: F32Positive::unchecked(0.0) };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget<const N: usize>(available: F32Positive, desired: &[F32Positive; N]) -> (F32Positive, [F32Positive; N]) {
        let desired_sum: F32Positive = desired.iter().map(F32Positive::value).sum::<f32>().into();
        if available >= desired_sum {
            return (desired_sum, *desired);
        }

        let reduction_factor = available / desired_sum;
        let mut budgeted = [0.0.into(); N];
        for i in 0..N {
            budgeted[i] = desired[i] * reduction_factor;
        }
        (available, budgeted)
    }

    #[test]
    fn budgeting_adjusts_downward_proportionally() {
        let desired: [F32Positive; 2] = [10.0.into(), 5.0.into()];
        let (used, budgeted) = budget(7.5.into(), &desired);
        assert_eq!(used, 7.5.into());
        assert_eq!(budgeted, [5.0.into(), 2.5.into()]);
    }

    #[test]
    fn budgeting_leaves_satisfiable_requests_unchanged() {
        let desired: [F32Positive; 2] = [10.0.into(), 5.0.into()];
        let (used, budgeted) = budget(20.0.into(), &desired);
        assert_eq!(used, 15.0.into());
        assert_eq!(budgeted, [10.0.into(), 5.0.into()]);
    }

    #[test]
    fn new_cell_has_full_health() {
        let cell = Cell::new(&Rc::new(CellConstants::DEFAULT), CellParams::DEFAULT);
        assert_eq!(cell.health(), 1.0.into());
    }

    #[test]
    fn cell_uses_energy() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 5.25.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75.into());
    }

    #[test]
    fn cell_cannot_expend_energy_below_zero() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 11.0.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 0.0.into());
    }

    #[test]
    fn expending_maintenance_energy_reduces_health() {
        let constants = Rc::new(CellConstants {
            maintenance_energy_use: 2.0.into(),
            health_reduction_per_energy_expended: 0.125.into(),
            ..CellConstants::DEFAULT
        });
        let mut cell = Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let cell = Cell::new(&Rc::new(CellConstants::DEFAULT), CellParams::DEFAULT).with_energy(0.0.into());
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
            food_per_cell: 10.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(1.0.into());
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 3.0.into());
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
            food_per_cell: 2.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(1.0.into());
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 2.0.into());
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
            food_per_cell: 10.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(5.0.into());
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0.into());
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
            food_per_cell: 0.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(5.0.into());
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0.into());
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
            food_per_cell: 10.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0.into());
        cell.step(&environment);
        assert_eq!(cell.energy(), 11.0.into());
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
        let mut cell = Cell::new(&constants, params).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75.into());
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
        let mut cell = Cell::new(&constants, params).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.0.into());
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
        let mut cell = Cell::new(&constants, params).with_health(0.5.into()).with_energy(10.0.into());
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let params = CellParams {
            child_threshold_energy: 4.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(3.0.into());
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
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(1.0.into());
        let environment = CellEnvironment {
            food_per_cell: 3.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let (child, _) = cell.step(&environment);
        assert_eq!(child, None);
    }

    #[test]
    fn reproduction_clones_cell_params() {
        let params = CellParams {
            child_threshold_energy: 1.0.into(),
            child_threshold_food: 2.0.into(),
            attempted_eating_energy: 3.0.into(),
            attempted_healing_energy: 4.0.into(),
        };
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params).with_energy(10.0.into());
        let environment = CellEnvironment {
            food_per_cell: 10.0.into(),
            ..CellEnvironment::DEFAULT
        };
        let (child, _) = cell.step(&environment);
        assert_ne!(child, None);
        assert_eq!(child.unwrap().params, CellParams {
            child_threshold_energy: 1.0.into(),
            child_threshold_food: 2.0.into(),
            attempted_eating_energy: 3.0.into(),
            attempted_healing_energy: 4.0.into(),
        });
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
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&constants, params).with_energy(10.0.into());
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_ne!(child, None);
        assert_eq!(child.unwrap().state.energy, 2.5.into());
        assert_eq!(cell.energy(), 6.0.into());
    }

    #[test]
    fn child_starts_with_full_health() {
        let params = CellParams {
            child_threshold_energy: 1.0.into(),
            child_threshold_food: 0.0.into(),
            ..CellParams::DEFAULT
        };
        let mut cell = Cell::new(&Rc::new(CellConstants::DEFAULT), params)
            .with_health(0.5.into()).with_energy(10.0.into());
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_ne!(child, None);
        assert_eq!(child.unwrap().state.health, 1.0.into());
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
        let mut cell = Cell::new(&constants, params).with_energy(10.0.into());
        let (child, _) = cell.step(&CellEnvironment::DEFAULT);
        assert_ne!(child, None);
        assert_eq!(cell.health(), 0.75.into());
    }
}
