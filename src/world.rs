use crate::{Cell, CellEnvironment, CellParameters};
use rand::distributions::Distribution;
use rand_distr::Normal;

pub const DEFAULT_FOOD_AMOUNT: f32 = 0.0;

pub struct World<'a> {
    cells: Vec<Cell<'a>>,
    food: f32,
}

impl<'a> World<'a> {
    pub fn new() -> World<'a> {
        World {
            cells: vec![],
            food: 0.0,
        }
    }

    pub fn with_cells(mut self, cells: Vec<Cell<'a>>) -> Self {
        self.cells = cells;
        self
    }

    #[allow(dead_code)]
    pub fn with_cell(mut self, cell: Cell<'a>) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn with_food(mut self, food: f32) -> Self {
        self.food = food;
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
        let environment = CellEnvironment {
            food_per_cell: self.food / (self.cells.len() as f32),
        };
        //let mut new_cells = vec![];
        let mut dead_indexes = Vec::with_capacity(self.cells.len());

        for (index, cell) in self.cells.iter_mut().enumerate() {
            let (_child, food_eaten) = cell.step(&environment);
            // if let Some(child) = child {
            //     new_cells.push(child);
            // }
            self.food -= food_eaten;
            if !cell.is_alive() {
                dead_indexes.push(index);
            }
        }

        //self.cells.append(&mut new_cells);
        self.remove_cells(&mut dead_indexes);

        (0, dead_indexes.len())
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
    cell_params: &CellParameters,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::generate_cells;

    #[test]
    fn world_counts_both_living_and_dead_cells() {
        let world = World::new().with_cells(vec![
            Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 0.0),
            Cell::new(&CellParameters::DEFAULT, 0.0, f32::MAX, 0.0),
            Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 0.0),
        ]);
        assert_eq!(world.num_cells(), 3);
    }

    #[test]
    fn world_mean_energy_with_no_cells_is_zero() {
        assert_eq!(World::new().mean_energy(), 0.0);
    }

    #[test]
    fn world_calculates_mean_energy() {
        let world = World::new().with_cells(vec![
            Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 0.0),
            Cell::new(&CellParameters::DEFAULT, 2.0, f32::MAX, 0.0),
        ]);
        assert_eq!(world.mean_energy(), 1.5);
    }

    #[test]
    fn generate_cells_with_normal_energy_distribution() {
        let cells = generate_cells(
            100,
            Normal::new(100.0, 5.0).unwrap(),
            Normal::new(0.0, 0.0).unwrap(),
            Normal::new(f32::MAX, 0.0).unwrap(),
            &CellParameters::DEFAULT,
        );
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e < 100.0));
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e > 100.0));
    }

    #[test]
    #[ignore]
    fn world_adds_new_cells() {
        let mut world = World::new()
            .with_food(0.0)
            .with_cells(vec![
                Cell::new(&CellParameters::DEFAULT, 10.0, 4.0, 0.0),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 2);
    }

    #[test]
    fn world_removes_dead_cells() {
        let mut world = World::new()
            .with_food(0.0)
            .with_cells(vec![
                Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 0.0),
                Cell::new(&CellParameters::DEFAULT, 0.0, f32::MAX, 0.0),
            ]);
        world.step();
        assert_eq!(world.num_cells(), 1);
    }

    #[test]
    fn world_reports_num_died() {
        let cell_params = CellParameters {
            maintenance_energy_use: 5.0,
            ..CellParameters::DEFAULT
        };
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
        let mut world = World::new()
            .with_food(10.0)
            .with_cells(vec![
                Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 2.0),
                Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 3.0),
            ]);
        world.step();
        assert_eq!(world.food(), 5.0);
    }

    #[test]
    fn cells_cannot_consume_more_than_their_share_of_world_food() {
        let mut world = World::new()
            .with_food(4.0)
            .with_cells(vec![
                Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 3.0),
                Cell::new(&CellParameters::DEFAULT, 1.0, f32::MAX, 1.0),
            ]);
        world.step();
        assert_eq!(world.food(), 1.0);
    }
}
