pub trait StringToNumber {
    fn string_to_number(&self) -> f64;
}

impl StringToNumber for &str {
    fn string_to_number(&self) -> f64 {
        self.parse::<f64>().unwrap_or(f64::NAN)
    }
}
