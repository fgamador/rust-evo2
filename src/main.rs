use clap::Parser;
use rand_distr::{Distribution, Normal};

const DEFAULT_MEAN_INITIAL_ENERGY: f32 = 100.0;
const DEFAULT_STD_DEV_INITIAL_ENERGY: f32 = 0.0;
const DEFAULT_ENERGY_USE_PER_STEP: f32 = 0.0;
const DEFAULT_ABSORPTION_YIELD_FACTOR: f32 = 1.0;
const DEFAULT_FOOD_CONCENTRATION: f32 = 0.0;
const DEFAULT_FOOD_AMOUNT: f32 = 0.0;

fn main() {
    let args = Args::parse();

    let cell_params = CellParameters {
        energy_use_per_step: args.energy_use,
        ..CellParameters::DEFAULT
    };

    let mut world = World::new(generate_cells(
        args.cells,
        args.mean_energy,
        args.std_dev_energy,
        &cell_params,
    ));

    while world.num_alive() > 0 {
        let (num_created, num_died) = world.step(&Environment::DEFAULT);
        println!(
            "+{} -{} -> {} (e: {})",
            num_created,
            num_died,
            world.num_alive(),
            world.mean_energy()
        );
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Initial number of cells
    #[clap(short('n'), long, default_value_t = 100)]
    cells: usize,

    /// Mean of cell initial energies
    #[clap(short('e'), long, default_value_t = DEFAULT_MEAN_INITIAL_ENERGY)]
    mean_energy: f32,

    /// Standard deviation of cell initial energies
    #[clap(short('s'), long, default_value_t = DEFAULT_STD_DEV_INITIAL_ENERGY)]
    std_dev_energy: f32,

    /// Cell energy use per time step
    #[clap(short('u'), long, default_value_t = DEFAULT_ENERGY_USE_PER_STEP)]
    energy_use: f32,

    /// Initial amount of food
    #[clap(short('f'), long, default_value_t = DEFAULT_FOOD_AMOUNT)]
    food_amount: f32,
}

pub struct World<'a> {
    cells: Vec<Cell<'a>>,
}

impl<'a> World<'a> {
    pub fn new(cells: Vec<Cell>) -> World {
        World { cells }
    }

    pub fn num_alive(&self) -> usize {
        self.cells.len()
    }

    pub fn mean_energy(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }

        self.cells.iter().map(|cell| cell.energy()).sum::<f32>() / self.cells.len() as f32
    }

    pub fn step(&mut self, environment: &Environment) -> (usize, usize) {
        let mut dead_indexes = Vec::with_capacity(self.cells.len());
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.step(environment);
            if !cell.is_alive() {
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

pub fn generate_cells(
    num_cells: usize,
    mean_initial_energy: f32,
    std_dev_initial_energy: f32,
    //absorption_energy_per_step: f32,
    cell_params: &CellParameters,
) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(mean_initial_energy, std_dev_initial_energy).unwrap();

    let mut cells = Vec::with_capacity(num_cells);
    for _ in 0..num_cells {
        cells.push(Cell::new(
            cell_params,
            normal.sample(&mut rng),
            0.0, // absorption_energy_per_step: f32,
        ));
    }
    cells
}

pub struct Cell<'a> {
    cell_params: &'a CellParameters,
    energy: f32,
    absorption_energy_per_step: f32,
}

impl<'a> Cell<'a> {
    pub fn new(
        cell_params: &'a CellParameters,
        energy: f32,
        absorption_energy_per_step: f32,
    ) -> Self {
        Cell {
            cell_params,
            energy,
            absorption_energy_per_step,
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy() > 0.0
    }

    pub fn step(&mut self, environment: &Environment) {
        self.energy += self.absorption_energy_per_step
            * self.cell_params.absorption_yield_factor
            * environment.food_concentration;
        self.energy -= self.cell_params.energy_use_per_step;
    }
}

pub struct CellParameters {
    pub energy_use_per_step: f32,
    pub absorption_yield_factor: f32,
}

impl CellParameters {
    pub const DEFAULT: CellParameters = CellParameters {
        energy_use_per_step: DEFAULT_ENERGY_USE_PER_STEP,
        absorption_yield_factor: DEFAULT_ABSORPTION_YIELD_FACTOR,
    };
}

pub struct Environment {
    food_concentration: f32,
}

impl Environment {
    pub const DEFAULT: Environment = Environment {
        food_concentration: DEFAULT_FOOD_CONCENTRATION,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let subject = World::new(generate_cells(42, 10.0, 0.0, &CellParameters::DEFAULT));
        assert_eq!(subject.num_alive(), 42);
    }

    #[test]
    fn mean_energy_starts_at_mean_initial_energy() {
        let subject = World::new(generate_cells(100, 39.5, 0.0, &CellParameters::DEFAULT));
        assert_eq!(subject.mean_energy(), 39.5);
    }

    #[test]
    fn mean_energy_with_no_cells_is_zero() {
        let subject = World::new(generate_cells(0, 10.0, 0.0, &CellParameters::DEFAULT));
        assert_eq!(subject.mean_energy(), 0.0);
    }

    #[test]
    fn calculate_mean_energy() {
        let subject = World::new(vec![
            Cell::new(&CellParameters::DEFAULT, 1.0, 0.0),
            Cell::new(&CellParameters::DEFAULT, 2.0, 0.0),
        ]);
        assert_eq!(subject.mean_energy(), 1.5);
    }

    #[test]
    fn generate_cells_from_normal_distribution() {
        let cells = generate_cells(100, 100.0, 5.0, &CellParameters::DEFAULT);
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e < 100.0));
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e > 100.0));
    }

    #[test]
    fn dead_cells_disappear() {
        let cell_params = CellParameters {
            energy_use_per_step: 11.0,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(generate_cells(
            10,
            10.0,
            DEFAULT_STD_DEV_INITIAL_ENERGY,
            &cell_params,
        ));
        subject.step(&Environment::DEFAULT);
        assert_eq!(subject.num_alive(), 0);
    }

    #[test]
    fn world_step_reports_num_died() {
        let cell_params = CellParameters {
            energy_use_per_step: 5.0,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(vec![
            Cell::new(&cell_params, 10.0, 0.0),
            Cell::new(&cell_params, 5.0, 0.0),
            Cell::new(&cell_params, 5.0, 0.0),
        ]);
        let (_, num_died) = subject.step(&Environment::DEFAULT);
        assert_eq!(num_died, 2);
    }

    #[test]
    fn cell_uses_energy() {
        let cell_params = CellParameters {
            energy_use_per_step: 5.25,
            ..CellParameters::DEFAULT
        };
        let mut subject = Cell::new(&cell_params, 10.0, 0.0);
        subject.step(&Environment::DEFAULT);
        assert_eq!(subject.energy(), 4.75);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let subject = Cell::new(&CellParameters::DEFAULT, 0.0, 0.0);
        assert!(!subject.is_alive());
    }

    #[test]
    fn cell_absorbs_energy_from_environment() {
        let cell_params = CellParameters {
            energy_use_per_step: 0.0,
            absorption_yield_factor: 2.0,
            ..CellParameters::DEFAULT
        };
        let environment = Environment {
            food_concentration: 3.0,
        };
        let mut subject = Cell::new(&cell_params, 10.0, 2.5);
        subject.step(&environment);
        assert_eq!(subject.energy(), 10.0 + 2.5 * 2.0 * 3.0);
    }
}
