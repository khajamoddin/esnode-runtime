pub trait MetricsHook: Send + Sync {
    fn counter(&self, name: &str, value: u64, attrs: &[(&str, &str)]);
    fn histogram(&self, name: &str, value: f64, attrs: &[(&str, &str)]);
}

pub trait TraceHook: Send + Sync {
    fn event(&self, name: &str, attrs: &[(&str, &str)]);
}
