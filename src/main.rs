use clap::Parser;
use evo2::main_support::{create_and_run_world, Args};

fn main() {
    create_and_run_world(&Args::parse());
}
