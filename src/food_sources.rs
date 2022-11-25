use crate::number_types::F32Positive;

pub trait FoodSource {
    fn food_this_step(&mut self) -> F32Positive;
}

pub struct ConstantFoodSource {
    food_per_step: F32Positive,
}

impl ConstantFoodSource {
    pub fn new(food_per_step: F32Positive) -> Self {
        ConstantFoodSource {
            food_per_step
        }
    }
}

impl FoodSource for ConstantFoodSource {
    fn food_this_step(&mut self) -> F32Positive {
        self.food_per_step
    }
}

struct LinearlyGrowingFoodSource {
    next_food: F32Positive,
    food_increase_per_step: F32Positive,
}

impl LinearlyGrowingFoodSource {
    pub fn new(starting_food: F32Positive, food_increase_per_step: F32Positive) -> Self {
        LinearlyGrowingFoodSource {
            next_food: starting_food,
            food_increase_per_step,
        }
    }
}

impl FoodSource for LinearlyGrowingFoodSource {
    fn food_this_step(&mut self) -> F32Positive {
        let result = self.next_food;
        self.next_food += self.food_increase_per_step;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linearly_growing_food_source_grows_linearly() {
        let mut source = LinearlyGrowingFoodSource::new(100.0.into(), 10.0.into());
        assert_eq!(source.food_this_step(), 100.0.into());
        assert_eq!(source.food_this_step(), 110.0.into());
    }
}
