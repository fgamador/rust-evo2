use rand::distributions::Distribution;
use rand_distr::Normal;
use std::rc::Rc;
use crate::cell::{Cell, CellEnvironment, CellConstants, CellParams};
use crate::food_sources::FoodSource;
use crate::number_types::F32Positive;

pub struct World {
    cells: Vec<Cell>,
    food: F32Positive,
    food_sources: Vec<Box<dyn FoodSource>>,
}

impl World {
    pub fn new() -> Self {
        World {
            cells: vec![],
            food: 0.0.into(),
            food_sources: vec![],
        }
    }

    pub fn with_cells(mut self, cells: Vec<Cell>) -> Self {
        self.cells = cells;
        self
    }

    #[allow(dead_code)]
    pub fn with_cell(mut self, cell: Cell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn with_food(mut self, food: F32Positive) -> Self {
        self.food = food;
        self
    }

    pub fn with_food_sources(mut self, food_sources: Vec<Box<dyn FoodSource>>) -> Self {
        self.food_sources = food_sources;
        self
    }

    #[allow(dead_code)]
    pub fn cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    pub fn mean_health(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }

        self.cells.iter().map(|cell| cell.health().value()).sum::<f32>() / self.cells.len() as f32
    }

    pub fn mean_energy(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }

        self.cells.iter().map(|cell| cell.energy().value()).sum::<f32>() / self.cells.len() as f32
    }

    pub fn food(&self) -> F32Positive {
        self.food
    }

    pub fn step(&mut self) -> (usize, usize) {
        self.step_food_sources();

        let environment = CellEnvironment {
            food_per_cell: (self.food.value() / (self.cells.len() as f32)).into(),
        };
        let mut new_cells = vec![];
        let mut dead_cell_indexes = Vec::with_capacity(self.cells.len());

        self.step_cells(&environment, &mut new_cells, &mut dead_cell_indexes);

        let num_added = new_cells.len();
        self.cells.append(&mut new_cells);
        self.remove_cells(&mut dead_cell_indexes);

        (num_added, dead_cell_indexes.len())
    }

    fn step_food_sources(&mut self) {
        for food_source in &mut self.food_sources {
            self.food += food_source.food_this_step();
        }
    }

    fn step_cells(&mut self, environment: &CellEnvironment, new_cells: &mut Vec<Cell>, dead_cell_indexes: &mut Vec<usize>) {
        for (index, cell) in self.cells.iter_mut().enumerate() {
            let (child, food_eaten) = cell.step(environment);
            if let Some(child) = child {
                new_cells.push(child);
            }
            self.food -= food_eaten;
            if !cell.is_alive() {
                dead_cell_indexes.push(index);
            }
        }
    }

    fn remove_cells(&mut self, sorted_indexes: &mut [usize]) {
        for index in sorted_indexes.iter().rev() {
            self.cells.swap_remove(*index);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_cells(
    num_cells: usize,
    initial_energies: Normal<f32>,
    eating_energies: Normal<f32>,
    healing_energies: Normal<f32>,
    child_threshold_energies: Normal<f32>,
    child_threshold_foods: Normal<f32>,
    constants: &Rc<CellConstants>,
) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells = Vec::with_capacity(num_cells);
    for _ in 0..num_cells {
        cells.push(Cell::new(
            constants,
            CellParams {
                child_threshold_energy: child_threshold_energies.sample(&mut rng).into(),
                child_threshold_food: child_threshold_foods.sample(&mut rng).into(),
                attempted_eating_energy: eating_energies.sample(&mut rng).into(),
                attempted_healing_energy: healing_energies.sample(&mut rng).into(),
            },
        ).with_energy(initial_energies.sample(&mut rng).into()));
    }
    cells
}

#[cfg(test)]
mod tests {
    use crate::food_sources::ConstantFoodSource;
    use super::*;
    use crate::world::generate_cells;

    #[test]
    fn world_counts_both_living_and_dead_cells() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let world = World::new().with_cells(vec![
            Cell::new(&constants, CellParams::DEFAULT).with_energy(1.0.into()),
            Cell::new(&constants, CellParams::DEFAULT).with_energy(0.0.into()),
            Cell::new(&constants, CellParams::DEFAULT).with_energy(1.0.into()),
        ]);
        assert_eq!(world.num_cells(), 3);
    }

    #[test]
    fn world_mean_energy_with_no_cells_is_zero() {
        assert_eq!(World::new().mean_energy(), 0.0);
    }

    #[test]
    fn world_calculates_mean_energy() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let world = World::new().with_cells(vec![
            Cell::new(&constants, CellParams::DEFAULT).with_energy(1.0.into()),
            Cell::new(&constants, CellParams::DEFAULT).with_energy(2.0.into()),
        ]);
        assert_eq!(world.mean_energy(), 1.5);
    }

    #[test]
    fn generate_cells_with_normal_energy_distribution() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let cells = generate_cells(
            100,
            Normal::new(100.0, 5.0).unwrap(),
            Normal::new(0.0, 0.0).unwrap(),
            Normal::new(0.0, 0.0).unwrap(),
            Normal::new(0.0, 0.0).unwrap(),
            Normal::new(f32::MAX, 0.0).unwrap(),
            &constants,
        );
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e < 100.0.into()));
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e > 100.0.into()));
    }

    #[test]
    fn world_adds_new_cells() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let mut world = World::new()
            .with_food(0.0.into())
            .with_cells(vec![
                Cell::new(
                    &constants,
                    CellParams {
                        child_threshold_energy: 4.0.into(),
                        child_threshold_food: 0.0.into(),
                        ..CellParams::DEFAULT
                    })
                    .with_energy(10.0.into()),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 2);
    }

    #[test]
    fn world_reports_num_added() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let params = CellParams {
            child_threshold_energy: 4.0.into(),
            child_threshold_food: 0.0.into(),
            ..CellParams::DEFAULT
        };
        let mut world = World::new()
            .with_food(0.0.into())
            .with_cells(vec![
                Cell::new(&constants, params).with_energy(10.0.into()),
                Cell::new(&constants, params).with_energy(10.0.into()),
            ]);
        let (num_added, _) = world.step();
        assert_eq!(num_added, 2);
    }

    #[test]
    fn world_removes_dead_cells() {
        let constants = Rc::new(CellConstants::DEFAULT);
        let mut world = World::new()
            .with_food(0.0.into())
            .with_cells(vec![
                Cell::new(&constants, CellParams::DEFAULT).with_health(1.0.into()),
                Cell::new(&constants, CellParams::DEFAULT).with_health(0.0.into()),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 1);
    }

    #[test]
    fn world_reports_num_died() {
        let constants = Rc::new(CellConstants {
            health_reduction_per_energy_expended: 0.2.into(),
            ..CellConstants::DEFAULT
        });
        let hungry_params = CellParams {
            attempted_eating_energy: 5.0.into(),
            ..CellParams::DEFAULT
        };
        let mut world = World::new().with_cells(vec![
            Cell::new(&constants, CellParams::DEFAULT).with_energy(10.0.into()),
            Cell::new(&constants, hungry_params).with_energy(5.0.into()),
            Cell::new(&constants, hungry_params).with_energy(5.0.into()),
        ]);
        let (_, num_died) = world.step();
        assert_eq!(num_died, 2);
    }

    #[test]
    fn cells_consume_world_food() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: F32Positive::unchecked(1.0),
            ..CellConstants::DEFAULT
        });
        let mut world = World::new()
            .with_food(10.0.into())
            .with_cells(vec![
                Cell::new(
                    &constants,
                    CellParams {
                        attempted_eating_energy: 2.0.into(),
                        ..CellParams::DEFAULT
                    })
                    .with_energy(10.0.into()),
                Cell::new(
                    &constants,
                    CellParams {
                        attempted_eating_energy: 3.0.into(),
                        ..CellParams::DEFAULT
                    })
                    .with_energy(10.0.into()),
            ]);
        world.step();
        assert_eq!(world.food().value(), 5.0);
    }

    #[test]
    fn cells_cannot_consume_more_than_their_share_of_world_food() {
        let constants = Rc::new(CellConstants {
            food_yield_from_eating: F32Positive::unchecked(1.0),
            ..CellConstants::DEFAULT
        });
        let mut world = World::new()
            .with_food(4.0.into())
            .with_cells(vec![
                Cell::new(
                    &constants,
                    CellParams {
                        attempted_eating_energy: 3.0.into(),
                        ..CellParams::DEFAULT
                    })
                    .with_energy(10.0.into()),
                Cell::new(
                    &constants,
                    CellParams {
                        attempted_eating_energy: 1.0.into(),
                        ..CellParams::DEFAULT
                    })
                    .with_energy(10.0.into()),
            ]);
        world.step();
        assert_eq!(world.food().value(), 1.0);
    }

    #[test]
    fn food_sources_add_to_world_food() {
        let mut world = World::new()
            .with_food(0.0.into())
            .with_food_sources(vec![
                Box::new(ConstantFoodSource::new(2.0.into())),
                Box::new(ConstantFoodSource::new(3.0.into())),
            ]);
        world.step();
        assert_eq!(world.food().value(), 5.0);
    }
}
