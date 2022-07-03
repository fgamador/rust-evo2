pub const DEFAULT_MAINTENANCE_ENERGY_USE: f32 = 0.0;
pub const DEFAULT_FOOD_YIELD_FROM_EATING: f32 = 1.0;
pub const DEFAULT_ENERGY_YIELD_FROM_DIGESTION: f32 = 1.0;

pub struct Cell<'a> {
    cell_params: &'a CellParameters,
    energy: f32,
    attempted_eating_energy: f32,
}

impl<'a> Cell<'a> {
    pub fn new(cell_params: &'a CellParameters, energy: f32, attempted_eating_energy: f32) -> Self {
        Cell {
            cell_params,
            energy,
            attempted_eating_energy,
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy() > 0.0
    }

    pub fn step(&mut self, environment: &CellEnvironment) -> (Option<Cell>, f32) {
        let food = self.eat(environment.food_per_cell);
        self.digest(food);
        self.maintain();
        (None, food)
    }

    fn eat(&mut self, food_per_cell: f32) -> f32 {
        self.energy -= self.attempted_eating_energy;
        (self.attempted_eating_energy * self.cell_params.food_yield_from_eating).min(food_per_cell)
    }

    fn digest(&mut self, food_amount: f32) {
        self.energy += food_amount * self.cell_params.energy_yield_from_digestion;
    }

    fn maintain(&mut self) {
        self.energy -= self.cell_params.maintenance_energy_use;
    }
}

pub struct CellParameters {
    pub maintenance_energy_use: f32,
    pub food_yield_from_eating: f32,
    pub energy_yield_from_digestion: f32,
}

impl CellParameters {
    #[allow(dead_code)]
    pub const DEFAULT: CellParameters = CellParameters {
        maintenance_energy_use: DEFAULT_MAINTENANCE_ENERGY_USE,
        food_yield_from_eating: DEFAULT_FOOD_YIELD_FROM_EATING,
        energy_yield_from_digestion: DEFAULT_ENERGY_YIELD_FROM_DIGESTION,
    };
}

pub struct CellEnvironment {
    pub food_per_cell: f32,
}

impl CellEnvironment {
    #[allow(dead_code)]
    pub const DEFAULT: CellEnvironment = CellEnvironment { food_per_cell: 0.0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_uses_energy() {
        let cell_params = CellParameters {
            maintenance_energy_use: 5.25,
            ..CellParameters::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 10.0, 0.0);
        cell.step(&CellEnvironment::DEFAULT);
        assert_eq!(cell.energy(), 4.75);
    }

    #[test]
    fn cell_with_no_energy_is_dead() {
        let cell = Cell::new(&CellParameters::DEFAULT, 0.0, 0.0);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_eats_food() {
        let cell_params = CellParameters {
            food_yield_from_eating: 1.5,
            ..CellParameters::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 1.0, 2.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 3.0);
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let cell_params = CellParameters {
            food_yield_from_eating: 1.0,
            ..CellParameters::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 2.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 1.0, 3.0);
        let (_, food_eaten) = cell.step(&environment);
        assert_eq!(food_eaten, 2.0);
    }

    #[test]
    fn cell_expends_energy_eating() {
        let cell_params = CellParameters {
            food_yield_from_eating: 0.0,
            ..CellParameters::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 5.0, 2.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 3.0);
    }

    #[test]
    fn cell_digests_food() {
        let cell_params = CellParameters {
            maintenance_energy_use: 0.0,
            food_yield_from_eating: 1.0,
            energy_yield_from_digestion: 1.5,
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 10.0, 2.0);
        cell.step(&environment);
        assert_eq!(cell.energy(), 11.0);
    }
}
