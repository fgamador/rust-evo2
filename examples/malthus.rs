use evo2::main_support::{create_and_run_world, Args};

fn main() {
    create_and_run_world(&Args {
        cells: 1,
        mean_en: 10.0,
        initial_food: 100.0,
        mean_eat: 1.0,
        eat_yield: 10.0,
        digest_yield: 1.0,
        mean_child_en: 2.0,
        create_child: 2.0,
        maint: 1.0,
        ..Args::DEFAULT
    });
}
