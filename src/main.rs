fn main() {
    loop {
        let (num_created, num_died) = World::step();
        let num_alive = World::num_alive();
        println!("+{} -{} -> {}", num_created, num_died, num_alive);
    }
}

pub struct World;

impl World {
    pub fn num_alive() -> i32 {
        100
    }

    pub fn step() -> (i32, i32) {
        (0, 0)
    }
}
