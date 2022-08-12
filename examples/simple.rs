use evo2::main_support;
use evo2::main_support::Args;

fn main() {
    let args = Args {
        steps: 20,
        initial_food: 0.0,
        added_food: 0.0,
        cells: 100,
        mean_en: 100.0,
        sd_en: 0.0,
        mean_child_en: f32::MAX,
        sd_child_en: 0.0,
        mean_child_fd: f32::MAX,
        sd_child_fd: 0.0,
        create_child: 0.0,
        maint: 0.0,
        mean_eat: 0.0,
        sd_eat: 0.0,
        eat_yield: 1.0,
        digest_yield: 1.0
    };
    main_support::create_and_run_world(&args);
}
