use std::rc::Rc;
use clap::Parser;
use rand_distr::Normal;
use crate::{cell, CellParameters, ConstantFoodSource, World, world};

const DEFAULT_MEAN_INITIAL_ENERGY: f32 = 100.0;
const DEFAULT_STD_DEV_INITIAL_ENERGY: f32 = 0.0;
const DEFAULT_MEAN_CHILD_THRESHOLD_ENERGY: f32 = f32::MAX;
const DEFAULT_STD_DEV_CHILD_THRESHOLD_ENERGY: f32 = 0.0;
const DEFAULT_MEAN_CHILD_THRESHOLD_FOOD: f32 = 0.0;
const DEFAULT_STD_DEV_CHILD_THRESHOLD_FOOD: f32 = 0.0;
const DEFAULT_MEAN_EATING_ENERGY: f32 = 0.0;
const DEFAULT_STD_DEV_EATING_ENERGY: f32 = 0.0;

pub fn create_and_run_world(args: &Args) {
    let cell_params = Rc::new(CellParameters {
        maintenance_energy_use: args.maint,
        food_yield_from_eating: args.eat_yield,
        energy_yield_from_digestion: args.digest_yield,
        create_child_energy: args.create_child,
    });

    let mut world = create_world(args, &cell_params);

    run(&mut world, args.steps);
}

fn create_world(args: &Args, cell_params: &Rc<CellParameters>) -> World {
    World::new()
        .with_cells(world::generate_cells(
            args.cells,
            Normal::new(args.mean_en, args.sd_en).unwrap(),
            Normal::new(args.mean_eat, args.sd_eat).unwrap(),
            Normal::new(args.mean_child_en, args.sd_child_en).unwrap(),
            Normal::new(args.mean_child_fd, args.sd_child_fd).unwrap(),
            cell_params,
        ))
        .with_food(args.initial_food)
        .with_food_sources(vec![
            Box::new(ConstantFoodSource::new(args.added_food))
        ])
}

fn run(world: &mut World, steps: u32) {
    println!("<step>: +<born> -<died> -> <cells> (e: <mean_cell_energy>, f: <total_food>)");

    let mut step = 0;
    println!("{}: +{} -{} -> {} (e: {}, f: {})",
             step,
             0,
             0,
             world.num_cells(),
             world.mean_energy(),
             world.food());

    while step < steps && world.num_cells() > 0 {
        let (num_created, num_died) = world.step();
        step += 1;
        println!("{}: +{} -{} -> {} (e: {}, f: {})",
                 step,
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
pub struct Args {
    /// Number of steps
    #[clap(short('s'), long, default_value_t = u32::MAX)]
    steps: u32,

    /// Initial world food
    #[clap(short('f'), long, default_value_t = world::DEFAULT_FOOD_AMOUNT)]
    initial_food: f32,

    /// World food added per step
    #[clap(long, default_value_t = 0.0)]
    added_food: f32,

    /// Initial number of cells
    #[clap(short('n'), long, default_value_t = 100)]
    cells: usize,

    /// Mean of cell initial energies
    #[clap(short('e'), long, default_value_t = DEFAULT_MEAN_INITIAL_ENERGY)]
    mean_en: f32,

    /// Standard deviation of cell initial energies
    #[clap(long, default_value_t = DEFAULT_STD_DEV_INITIAL_ENERGY)]
    sd_en: f32,

    /// Mean of child threshold energies
    #[clap(short('C'), long, default_value_t = DEFAULT_MEAN_CHILD_THRESHOLD_ENERGY)]
    mean_child_en: f32,

    /// Standard deviation of child threshold energies
    #[clap(long, default_value_t = DEFAULT_STD_DEV_CHILD_THRESHOLD_ENERGY)]
    sd_child_en: f32,

    /// Mean of child threshold foods
    #[clap(long, default_value_t = DEFAULT_MEAN_CHILD_THRESHOLD_FOOD)]
    mean_child_fd: f32,

    /// Standard deviation of child threshold foods
    #[clap(long, default_value_t = DEFAULT_STD_DEV_CHILD_THRESHOLD_FOOD)]
    sd_child_fd: f32,

    /// Energy cost of creating a child
    #[clap(long, default_value_t = cell::DEFAULT_CREATE_CHILD_ENERGY)]
    create_child: f32,

    /// Cell maintenance energy
    #[clap(short('M'), long, default_value_t = cell::DEFAULT_MAINTENANCE_ENERGY_USE)]
    maint: f32,

    /// Mean of cell eating energies
    #[clap(short('E'), long, default_value_t = DEFAULT_MEAN_EATING_ENERGY)]
    mean_eat: f32,

    /// Standard deviation of cell eating energies
    #[clap(long, default_value_t = DEFAULT_STD_DEV_EATING_ENERGY)]
    sd_eat: f32,

    /// Food gained per unit eating energy
    #[clap(short('F'), long, default_value_t = cell::DEFAULT_FOOD_YIELD_FROM_EATING)]
    eat_yield: f32,

    /// Energy gained per unit food
    #[clap(short('D'), long, default_value_t = cell::DEFAULT_ENERGY_YIELD_FROM_DIGESTION)]
    digest_yield: f32,
}
