fn main() {
    let mut world = World::new();
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
    pub fn new() -> Self {
        World { cells: vec![Cell; 100] }
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