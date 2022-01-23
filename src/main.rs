use clap::Parser;
use rand_distr::{Normal, Distribution};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Initial number of cells
    #[clap(short('n'), long, default_value_t = 100)]
    cells: usize,

    /// Mean of cell initial energies
    #[clap(short('e'), long, default_value_t = 100.0)]
    mean_energy: f32,

    /// Cell energy use per time step
    #[clap(short('u'), long, default_value_t = 5.0)]
    energy_use: f32,
}

fn main() {
    let args = Args::parse();

    let cell_params = CellParameters {
        mean_initial_energy: args.mean_energy,
        energy_use_per_step: args.energy_use,
        ..CellParameters::DEFAULT
    };

    let mut world = World::new(generate_cells(args.cells, cell_params));

    while world.num_alive() > 0 {
        let (num_created, num_died) = world.step();
        println!("+{} -{} -> {} (e: {})", num_created, num_died, world.num_alive(),
                 world.mean_energy());
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

    pub fn mean_energy(&self) -> f32 {
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
    let mut rng = rand::thread_rng();
    let normal = Normal::new(cell_params.mean_initial_energy, cell_params.stdev_initial_energy).unwrap();

    let mut cells = Vec::with_capacity(num_cells);
    for _ in 0..num_cells {
        cells.push(Cell {
            energy: normal.sample(&mut rng),
            energy_use_per_step: cell_params.energy_use_per_step,
        });
    }
    cells
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
    pub mean_initial_energy: f32,
    pub stdev_initial_energy: f32,
    pub energy_use_per_step: f32,
}

impl CellParameters {
    pub const DEFAULT: CellParameters = CellParameters {
        mean_initial_energy: 100.0,
        stdev_initial_energy: 0.0,
        energy_use_per_step: 0.0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let subject = World::new(generate_cells(42, CellParameters::DEFAULT));
        assert_eq!(subject.num_alive(), 42);
    }

    #[test]
    fn mean_energy_starts_at_mean_initial_energy() {
        let cell_params = CellParameters {
            mean_initial_energy: 39.5,
            ..CellParameters::DEFAULT
        };
        let subject = World::new(generate_cells(100, cell_params));
        assert_eq!(subject.mean_energy(), 39.5);
    }

    #[test]
    fn mean_energy_with_no_cells_is_zero() {
        let subject = World::new(generate_cells(0, CellParameters::DEFAULT));
        assert_eq!(subject.mean_energy(), 0.0);
    }

    #[test]
    fn cells_use_energy() {
        let cell_params = CellParameters {
            mean_initial_energy: 10.0,
            energy_use_per_step: 5.25,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(generate_cells(100, cell_params));
        subject.step();
        assert_eq!(subject.mean_energy(), 4.75);
    }

    #[test]
    fn generate_cells_from_normal_distribution() {
        let cell_params = CellParameters {
            mean_initial_energy: 100.0,
            stdev_initial_energy: 5.0,
            ..CellParameters::DEFAULT
        };
        let cells = generate_cells(100, cell_params);
        assert!(cells.iter().map(|cell| cell.energy).any(|e| e < 100.0));
        assert!(cells.iter().map(|cell| cell.energy).any(|e| e > 100.0));
    }

    #[test]
    fn dead_cells_disappear() {
        let cell_params = CellParameters {
            mean_initial_energy: 10.0,
            energy_use_per_step: 11.0,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(generate_cells(10, cell_params));
        subject.step();
        assert_eq!(subject.num_alive(), 0);
    }

    #[test]
    fn world_step_reports_num_died() {
        let cell_params = CellParameters {
            mean_initial_energy: 10.0,
            energy_use_per_step: 5.0,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(generate_cells(10, cell_params));
        let (_, num_died) = subject.step();
        assert_eq!(num_died, 0);
        let (_, num_died) = subject.step();
        assert_eq!(num_died, 10);
    }
}
