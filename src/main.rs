fn main() {
    loop {
        let (num_created, num_died) = step();
        let num_alive = num_alive();
        println!("+{} -{} -> {}", num_created, num_died, num_alive);
    }
}

fn num_alive() -> i32 {
    100
}

fn step() -> (i32, i32) {
    (0, 0)
}
