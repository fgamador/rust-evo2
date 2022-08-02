use clap::Parser;
use evo2::main_support;
use evo2::main_support::Args;

fn main() {
    main_support::create_and_run_world(&Args::parse());
}
