use crate::number_types::F32Positive;

pub trait FoodSource {
    fn food_this_step(&self) -> F32Positive;
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
    fn food_this_step(&self) -> F32Positive {
        self.food_per_step
    }
}
