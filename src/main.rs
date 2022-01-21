fn main() {
    let mut world = World;
    loop {
        let (num_created, num_died) = world.step();
        let num_alive = world.num_alive();
        println!("+{} -{} -> {}", num_created, num_died, num_alive);
    }
}

pub struct World;

impl World {
    pub fn num_alive(&self) -> i32 {
        100
    }

    pub fn step(&mut self) -> (i32, i32) {
        (0, 0)
    }
}
