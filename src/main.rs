fn main() {
    let mut world = World::new(100, CellParameters::DEFAULT);
    loop {
        let (num_created, num_died) = world.step();
        println!("+{} -{} -> {} (e: {})", num_created, num_died, world.num_alive(),
                 world.average_energy());
    }
}

pub struct World {
    cells: Vec<Cell>,
}

impl World {
    pub fn new(num_cells: usize, cell_params: CellParameters) -> Self {
        World {
            cells: vec![Cell { energy: cell_params.initial_energy }; num_cells],
        }
    }

    pub fn num_alive(&self) -> usize {
        self.cells.len()
    }

    pub fn average_energy(&self) -> f32 {
        self.cells[0].energy
    }

    pub fn step(&mut self) -> (i32, i32) {
        (0, 0)
    }
}

#[derive(Clone)]
pub struct Cell {
    pub energy: f32,
}

pub struct CellParameters {
    pub initial_energy: f32,
}

impl CellParameters {
    pub const DEFAULT: CellParameters = CellParameters {
        initial_energy: 100.0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let world = World::new(42, CellParameters::DEFAULT);
        assert_eq!(world.num_alive(), 42);
    }

    #[test]
    fn average_energy_starts_at_initial_energy() {
        let world = World::new(100, CellParameters {
            initial_energy: 39.5,
            ..CellParameters::DEFAULT
        });
        assert_eq!(world.average_energy(), 39.5);
    }

    // #[test]
    // fn cells_use_energy() {
    //     let mut world = World::new(100, 10.0, 5.25);
    //     world.step();
    //     assert_eq!(world.average_energy(), 4.75);
    // }
}
