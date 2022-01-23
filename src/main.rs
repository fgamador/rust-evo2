fn main() {
    let num_cells = 100;
    let cell_params = CellParameters {
        energy_use_per_step: 5.0,
        ..CellParameters::DEFAULT
    };
    let mut world = World::new(generate_cells(num_cells, cell_params));
    while world.num_alive() > 0 {
        let (num_created, num_died) = world.step();
        println!("+{} -{} -> {} (e: {})", num_created, num_died, world.num_alive(),
                 world.average_energy());
    }
}

pub struct World {
    cells: Vec<Cell>,
}

impl World {
    pub fn new(cells: Vec<Cell>) -> World {
        World {
            cells,
        }
    }

    pub fn num_alive(&self) -> usize {
        self.cells.len()
    }

    pub fn average_energy(&self) -> f32 {
        if self.cells.is_empty() { 0.0 } else { self.cells[0].energy }
    }

    pub fn step(&mut self) -> (usize, usize) {
        let mut dead_indexes = Vec::with_capacity(self.cells.len());
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.step();
            if cell.energy <= 0.0 {
                dead_indexes.push(index);
            }
        }
        self.remove_cells(&mut dead_indexes);
        (0, dead_indexes.len())
    }

    fn remove_cells(&mut self, sorted_indexes: &mut Vec<usize>) {
        for index in sorted_indexes.iter().rev() {
            self.cells.swap_remove(*index);
        }
    }
}

pub fn generate_cells(num_cells: usize, cell_params: CellParameters) -> Vec<Cell> {
    vec![Cell {
        energy: cell_params.initial_energy,
        energy_use_per_step: cell_params.energy_use_per_step,
    }; num_cells]
}

#[derive(Clone)]
pub struct Cell {
    pub energy: f32,
    pub energy_use_per_step: f32,
}

impl Cell {
    pub fn step(&mut self) {
        self.energy -= self.energy_use_per_step;
    }
}

pub struct CellParameters {
    pub initial_energy: f32,
    pub energy_use_per_step: f32,
}

impl CellParameters {
    pub const DEFAULT: CellParameters = CellParameters {
        initial_energy: 100.0,
        energy_use_per_step: 0.0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let world = World::new(generate_cells(42, CellParameters::DEFAULT));
        assert_eq!(world.num_alive(), 42);
    }

    #[test]
    fn average_energy_starts_at_initial_energy() {
        let cell_params = CellParameters {
            initial_energy: 39.5,
            ..CellParameters::DEFAULT
        };
        let world = World::new(generate_cells(100, cell_params));
        assert_eq!(world.average_energy(), 39.5);
    }

    #[test]
    fn average_energy_with_no_cells_is_zero() {
        let cell_params = CellParameters::DEFAULT;
        let world = World::new(generate_cells(0, cell_params));
        assert_eq!(world.average_energy(), 0.0);
    }

    #[test]
    fn cells_use_energy() {
        let cell_params = CellParameters {
            initial_energy: 10.0,
            energy_use_per_step: 5.25,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(generate_cells(100, cell_params));
        world.step();
        assert_eq!(world.average_energy(), 4.75);
    }

    #[test]
    fn dead_cells_disappear() {
        let cell_params = CellParameters {
            initial_energy: 10.0,
            energy_use_per_step: 11.0,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(generate_cells(10, cell_params));
        world.step();
        assert_eq!(world.num_alive(), 0);
    }

    #[test]
    fn world_step_reports_num_died() {
        let cell_params = CellParameters {
            initial_energy: 10.0,
            energy_use_per_step: 5.0,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(generate_cells(10, cell_params));
        let (_, num_died) = world.step();
        assert_eq!(num_died, 0);
        let (_, num_died) = world.step();
        assert_eq!(num_died, 10);
    }
}
