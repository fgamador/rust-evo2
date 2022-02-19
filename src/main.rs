use clap::Parser;
use rand_distr::{Distribution, Normal};

const DEFAULT_MEAN_INITIAL_ENERGY: f32 = 100.0;
const DEFAULT_STD_DEV_INITIAL_ENERGY: f32 = 0.0;
const DEFAULT_ENERGY_USE_PER_STEP: f32 = 0.0;
const DEFAULT_EATING_FOOD_YIELD: f32 = 1.0;
const DEFAULT_DIGESTION_ENERGY_YIELD: f32 = 1.0;
const DEFAULT_FOOD_AMOUNT: f32 = 0.0;

fn main() {
    let args = Args::parse();

    let cell_params = CellParameters {
        energy_use_per_step: args.energy_use,
        ..CellParameters::DEFAULT
    };

    let mut world = World::new(
        generate_cells(
            args.cells,
            Normal::new(args.mean_energy, args.std_dev_energy).unwrap(),
            0.0,
            &cell_params,
        ),
        args.food_amount,
    );

    while world.num_alive() > 0 {
        let (num_created, num_died) = world.step(&Environment::DEFAULT);
        println!(
            "+{} -{} -> {} (e: {}, f: {})",
            num_created,
            num_died,
            world.num_alive(),
            world.mean_energy(),
            world.food_amount()
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
    food_amount: f32,
}

impl<'a> World<'a> {
    pub fn new(cells: Vec<Cell>, food_amount: f32) -> World {
        World { cells, food_amount }
    }

    pub fn cell(&self, index: usize) -> &Cell {
        &self.cells[index]
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

    pub fn food_amount(&self) -> f32 {
        self.food_amount
    }

    pub fn step(&mut self, environment: &Environment) -> (usize, usize) {
        let mut food_requested = 0.0;
        for cell in &mut self.cells {
            let food = cell.request_food();
            cell.digest_food(food);
            food_requested += food;
        }

        let mut dead_indexes = Vec::with_capacity(self.cells.len());
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.step(environment);
            if !cell.is_alive() {
                dead_indexes.push(index);
            }
        }
        self.remove_cells(&mut dead_indexes);
        self.food_amount -= food_requested;

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
    initial_energies: Normal<f32>,
    eating_energy_per_step: f32,
    cell_params: &CellParameters,
) -> Vec<Cell> {
    let mut rng = rand::thread_rng();
    let mut cells = Vec::with_capacity(num_cells);
    for _ in 0..num_cells {
        cells.push(Cell::new(
            cell_params,
            initial_energies.sample(&mut rng),
            eating_energy_per_step,
        ));
    }
    cells
}

pub struct Cell<'a> {
    cell_params: &'a CellParameters,
    energy: f32,
    eating_energy_per_step: f32,
}

impl<'a> Cell<'a> {
    pub fn new(cell_params: &'a CellParameters, energy: f32, eating_energy_per_step: f32) -> Self {
        Cell {
            cell_params,
            energy,
            eating_energy_per_step,
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy() > 0.0
    }

    pub fn request_food(&self) -> f32 {
        self.eating_energy_per_step * self.cell_params.eating_food_yield
    }

    pub fn digest_food(&mut self, food_amount: f32) {
        self.energy += food_amount * self.cell_params.digestion_energy_yield;
    }

    pub fn step(&mut self, _environment: &Environment) {
        self.energy -= self.cell_params.energy_use_per_step;
    }
}

pub struct CellParameters {
    pub energy_use_per_step: f32,
    pub eating_food_yield: f32,
    pub digestion_energy_yield: f32,
}

impl CellParameters {
    pub const DEFAULT: CellParameters = CellParameters {
        energy_use_per_step: DEFAULT_ENERGY_USE_PER_STEP,
        eating_food_yield: DEFAULT_EATING_FOOD_YIELD,
        digestion_energy_yield: DEFAULT_DIGESTION_ENERGY_YIELD,
    };
}

pub struct Environment {}

impl Environment {
    pub const DEFAULT: Environment = Environment {};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let initial_energies = Normal::new(10.0, 0.0).unwrap();
        let subject = World::new(
            generate_cells(42, initial_energies, 0.0, &CellParameters::DEFAULT),
            0.0,
        );
        assert_eq!(subject.num_alive(), 42);
    }

    #[test]
    fn mean_energy_starts_at_mean_initial_energy() {
        let initial_energies = Normal::new(39.5, 0.0).unwrap();
        let subject = World::new(
            generate_cells(100, initial_energies, 0.0, &CellParameters::DEFAULT),
            0.0,
        );
        assert_eq!(subject.mean_energy(), 39.5);
    }

    #[test]
    fn mean_energy_with_no_cells_is_zero() {
        let initial_energies = Normal::new(10.0, 0.0).unwrap();
        let subject = World::new(
            generate_cells(0, initial_energies, 0.0, &CellParameters::DEFAULT),
            0.0,
        );
        assert_eq!(subject.mean_energy(), 0.0);
    }

    #[test]
    fn calculate_mean_energy() {
        let subject = World::new(
            vec![
                Cell::new(&CellParameters::DEFAULT, 1.0, 0.0),
                Cell::new(&CellParameters::DEFAULT, 2.0, 0.0),
            ],
            0.0,
        );
        assert_eq!(subject.mean_energy(), 1.5);
    }

    #[test]
    fn generate_cells_from_normal_distribution() {
        let initial_energies = Normal::new(100.0, 5.0).unwrap();
        let cells = generate_cells(100, initial_energies, 0.0, &CellParameters::DEFAULT);
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e < 100.0));
        assert!(cells.iter().map(|cell| cell.energy()).any(|e| e > 100.0));
    }

    #[test]
    fn dead_cells_disappear() {
        let cell_params = CellParameters {
            energy_use_per_step: 11.0,
            ..CellParameters::DEFAULT
        };
        let initial_energies = Normal::new(10.0, DEFAULT_STD_DEV_INITIAL_ENERGY).unwrap();
        let mut subject = World::new(generate_cells(10, initial_energies, 0.0, &cell_params), 0.0);
        subject.step(&Environment::DEFAULT);
        assert_eq!(subject.num_alive(), 0);
    }

    #[test]
    fn world_step_reports_num_died() {
        let cell_params = CellParameters {
            energy_use_per_step: 5.0,
            ..CellParameters::DEFAULT
        };
        let mut subject = World::new(
            vec![
                Cell::new(&cell_params, 10.0, 0.0),
                Cell::new(&cell_params, 5.0, 0.0),
                Cell::new(&cell_params, 5.0, 0.0),
            ],
            0.0,
        );
        let (_, num_died) = subject.step(&Environment::DEFAULT);
        assert_eq!(num_died, 2);
    }

    #[test]
    fn cells_consume_world_food() {
        let cell_params = CellParameters {
            eating_food_yield: 1.0,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(
            vec![
                Cell::new(&cell_params, 1.0, 2.0),
                Cell::new(&cell_params, 1.0, 3.0),
            ],
            10.0,
        );
        world.step(&Environment::DEFAULT);
        assert_eq!(world.food_amount(), 5.0);
    }

    #[test]
    #[ignore]
    fn cells_cannot_consume_more_than_their_share_of_world_food() {
        let cell_params = CellParameters {
            energy_use_per_step: 0.0,
            eating_food_yield: 1.0,
            digestion_energy_yield: 1.0,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(
            vec![
                Cell::new(&cell_params, 10.0, 2.0),
                Cell::new(&cell_params, 10.0, 3.0),
            ],
            4.0,
        );
        world.step(&Environment::DEFAULT);
        assert_eq!(world.food_amount(), 0.0);
        assert_eq!(world.cell(0).energy(), 12.0);
        assert_eq!(world.cell(1).energy(), 12.0);
    }

    #[test]
    fn cells_gain_energy_from_eating_world_food() {
        let cell_params = CellParameters {
            eating_food_yield: 1.0,
            digestion_energy_yield: 2.0,
            ..CellParameters::DEFAULT
        };
        let mut world = World::new(vec![Cell::new(&cell_params, 10.0, 1.0)], 10.0);
        world.step(&Environment::DEFAULT);
        assert_eq!(world.mean_energy(), 12.0);
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
    fn cell_requests_food() {
        let cell_params = CellParameters {
            eating_food_yield: 1.5,
            ..CellParameters::DEFAULT
        };
        let cell = Cell::new(&cell_params, 1.0, 2.0);
        assert_eq!(cell.request_food(), 3.0);
    }

    #[test]
    fn cell_digests_food() {
        let cell_params = CellParameters {
            digestion_energy_yield: 1.5,
            ..CellParameters::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 10.0, 0.0);
        cell.digest_food(3.0);
        assert_eq!(cell.energy(), 14.5);
    }
}
