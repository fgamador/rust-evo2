use std::rc::Rc;
use crate::{Cell, CellEnvironment, CellParameters};
use rand::distributions::Distribution;
use rand_distr::Normal;

pub const DEFAULT_FOOD_AMOUNT: f32 = 0.0;

pub struct World {
    cells: Vec<Cell>,
    food: f32,
    food_sources: Vec<Box<dyn FoodSource>>,
}

impl World {
    pub fn new() -> Self {
        World {
            cells: vec![],
            food: 0.0,
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

    pub fn with_food(mut self, food: f32) -> Self {
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

    pub fn mean_energy(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }

        self.cells.iter().map(|cell| cell.energy()).sum::<f32>() / self.cells.len() as f32
    }

    pub fn food(&self) -> f32 {
        self.food
    }

    pub fn step(&mut self) -> (usize, usize) {
        self.step_food_sources();

        let environment = CellEnvironment {
            food_per_cell: self.food / (self.cells.len() as f32),
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
        for food_source in &self.food_sources {
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

pub fn generate_cells(
    num_cells: usize,
    initial_energies: Normal<f32>,
    eating_energies: Normal<f32>,
    child_threshold_energies: Normal<f32>,
    cell_params: &Rc<CellParameters>,
) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells = Vec::with_capacity(num_cells);
    for _ in 0..num_cells {
        cells.push(Cell::new(
            cell_params,
            initial_energies.sample(&mut rng),
            child_threshold_energies.sample(&mut rng),
            eating_energies.sample(&mut rng),
        ));
    }
    cells
}

pub trait FoodSource {
    fn food_this_step(&self) -> f32;
}

pub struct ConstantFoodSource {
    food_per_step: f32,
}

impl ConstantFoodSource {
    pub fn new(food_per_step: f32) -> Self {
        ConstantFoodSource {
            food_per_step
        }
    }
}

impl FoodSource for ConstantFoodSource {
    fn food_this_step(&self) -> f32 {
        self.food_per_step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::generate_cells;

    #[test]
    fn world_counts_both_living_and_dead_cells() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let world = World::new().with_cells(vec![
            Cell::new(&cell_params, 1.0, f32::MAX, 0.0),
            Cell::new(&cell_params, 0.0, f32::MAX, 0.0),
            Cell::new(&cell_params, 1.0, f32::MAX, 0.0),
        ]);
        assert_eq!(world.num_cells(), 3);
    }

    #[test]
    fn world_mean_energy_with_no_cells_is_zero() {
        assert_eq!(World::new().mean_energy(), 0.0);
    }

    #[test]
    fn world_calculates_mean_energy() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let world = World::new().with_cells(vec![
            Cell::new(&cell_params, 1.0, f32::MAX, 0.0),
            Cell::new(&cell_params, 2.0, f32::MAX, 0.0),
        ]);
        assert_eq!(world.mean_energy(), 1.5);
    }

    #[test]
    fn generate_cells_with_normal_energy_distribution() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let cells = generate_cells(
            100,
            Normal::new(100.0, 5.0).unwrap(),
            Normal::new(0.0, 0.0).unwrap(),
            Normal::new(f32::MAX, 0.0).unwrap(),
            &cell_params,
        );
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e < 100.0));
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e > 100.0));
    }

    #[test]
    fn world_adds_new_cells() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let mut world = World::new()
            .with_food(0.0)
            .with_cells(vec![
                Cell::new(&cell_params, 10.0, 4.0, 0.0),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 2);
    }

    #[test]
    fn world_reports_num_added() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let mut world = World::new()
            .with_food(0.0)
            .with_cells(vec![
                Cell::new(&cell_params, 10.0, 4.0, 0.0),
                Cell::new(&cell_params, 10.0, 4.0, 0.0),
            ]);
        let (num_added, _) = world.step();
        assert_eq!(num_added, 2);
    }

    #[test]
    fn world_removes_dead_cells() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let mut world = World::new()
            .with_food(0.0)
            .with_cells(vec![
                Cell::new(&cell_params, 1.0, f32::MAX, 0.0),
                Cell::new(&cell_params, 0.0, f32::MAX, 0.0),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 1);
    }

    #[test]
    fn world_reports_num_died() {
        let cell_params = Rc::new(CellParameters {
            maintenance_energy_use: 5.0,
            ..CellParameters::DEFAULT
        });
        let mut world = World::new().with_cells(vec![
            Cell::new(&cell_params, 10.0, f32::MAX, 0.0),
            Cell::new(&cell_params, 5.0, f32::MAX, 0.0),
            Cell::new(&cell_params, 5.0, f32::MAX, 0.0),
        ]);
        let (_, num_died) = world.step();
        assert_eq!(num_died, 2);
    }

    #[test]
    fn cells_consume_world_food() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let mut world = World::new()
            .with_food(10.0)
            .with_cells(vec![
                Cell::new(&cell_params, 1.0, f32::MAX, 2.0),
                Cell::new(&cell_params, 1.0, f32::MAX, 3.0),
            ]);
        world.step();
        assert_eq!(world.food(), 5.0);
    }

    #[test]
    fn cells_cannot_consume_more_than_their_share_of_world_food() {
        let cell_params = Rc::new(CellParameters::DEFAULT);
        let mut world = World::new()
            .with_food(4.0)
            .with_cells(vec![
                Cell::new(&cell_params, 1.0, f32::MAX, 3.0),
                Cell::new(&cell_params, 1.0, f32::MAX, 1.0),
            ]);
        world.step();
        assert_eq!(world.food(), 1.0);
    }

    #[test]
    fn food_sources_add_to_world_food() {
        let mut world = World::new()
            .with_food(0.0)
            .with_food_sources(vec![
                Box::new(ConstantFoodSource::new(2.0)),
                Box::new(ConstantFoodSource::new(3.0)),
            ]);
        world.step();
        assert_eq!(world.food(), 5.0);
    }
}
