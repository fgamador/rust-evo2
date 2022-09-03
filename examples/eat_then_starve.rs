use evo2::main_support::{create_and_run_world, Args};

fn main() {
    create_and_run_world(&Args {
        initial_food: 100.0,
        cells: 1,
        initial_energy_mean: 10.0,
        attempted_eating_energy_mean: 1.0,
        food_yield_from_eating: 10.0,
        energy_yield_from_digestion: 1.0,
        health_reduction_per_energy_expended: 0.1,
        ..Args::DEFAULT
    });
}