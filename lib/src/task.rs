pub trait TimeSource {
    fn get_time_ms(&self) -> i128;
}

pub struct Task {
    repeat: bool,
    delay: u64,
    start: i128,
    time_src: Box<dyn TimeSource>
}
