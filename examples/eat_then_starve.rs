use evo2::cell::{Cell, CellConstants, CellParams};
use evo2::main_support::run;
use evo2::world::World;
use std::rc::Rc;

fn main() {
    let mut world = World::new()
        .with_cells(vec![
            Cell::new(
                &Rc::new(CellConstants {
                    energy_yield_from_digestion: 0.5.into(),
                    food_yield_from_eating: 10.into(),
                    health_increase_per_healing_energy: 0.5.into(),
                    health_reduction_from_entropy: 0.5.into(),
                    health_reduction_per_energy_expended: 0.1.into(),
                    ..CellConstants::DEFAULT
                }),
                CellParams {
                    attempted_eating_energy: 1.into(),
                    attempted_healing_energy: 2.into(),
                    ..CellParams::DEFAULT
                })
                .with_energy(10.into()),
        ])
        .with_food(50.into());

    run(&mut world, 1000);
}
