fn main() {
    let mut world = World::new(100, 100.0);
    loop {
        let (num_created, num_died) = world.step();
        println!("+{} -{} -> {} (e: {})", num_created, num_died, world.num_alive(),
                 world.average_energy());
    }
}

pub struct World {
    average_energy: f64,
    cells: Vec<Cell>,
}

impl World {
    pub fn new(num_cells: usize, average_energy: f64) -> Self {
        World {
            average_energy,
            cells: vec![Cell { energy: average_energy }; num_cells],
        }
    }

    pub fn num_alive(&self) -> usize {
        self.cells.len()
    }

    pub fn average_energy(&self) -> f64 {
        self.cells[0].energy
    }

    pub fn step(&mut self) -> (i32, i32) {
        (0, 0)
    }
}

#[derive(Clone)]
pub struct Cell {
    pub energy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_cells_start_alive() {
        let world = World::new(42, 100.0);
        assert_eq!(world.num_alive(), 42);
    }

    #[test]
    fn average_energy_starts_as_specified() {
        let world = World::new(100, 39.5);
        assert_eq!(world.average_energy(), 39.5);
    }
}
