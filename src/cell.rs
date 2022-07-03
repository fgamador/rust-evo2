pub const DEFAULT_ENERGY_USE_PER_STEP: f32 = 0.0;
pub const DEFAULT_EATING_FOOD_YIELD: f32 = 1.0;
pub const DEFAULT_DIGESTION_ENERGY_YIELD: f32 = 1.0;

pub struct Cell<'a> {
    cell_params: &'a CellParameters,
    energy: f32,
    eating_energy_per_step: f32,
}

impl<'a> Cell<'a> {
    pub fn new(cell_params: &'a CellParameters, energy: f32, eating_energy_per_step: f32) -> Self {
        Cell {
            cell_params,
            energy,
            eating_energy_per_step,
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.energy() > 0.0
    }

    pub fn step(&mut self, environment: &CellEnvironment) -> f32 {
        let food = self.eat_food(environment.food_per_cell);
        self.digest_food(food);
        self.use_energy();
        food
    }

    fn eat_food(&mut self, food_per_cell: f32) -> f32 {
        let food_gained = (self.eating_energy_per_step * self.cell_params.eating_food_yield).min(food_per_cell);
        self.energy -= self.eating_energy_per_step;
        food_gained
    }

    fn digest_food(&mut self, food_amount: f32) {
        self.energy += food_amount * self.cell_params.digestion_energy_yield;
    }

    fn use_energy(&mut self) {
        self.energy -= self.cell_params.energy_use_per_step;
    }
}

pub struct CellParameters {
    pub energy_use_per_step: f32,
    pub eating_food_yield: f32,
    pub digestion_energy_yield: f32,
}

impl CellParameters {
    #[allow(dead_code)]
    pub const DEFAULT: CellParameters = CellParameters {
        energy_use_per_step: DEFAULT_ENERGY_USE_PER_STEP,
        eating_food_yield: DEFAULT_EATING_FOOD_YIELD,
        digestion_energy_yield: DEFAULT_DIGESTION_ENERGY_YIELD,
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
            energy_use_per_step: 5.25,
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
            eating_food_yield: 1.5,
            ..CellParameters::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 10.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 1.0, 2.0);
        let food_eaten = cell.step(&environment);
        assert_eq!(food_eaten, 3.0);
    }

    #[test]
    fn cell_cannot_eat_more_food_than_is_available() {
        let cell_params = CellParameters {
            eating_food_yield: 1.0,
            ..CellParameters::DEFAULT
        };
        let environment = CellEnvironment {
            food_per_cell: 2.0,
            ..CellEnvironment::DEFAULT
        };
        let mut cell = Cell::new(&cell_params, 1.0, 3.0);
        let food_eaten = cell.step(&environment);
        assert_eq!(food_eaten, 2.0);
    }

    #[test]
    fn cell_expends_energy_eating() {
        let cell_params = CellParameters {
            eating_food_yield: 0.0,
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
            energy_use_per_step: 0.0,
            eating_food_yield: 1.0,
            digestion_energy_yield: 1.5,
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
