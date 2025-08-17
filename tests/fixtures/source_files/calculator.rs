// Simple Rust calculator for testing
pub struct Calculator {
    result: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { result: 0.0 }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        self.result += value;
        self.result
    }

    pub fn subtract(&mut self, value: f64) -> f64 {
        self.result -= value;
        self.result
    }

    pub fn multiply(&mut self, value: f64) -> f64 {
        self.result *= value;
        self.result
    }

    pub fn divide(&mut self, value: f64) -> Result<f64, String> {
        if value == 0.0 {
            Err("Division by zero".to_string())
        } else {
            self.result /= value;
            Ok(self.result)
        }
    }

    pub fn get_result(&self) -> f64 {
        self.result
    }

    pub fn clear(&mut self) {
        self.result = 0.0;
    }
}