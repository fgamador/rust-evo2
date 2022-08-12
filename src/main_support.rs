use clap::Parser;
use rand_distr::Normal;
use std::rc::Rc;
use crate::cell::CellConstants;
use crate::food_sources::ConstantFoodSource;
use crate::world;
use crate::world::World;

pub fn create_and_run_world(args: &Args) {
    let bio_constants = Rc::new(CellConstants {
        maintenance_energy_use: args.maint,
        food_yield_from_eating: args.eat_yield,
        energy_yield_from_digestion: args.digest_yield,
        create_child_energy: args.create_child,
        ..CellConstants::DEFAULT
    });

    let mut world = create_world(args, &bio_constants);

    run(&mut world, args.steps);
}

fn create_world(args: &Args, bio_constants: &Rc<CellConstants>) -> World {
    World::new()
        .with_cells(world::generate_cells(
            args.cells,
            Normal::new(args.mean_en, args.sd_en).unwrap(),
            Normal::new(args.mean_eat, args.sd_eat).unwrap(),
            Normal::new(args.mean_child_en, args.sd_child_en).unwrap(),
            Normal::new(args.mean_child_fd, args.sd_child_fd).unwrap(),
            bio_constants,
        ))
        .with_food(args.initial_food)
        .with_food_sources(vec![
            Box::new(ConstantFoodSource::new(args.added_food))
        ])
}

fn run(world: &mut World, steps: u32) {
    print_stats_header();

    let mut step = 0;
    print_stats(world, step, 0, 0);

    while step < steps && world.num_cells() > 0 {
        let (num_created, num_died) = world.step();
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

    /// Mean of cell initial energies
    #[clap(short('e'), long, default_value_t = Args::DEFAULT.mean_en)]
    pub mean_en: f32,

    /// Standard deviation of cell initial energies
    #[clap(long, default_value_t = Args::DEFAULT.sd_en)]
    pub sd_en: f32,

    /// Mean of child threshold energies
    #[clap(short('C'), long, default_value_t = Args::DEFAULT.mean_child_en)]
    pub mean_child_en: f32,

    /// Standard deviation of child threshold energies
    #[clap(long, default_value_t = Args::DEFAULT.sd_child_en)]
    pub sd_child_en: f32,

    /// Mean of child threshold foods
    #[clap(long, default_value_t = Args::DEFAULT.mean_child_fd)]
    pub mean_child_fd: f32,

    /// Standard deviation of child threshold foods
    #[clap(long, default_value_t = Args::DEFAULT.sd_child_fd)]
    pub sd_child_fd: f32,

    /// Energy cost of creating a child
    #[clap(long, default_value_t = Args::DEFAULT.create_child)]
    pub create_child: f32,

    /// Cell maintenance energy
    #[clap(short('M'), long, default_value_t = Args::DEFAULT.maint)]
    pub maint: f32,

    /// Mean of cell eating energies
    #[clap(short('E'), long, default_value_t = Args::DEFAULT.mean_eat)]
    pub mean_eat: f32,

    /// Standard deviation of cell eating energies
    #[clap(long, default_value_t = Args::DEFAULT.sd_eat)]
    pub sd_eat: f32,

    /// Food gained per unit eating energy
    #[clap(short('F'), long, default_value_t = Args::DEFAULT.eat_yield)]
    pub eat_yield: f32,

    /// Energy gained per unit food
    #[clap(short('D'), long, default_value_t = Args::DEFAULT.digest_yield)]
    pub digest_yield: f32,
}

impl Args {
    #[allow(dead_code)]
    pub const DEFAULT: Args = Args {
        steps: u32::MAX,
        initial_food: 0.0,
        added_food: 0.0,
        cells: 100,
        mean_en: 100.0,
        sd_en: 0.0,
        mean_child_en: f32::MAX,
        sd_child_en: 0.0,
        mean_child_fd: f32::MAX,
        sd_child_fd: 0.0,
        create_child: CellConstants::DEFAULT.create_child_energy,
        maint: CellConstants::DEFAULT.maintenance_energy_use,
        mean_eat: 0.0,
        sd_eat: 0.0,
        eat_yield: CellConstants::DEFAULT.food_yield_from_eating,
        digest_yield: CellConstants::DEFAULT.energy_yield_from_digestion,
    };
}
