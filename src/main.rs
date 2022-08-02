use cell::{Cell, CellEnvironment, CellParameters};
use clap::Parser;
use main_support::Args;
use world::World;
use crate::food_sources::ConstantFoodSource;

mod cell;
mod food_sources;
mod main_support;
mod world;

fn main() {
    main_support::create_and_run_world(&Args::parse());
}
