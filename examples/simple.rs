use evo2::main_support;
use evo2::main_support::Args;

fn main() {
    let args = Args {
        steps: 20,
        ..Args::DEFAULT
    };
    main_support::create_and_run_world(&args);
}
