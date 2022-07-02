use cell::{Cell, CellEnvironment, CellParameters};
use clap::Parser;
use rand_distr::Normal;
use world::World;

mod cell;
mod world;

const DEFAULT_MEAN_INITIAL_ENERGY: f32 = 100.0;
const DEFAULT_STD_DEV_INITIAL_ENERGY: f32 = 0.0;

fn main() {
    let args = Args::parse();

    let cell_params = CellParameters {
        energy_use_per_step: args.energy_use,
        ..CellParameters::DEFAULT
    };

    let mut world = World::new()
        .with_cells(world::generate_cells(
            args.cells,
            Normal::new(args.mean_energy, args.std_dev_energy).unwrap(),
            0.0,
            &cell_params,
        ))
        .with_food(args.food_amount);

    run(&mut world);
}

fn run(world: &mut World) {
    println!("+born -died -> cells (e: mean_energy, f: total_food");
    while world.num_cells() > 0 {
        let (num_created, num_died) = world.step();
        println!(
            "+{} -{} -> {} (e: {}, f: {})",
            num_created,
            num_died,
            world.num_cells(),
            world.mean_energy(),
            world.food()
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
    #[clap(short('u'), long, default_value_t = cell::DEFAULT_ENERGY_USE_PER_STEP)]
    energy_use: f32,

    /// Initial amount of food
    #[clap(short('f'), long, default_value_t = world::DEFAULT_FOOD_AMOUNT)]
    food_amount: f32,
}
