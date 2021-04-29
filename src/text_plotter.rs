fn calculate_bar_width(x: f64, min: f64, max: f64, bar_capacity: usize) -> usize {
    let y = ((x - min) / (max - min)) * (bar_capacity as f64);
    if y < 0.0 {
        0
    } else {
        y as usize
    }
}

fn bar_to_string(bar_width: usize) -> String {
    std::iter::repeat("-").take(bar_width).collect::<String>()
}

fn print_bar(bar_width: usize) {
    println!("{}", bar_to_string(bar_width));
}

#[derive(Clone)]
pub struct TextPlotter<Out: std::io::Write> {
    bar_capacity: usize,
    range: Range,
    output: Out,
}

impl<Out: std::io::Write> TextPlotter<Out> {
    pub fn new(bar_capacity: usize, range: Range, output: Out) -> Self {
        TextPlotter {
            bar_capacity,
            range,
            output,
        }
    }

    pub fn update(&self, x: f64) {
        let bar_width = calculate_bar_width(x, self.range.min, self.range.max, self.bar_capacity);
        print_bar(bar_width);
    }
}

#[derive(Clone)]
pub struct Range {
    min: f64,
    max: f64,
}

impl Range {
    pub fn new(min: f64, max: f64) -> Self {
        Range { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_bar_width_test() {
        assert_eq!(calculate_bar_width(10.0, 0.0, 100.0, 50), 5);
        assert_eq!(calculate_bar_width(50.0, 30.0, 130.0, 100), 20);
        assert_eq!(calculate_bar_width(10.0, 30.0, 100.0, 50), 0);
    }

    #[test]
    fn bar_to_string_test() {
        assert_eq!(bar_to_string(8), "--------");
        assert_eq!(bar_to_string(10), "----------");
    }

    #[test]
    fn terminal_plotter_test() {
        let range = Range::new(0.0, 100.0);
        let output = std::io::stdout();
        let tp = TextPlotter::new(100, range, output.lock());
        tp.update(50.0);
    }
}
