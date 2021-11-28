/// Maintains a moving average.
///
/// This is partly based on the discussion here:
///
/// https://stackoverflow.com/questions/12636613/
#[derive(Debug)]
pub struct MovingAverage {
    max_counter: usize,
    counter: usize,
    pub average: f64,
}

impl MovingAverage {
    pub fn new(max_counter: usize) -> Self {
        MovingAverage {
            max_counter: max_counter,
            counter: 0,
            average: 0.0,
        }
    }

    pub fn add(&mut self, x: f64) {
        // n is the number of samples we are tracking, up to
        // self.max_counter
        let n = if self.counter == self.max_counter {
            self.max_counter as i64
        } else {
            self.counter += 1;
            self.counter as i64
        };
        // This is obviously the the approximate sum of the last n - 1 values and x:
        //   (n - 1) * self.average + x
        // Thus the approximate average of the last n - 1 values and x:
        //   ((n - 1) * self.average  + x) / n
        // = (n * self.average - self.average + x) / n
        // = (n * self.average + x - self.average) / n
        // = self.average + (x - self.average) / n
        //
        // The final formula is on https://stackoverflow.com/a/50854247.
        self.average = self.average + (x - self.average) / (n as f64);
    }
}
