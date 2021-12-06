use crate::{atomic_f64::AtomicF64, eq_core::units::Units};

pub struct Parameter {
    name: String,
    normalized_value: AtomicF64,
    value: AtomicF64,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    display_func: fn(f64) -> String,
    pub transform_func: fn(f64) -> f64,
    pub inv_transform_func: fn(f64) -> f64,
}

impl Parameter {
    pub fn new(
        name: &str,
        default: f64,
        min: f64,
        max: f64,
        display_func: fn(f64) -> String,
        transform_func: fn(f64) -> f64,
        inv_transform_func: fn(f64) -> f64,
    ) -> Parameter {
        Parameter {
            name: String::from(name),
            normalized_value: AtomicF64::new(default.from_range(min, max)),
            value: AtomicF64::new(default),
            default,
            min,
            max,
            display_func,
            transform_func,
            inv_transform_func,
        }
    }

    pub fn get_normalized(&self) -> f64 {
        self.normalized_value.get()
    }

    pub fn get_normalized_default(&self) -> f64 {
        (self.inv_transform_func)(self.default.from_range(self.min, self.max))
    }

    pub fn set_normalized(&self, x: f64) {
        self.normalized_value.set(x);
        self.value
            .set((self.transform_func)(x).to_range(self.min, self.max));
    }

    pub fn get(&self) -> f64 {
        self.value.get()
    }

    pub fn set(&self, x: f64) {
        self.value.set(x);
        self.normalized_value
            .set((self.inv_transform_func)(x.from_range(self.min, self.max)));
    }

    pub fn get_display(&self) -> String {
        (self.display_func)(self.value.get())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
