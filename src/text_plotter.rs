fn calculate_bar_width(x: f64, min: f64, max: f64, bar_capacity: usize) -> usize {
    let y = ((x - min) / (max - min)) * (bar_capacity as f64);
    if y < 0.0 {
        0
    } else {
        y as usize
    }
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

    pub fn update(&mut self, x: f64) {
        let str = self.to_string(x);
        self.output.write(str.as_bytes()).unwrap();
        self.output.write(b"\n").unwrap();
    }

    fn to_string(&self, x: f64) -> String {
        let bar_width = calculate_bar_width(x, self.range.min, self.range.max, self.bar_capacity);
        let overflow = bar_width > self.bar_capacity;
        let bar = if overflow {
            // TODO Show overflow icon
            std::iter::repeat("=")
                .take(self.bar_capacity)
                .collect::<String>()
        } else {
            let padding_width = self.bar_capacity - bar_width;
            std::iter::repeat("=")
                .take(bar_width)
                .chain(std::iter::repeat(".").take(padding_width))
                .collect::<String>()
        };
        [bar, format!("{}", x)].join(" ")
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
    fn text_plotter_test() {
        let range = Range::new(0.0, 10.0);
        let output = Vec::new();
        let mut tp = TextPlotter::new(10, range, output);
        tp.update(5.0);
        tp.update(2.0);
        assert_eq!(tp.output, b"=====..... 5\n==........ 2\n");
    }
}
