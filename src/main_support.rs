use clap::Parser;
use rand_distr::Normal;
use std::rc::Rc;
use crate::cell::{CellConstants, RandomMutationNumberSource};
use crate::food_sources::ConstantFoodSource;
use crate::world;
use crate::world::World;

pub fn create_and_run_world(args: &Args) {
    let cell_constants = Rc::new(CellConstants {
        create_child_energy: args.create_child_energy.into(),
        energy_yield_from_digestion: args.energy_yield_from_digestion.into(),
        food_yield_from_eating: args.food_yield_from_eating.into(),
        health_increase_per_healing_energy: args.health_increase_per_healing_energy.into(),
        health_reduction_from_entropy: args.health_reduction_from_entropy.into(),
        health_reduction_per_energy_expended: args.health_reduction_per_energy_expended.into(),
        ..CellConstants::DEFAULT
    });

    let mut world = create_world(args, &cell_constants);

    run(&mut world, args.steps);
}

fn create_world(args: &Args, cell_constants: &Rc<CellConstants>) -> World {
    World::new()
        .with_cells(world::generate_cells(
            args.cells,
            Normal::new(args.initial_energy_mean, args.initial_energy_stdev).unwrap(),
            Normal::new(args.attempted_eating_energy_mean, args.attempted_eating_energy_stdev).unwrap(),
            Normal::new(args.attempted_healing_energy_mean, args.attempted_healing_energy_stdev).unwrap(),
            Normal::new(args.child_threshold_energy_mean, args.child_threshold_energy_stdev).unwrap(),
            Normal::new(args.child_threshold_food_mean, args.child_threshold_food_stdev).unwrap(),
            cell_constants,
        ))
        .with_food(args.initial_food.into())
        .with_food_sources(vec![
            Box::new(ConstantFoodSource::new(args.added_food.into()))
        ])
}

pub fn run(world: &mut World, steps: u32) {
    print_stats_header();

    let mut step = 0;
    print_stats(world, step, 0, 0);

    let mut mutation_number_source = RandomMutationNumberSource::new();
    while step < steps && world.num_cells() > 0 {
        let (num_created, num_died) = world.step(&mut mutation_number_source);
        step += 1;
        print_stats(world, step, num_created, num_died);
    }
}

fn print_stats_header() {
    println!("<step>: +<born> -<died> -> <cells> (h: <mean_cell_health>, e: <mean_cell_energy>, f: <total_food>)");
}

fn print_stats(world: &World, step: u32, num_created: usize, num_died: usize) {
    println!("{}: +{} -{} -> {} (h: {}, e: {}, f: {})",
             step,
             num_created,
             num_died,
             world.num_cells(),
             world.mean_health(),
             world.mean_energy(),
             world.food()
    );
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Number of steps
    #[clap(short('s'), long, default_value_t = Args::DEFAULT.steps)]
    pub steps: u32,

    /// Initial world food
    #[clap(short('f'), long, default_value_t = Args::DEFAULT.initial_food)]
    pub initial_food: f32,

    /// World food added per step
    #[clap(long, default_value_t = Args::DEFAULT.added_food)]
    pub added_food: f32,

    /// Initial number of cells
    #[clap(short('n'), long, default_value_t = Args::DEFAULT.cells)]
    pub cells: usize,

    /// Mean of cell eating energies
    #[clap(short('E'), long, default_value_t = Args::DEFAULT.attempted_eating_energy_mean)]
    pub attempted_eating_energy_mean: f32,

    /// Standard deviation of cell eating energies
    #[clap(long, default_value_t = Args::DEFAULT.attempted_eating_energy_stdev)]
    pub attempted_eating_energy_stdev: f32,

    /// Mean of cell healing energies
    #[clap(long, default_value_t = Args::DEFAULT.attempted_healing_energy_mean)]
    pub attempted_healing_energy_mean: f32,

    /// Standard deviation of cell healing energies
    #[clap(long, default_value_t = Args::DEFAULT.attempted_healing_energy_stdev)]
    pub attempted_healing_energy_stdev: f32,

    /// Mean of child threshold energies
    #[clap(short('C'), long, default_value_t = Args::DEFAULT.child_threshold_energy_mean)]
    pub child_threshold_energy_mean: f32,

    /// Standard deviation of child threshold energies
    #[clap(long, default_value_t = Args::DEFAULT.child_threshold_energy_stdev)]
    pub child_threshold_energy_stdev: f32,

    /// Mean of child threshold foods
    #[clap(long, default_value_t = Args::DEFAULT.child_threshold_food_mean)]
    pub child_threshold_food_mean: f32,

    /// Standard deviation of child threshold foods
    #[clap(long, default_value_t = Args::DEFAULT.child_threshold_food_stdev)]
    pub child_threshold_food_stdev: f32,

    /// Energy cost of creating a child
    #[clap(long, default_value_t = Args::DEFAULT.create_child_energy)]
    pub create_child_energy: f32,

    /// Energy gained per unit food
    #[clap(short('D'), long, default_value_t = Args::DEFAULT.energy_yield_from_digestion)]
    pub energy_yield_from_digestion: f32,

    /// Food gained per unit eating energy
    #[clap(short('F'), long, default_value_t = Args::DEFAULT.food_yield_from_eating)]
    pub food_yield_from_eating: f32,

    /// Health increase per energy expended
    #[clap(long, default_value_t = Args::DEFAULT.health_increase_per_healing_energy)]
    pub health_increase_per_healing_energy: f32,

    /// Health reduction due to entropy
    #[clap(long, default_value_t = Args::DEFAULT.health_reduction_from_entropy)]
    pub health_reduction_from_entropy: f32,

    /// Health reduction per energy expended
    #[clap(long, default_value_t = Args::DEFAULT.health_reduction_per_energy_expended)]
    pub health_reduction_per_energy_expended: f32,

    /// Mean of cell initial energies
    #[clap(short('e'), long, default_value_t = Args::DEFAULT.initial_energy_mean)]
    pub initial_energy_mean: f32,

    /// Standard deviation of cell initial energies
    #[clap(long, default_value_t = Args::DEFAULT.initial_energy_stdev)]
    pub initial_energy_stdev: f32,
}

impl Args {
    #[allow(dead_code)]
    pub const DEFAULT: Args = Args {
        steps: u32::MAX,
        initial_food: 0.0,
        added_food: 0.0,
        cells: 100,
        attempted_eating_energy_mean: 0.0,
        attempted_eating_energy_stdev: 0.0,
        attempted_healing_energy_mean: 0.0,
        attempted_healing_energy_stdev: 0.0,
        child_threshold_energy_mean: f32::MAX,
        child_threshold_energy_stdev: 0.0,
        child_threshold_food_mean: 0.0,
        child_threshold_food_stdev: 0.0,
        create_child_energy: CellConstants::DEFAULT.create_child_energy.value(),
        energy_yield_from_digestion: CellConstants::DEFAULT.energy_yield_from_digestion.value(),
        food_yield_from_eating: CellConstants::DEFAULT.food_yield_from_eating.value(),
        health_increase_per_healing_energy: CellConstants::DEFAULT.health_increase_per_healing_energy.value(),
        health_reduction_from_entropy: CellConstants::DEFAULT.health_reduction_from_entropy.value(),
        health_reduction_per_energy_expended: CellConstants::DEFAULT.health_reduction_per_energy_expended.value(),
        initial_energy_mean: 100.0,
        initial_energy_stdev: 0.0,
    };
}
