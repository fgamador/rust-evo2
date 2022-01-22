fn main() {
    let mut world = World::new(100);
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
    pub fn new(num_cells: usize) -> Self {
        World { cells: vec![Cell; num_cells] }
    }

    pub fn num_alive(&self) -> usize {
        self.cells.len()
    }

    pub fn average_energy(&self) -> f64 {
        100.0
    }

    pub fn step(&mut self) -> (i32, i32) {
        (0, 0)
    }
}

#[derive(Clone)]
pub struct Cell;

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn world_cells_start_alive() {
//         let mut world = World::new(42);
//         assert_eq!(world.num_alive(), 42);
//     }
// }
