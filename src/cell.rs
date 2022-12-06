use rand::rngs::ThreadRng;
use rand_distr::{Normal, Distribution};
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
        self.health().value() > 0.0
    }

    pub fn step(&mut self, mutation_number_source: &mut dyn MutationNumberSource, environment: &CellEnvironment) -> (Option<Cell>, F32Positive) {
        let (total_budgeted, budgeted_energies, child) =
            self.budget_and_maybe_reproduce(mutation_number_source, environment);

        self.expend_energy(total_budgeted);

        let food = self.eat(budgeted_energies.eating, environment.food_per_cell);
        self.digest(food);
        self.entropy();
        self.heal(budgeted_energies.healing);

        (child, food)
    }

    fn budget_and_maybe_reproduce(&mut self, mutation_number_source: &mut dyn MutationNumberSource, environment: &CellEnvironment) -> (F32Positive, CellEnergies, Option<Cell>) {
        let (total_budgeted, budgeted_energies) = self.budget_including_reproduction();
        if self.can_reproduce(budgeted_energies.reproduction, environment) {
            let child
                = self.reproduce(budgeted_energies.reproduction, mutation_number_source);
            (total_budgeted, budgeted_energies, child)
        } else {
            let (total_budgeted, budgeted_energies) = self.budget_excluding_reproduction();
            (total_budgeted, budgeted_energies, None)
        }
    }

    #[allow(unused_assignments)]
    fn budget_including_reproduction(&self) -> (F32Positive, CellEnergies) {
        let mut total_budgeted = 0.into(); // make Rust plugin shut up about uninitialized var
        let mut budgeted_energies = CellEnergies::new();
        (total_budgeted,
         [budgeted_energies.reproduction,
             budgeted_energies.eating,
             budgeted_energies.healing]) =
            budget(self.state.energy,
                   &[self.params.child_threshold_energy,
                       self.params.attempted_eating_energy,
                       self.params.attempted_healing_energy]);
        (total_budgeted, budgeted_energies)
    }

    #[allow(unused_assignments)]
    fn budget_excluding_reproduction(&mut self) -> (F32Positive, CellEnergies) {
        let mut total_budgeted = 0.into(); // make Rust plugin shut up about uninitialized var
        let mut budgeted_energies = CellEnergies::new();
        (total_budgeted, [budgeted_energies.eating, budgeted_energies.healing]) =
            budget(self.state.energy,
                   &[self.params.attempted_eating_energy,
                       self.params.attempted_healing_energy]);
        budgeted_energies.reproduction = 0.into();
        (total_budgeted, budgeted_energies)
    }

    fn can_reproduce(&self, reproduction_energy: F32Positive, environment: &CellEnvironment) -> bool {
        reproduction_energy >= self.params.child_threshold_energy
            && environment.food_per_cell >= self.params.child_threshold_food
    }

    fn reproduce(&mut self, reproduction_energy: F32Positive, mutation_number_source: &mut dyn MutationNumberSource) -> Option<Cell> {
        let mut child = self.clone();
        child.mutate(mutation_number_source);
        child.state.health = 1.0.into();
        child.state.energy = reproduction_energy - self.constants.create_child_energy;
        Some(child)
    }

    fn mutate(&mut self, mutation_number_source: &mut dyn MutationNumberSource) {
        self.params.attempted_eating_energy = mutation_number_source.mutate(
            self.params.attempted_eating_energy, self.constants.attempted_eating_energy_mutation_stdev);
        self.params.attempted_healing_energy = mutation_number_source.mutate(
            self.params.attempted_healing_energy, self.constants.attempted_healing_energy_mutation_stdev);
        self.params.child_threshold_energy = mutation_number_source.mutate(
            self.params.child_threshold_energy, self.constants.child_threshold_energy_mutation_stdev);
        self.params.child_threshold_food = mutation_number_source.mutate(
            self.params.child_threshold_food, self.constants.child_threshold_food_mutation_stdev);
    }

    fn eat(&mut self, eating_energy: F32Positive, food_per_cell: F32Positive) -> F32Positive {
        (eating_energy * self.constants.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: F32Positive) {
        self.state.energy += food_amount * self.constants.energy_yield_from_digestion;
    }

    fn entropy(&mut self) {
        self.state.health -= self.constants.health_reduction_from_entropy;
    }

    fn heal(&mut self, healing_energy: F32Positive) {
        self.state.health += healing_energy * self.constants.health_increase_per_healing_energy;
    }

    fn expend_energy(&mut self, energy: F32Positive) {
        self.state.energy -= energy;
        self.state.health -= energy * self.constants.health_reduction_per_energy_expended;
    }
}

