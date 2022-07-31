pub trait FoodSource {
    fn food_this_step(&self) -> f32;
}

pub struct ConstantFoodSource {
    food_per_step: f32,
}

impl ConstantFoodSource {
    pub fn new(food_per_step: f32) -> Self {
        ConstantFoodSource {
            food_per_step
        }
    }
}

impl FoodSource for ConstantFoodSource {
    fn food_this_step(&self) -> f32 {
        self.food_per_step
    }
}