fn budget<const N: usize>(available: F32Positive, desired: &[F32Positive; N]) -> (F32Positive, [F32Positive; N]) {
    let desired_sum = desired.iter().sum::<F32Positive>();
    if available < desired_sum {
        let reduction_factor = available / desired_sum;
        let budgeted = desired.map(|item| { item * reduction_factor });
        (available, budgeted)
    } else {
        (desired_sum, *desired)
    }
}

#[derive(Debug, PartialEq)]
pub struct CellConstants {
    pub create_child_energy: F32Positive,
    pub energy_yield_from_digestion: F32Positive,
    pub food_yield_from_eating: F32Positive,
    pub health_increase_per_healing_energy: F32ZeroToOnePerF32Positive,
    pub health_reduction_from_entropy: F32ZeroToOne,
    pub health_reduction_per_energy_expended: F32ZeroToOnePerF32Positive,
    pub attempted_eating_energy_mutation_stdev: F32Positive,
    pub attempted_healing_energy_mutation_stdev: F32Positive,
    pub child_threshold_energy_mutation_stdev: F32Positive,
    pub child_threshold_food_mutation_stdev: F32Positive,
}

impl CellConstants {
    #[allow(dead_code)]
    pub const DEFAULT: CellConstants = CellConstants {
        create_child_energy: F32Positive::unchecked(0.0),
        energy_yield_from_digestion: F32Positive::unchecked(0.0),
        food_yield_from_eating: F32Positive::unchecked(0.0),
        health_increase_per_healing_energy: F32ZeroToOnePerF32Positive::unchecked(0.0),
        health_reduction_from_entropy: F32ZeroToOne::unchecked(0.0),
        health_reduction_per_energy_expended: F32ZeroToOnePerF32Positive::unchecked(0.0),
        attempted_eating_energy_mutation_stdev: F32Positive::unchecked(0.0),
        attempted_healing_energy_mutation_stdev: F32Positive::unchecked(0.0),
        child_threshold_energy_mutation_stdev: F32Positive::unchecked(0.0),
        child_threshold_food_mutation_stdev: F32Positive::unchecked(0.0),
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellParams {
    pub attempted_eating_energy: F32Positive,
    pub attempted_healing_energy: F32Positive,
    pub child_threshold_energy: F32Positive,
    pub child_threshold_food: F32Positive,
}

impl CellParams {
    #[allow(dead_code)]
    pub const DEFAULT: CellParams = CellParams {
        attempted_eating_energy: F32Positive::unchecked(0.0),
        attempted_healing_energy: F32Positive::unchecked(0.0),
        child_threshold_energy: F32Positive::unchecked(f32::MAX),
        child_threshold_food: F32Positive::unchecked(f32::MAX),
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellState {
    pub energy: F32Positive,
    pub health: F32ZeroToOne,
}

impl CellState {
    #[allow(dead_code)]
    pub const DEFAULT: CellState = CellState {
        energy: F32Positive::unchecked(0.0),
        health: F32ZeroToOne::unchecked(1.0),
    };
}

pub struct CellEnvironment {
    pub food_per_cell: F32Positive,
}

impl CellEnvironment {
    #[allow(dead_code)]
    pub const DEFAULT: CellEnvironment = CellEnvironment { food_per_cell: F32Positive::unchecked(0.0) };
}

struct CellEnergies {
    reproduction: F32Positive,
    eating: F32Positive,
    healing: F32Positive,
}

impl CellEnergies {
    fn new() -> Self {
        CellEnergies {
            reproduction: 0.into(),
            eating: 0.into(),
            healing: 0.into(),
        }
    }
}

pub trait MutationNumberSource {
    fn mutate(&mut self, value: F32Positive, stdev: F32Positive) -> F32Positive;
}

pub struct NullMutationNumberSource {}

impl NullMutationNumberSource {
    pub fn new() -> Self {
        NullMutationNumberSource {}
    }
}

impl MutationNumberSource for NullMutationNumberSource {
    fn mutate(&mut self, value: F32Positive, _stdev: F32Positive) -> F32Positive {
        value
    }
}

pub struct RandomMutationNumberSource {
    rng: ThreadRng,
}

impl RandomMutationNumberSource {
    pub fn new() -> Self {
        RandomMutationNumberSource {
            rng: rand::thread_rng(),
        }
    }
}

impl MutationNumberSource for RandomMutationNumberSource {
    fn mutate(&mut self, value: F32Positive, stdev: F32Positive) -> F32Positive {
        let normal = Normal::new(value.value(), stdev.value()).unwrap();
        loop {
            let mutated = normal.sample(&mut self.rng);
            if mutated >= 0.0 {
                return F32Positive::unchecked(mutated);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budgeting_adjusts_downward_proportionally() {
        let desired: [F32Positive; 2] = [10.into(), 5.into()];

        let (total_budgeted, budgeted) = budget(7.5.into(), &desired);

        assert_eq!(total_budgeted, 7.5.into());
        assert_eq!(budgeted, [5.into(), 2.5.into()]);
    }

    #[test]
    fn budgeting_leaves_satisfiable_requests_unchanged() {
        let desired: [F32Positive; 2] = [10.into(), 5.into()];

        let (total_budgeted, budgeted) = budget(20.into(), &desired);

        assert_eq!(total_budgeted, 15.into());
        assert_eq!(budgeted, [10.into(), 5.into()]);
    }

    #[test]
    fn new_cell_has_full_health() {
        let cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams::DEFAULT);

        assert_eq!(cell.health(), 1.0.into());
    }

    #[test]
    fn cell_suffers_entropic_damage() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                health_reduction_from_entropy: 0.25.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams::DEFAULT)
            .with_health(1.0.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn cell_uses_energy() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                attempted_eating_energy: 5.25.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.energy(), 4.75.into());
    }

    #[test]
    fn cell_cannot_expend_energy_below_zero() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                attempted_eating_energy: 11.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.energy(), 0.into());
    }

    #[test]
    fn cell_with_zero_health_is_dead() {
        let cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams::DEFAULT)
            .with_health(0.0.into())
            .with_energy(1.into());

        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_with_health_but_no_energy_is_alive() {
        let cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams::DEFAULT)
            .with_health(1.0.into())
            .with_energy(0.into());

        assert!(cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 1.5.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (_, food_eaten) = cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(food_eaten, 3.into());
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 1.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 3.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (_, food_eaten) = cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 2.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(food_eaten, 2.into());
    }

    #[test]
    fn cell_expends_energy_eating() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 0.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(5.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(cell.energy(), 3.into());
    }

    #[test]
    fn cell_expends_energy_eating_even_when_there_is_no_food() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 0.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(5.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 0.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(cell.energy(), 3.into());
    }

    #[test]
    fn cell_digests_food() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 1.into(),
                energy_yield_from_digestion: 1.5.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(cell.energy(), 11.into());
    }

    #[test]
    fn expending_eating_energy_reduces_health() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                health_reduction_per_energy_expended: 0.125.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn expending_eating_energy_cannot_reduce_health_below_zero() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                health_reduction_per_energy_expended: 1.0.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.health(), 0.0.into());
    }

    #[test]
    fn cell_can_heal() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                health_increase_per_healing_energy: 0.25.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_healing_energy: 1.into(),
                ..CellParams::DEFAULT
            })
            .with_health(0.5.into())
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn cell_can_fully_heal_despite_health_damage_from_energy_use() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                health_increase_per_healing_energy: 0.75.into(),
                health_reduction_per_energy_expended: 0.25.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_healing_energy: 1.into(),
                ..CellParams::DEFAULT
            })
            .with_health(0.5.into())
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(cell.health(), 1.0.into());
    }

    #[test]
    fn cell_with_insufficient_energy_does_not_reproduce() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                child_threshold_energy: 4.into(),
                child_threshold_food: 0.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(3.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_eq!(child, None);
    }

    #[test]
    fn cell_with_insufficient_food_does_not_reproduce() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                child_threshold_energy: 0.into(),
                child_threshold_food: 4.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(1.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 3.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(child, None);
    }

    #[test]
    fn reproduction_clones_cell_params() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                child_threshold_energy: 1.into(),
                child_threshold_food: 2.into(),
                attempted_eating_energy: 3.into(),
                attempted_healing_energy: 4.into(),
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(
            &mut mutation_number_source, &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_ne!(child, None);
        assert_eq!(child.unwrap().params, CellParams {
            child_threshold_energy: 1.into(),
            child_threshold_food: 2.into(),
            attempted_eating_energy: 3.into(),
            attempted_healing_energy: 4.into(),
        });
    }

    #[test]
    fn reproduction_mutates_cell_params() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                child_threshold_energy_mutation_stdev: 0.25.into(),
                child_threshold_food_mutation_stdev: 0.5.into(),
                attempted_eating_energy_mutation_stdev: 0.75.into(),
                attempted_healing_energy_mutation_stdev: 1.0.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                child_threshold_energy: 1.into(),
                child_threshold_food: 2.into(),
                attempted_eating_energy: 3.into(),
                attempted_healing_energy: 4.into(),
            })
            .with_energy(10.into());

        let mut mutation_number_source = AdditiveMutationNumberSource::new();
        let (child, _) = cell.step(
            &mut mutation_number_source,
            &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_ne!(child, None);
        assert_eq!(child.unwrap().params, CellParams {
            child_threshold_energy: 1.25.into(),
            child_threshold_food: 2.5.into(),
            attempted_eating_energy: 3.75.into(),
            attempted_healing_energy: 5.0.into(),
        });
    }

    #[test]
    fn cell_passes_energy_to_child() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                create_child_energy: 1.5.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                child_threshold_energy: 4.into(),
                child_threshold_food: 0.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_ne!(child, None);
        assert_eq!(child.unwrap().state.energy, 2.5.into());
        assert_eq!(cell.energy(), 6.into());
    }

    #[test]
    fn child_starts_with_full_health() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants::DEFAULT),
            CellParams {
                child_threshold_energy: 1.into(),
                child_threshold_food: 0.into(),
                ..CellParams::DEFAULT
            })
            .with_health(0.5.into())
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_ne!(child, None);
        assert_eq!(child.unwrap().state.health, 1.0.into());
    }

    #[test]
    fn expending_reproduction_energy_reduces_health() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                create_child_energy: 0.into(),
                health_reduction_per_energy_expended: 0.125.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                child_threshold_energy: 2.into(),
                child_threshold_food: 0.into(),
                ..CellParams::DEFAULT
            })
            .with_energy(10.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, _) = cell.step(&mut mutation_number_source, &CellEnvironment::DEFAULT);

        assert_ne!(child, None);
        assert_eq!(cell.health(), 0.75.into());
    }

    #[test]
    fn cell_behavior_is_limited_by_energy_budget() {
        let mut cell = Cell::new(
            &Rc::new(CellConstants {
                food_yield_from_eating: 1.into(),
                energy_yield_from_digestion: 0.into(),
                health_increase_per_healing_energy: 0.25.into(),
                ..CellConstants::DEFAULT
            }),
            CellParams {
                attempted_eating_energy: 2.into(),
                attempted_healing_energy: 2.into(),
                child_threshold_energy: 2.into(),
                child_threshold_food: 0.into(),
            })
            .with_health(0.25.into())
            .with_energy(2.into());
        let mut mutation_number_source = NullMutationNumberSource::new();

        let (child, food_eaten) = cell.step(
            &mut mutation_number_source,
            &CellEnvironment {
                food_per_cell: 10.into(),
                ..CellEnvironment::DEFAULT
            });

        assert_eq!(child, None);
        assert_eq!(food_eaten, 1.into());
        assert_eq!(cell.health(), 0.5.into());
        assert_eq!(cell.energy(), 0.into());
    }

    pub struct AdditiveMutationNumberSource {}

    impl AdditiveMutationNumberSource {
        pub fn new() -> Self {
            AdditiveMutationNumberSource {}
        }
    }

    impl MutationNumberSource for AdditiveMutationNumberSource {
        fn mutate(&mut self, value: F32Positive, stdev: F32Positive) -> F32Positive {
            value + stdev
        }
    }
}
