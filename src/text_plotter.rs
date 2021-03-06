use super::Plotter;
use super::PlotterOpt;
use super::Range;
use crate::MultiPlotter;

fn calculate_bar_width(x: f64, min: f64, max: f64, width: usize) -> usize {
    let y = ((x - min) / (max - min)) * (width as f64);
    if y < 0.0 {
        0
    } else {
        y as usize
    }
}

pub type StdoutTextPlotter = TextPlotter<std::io::Stdout>;

impl Plotter for StdoutTextPlotter {
    fn update(&mut self, y: f64) {
        self.update(y);
    }
}

#[derive(Clone)]
pub struct TextPlotter<Out: std::io::Write> {
    width: usize,
    range: Range,
    output: Out,
}

impl<Out: std::io::Write> TextPlotter<Out> {
    fn new(opt: PlotterOpt, output: Out) -> Self {
        TextPlotter {
            width: opt.width,
            range: opt.range,
            output,
        }
    }

    fn update(&mut self, x: f64) {
        let str = self.to_string(x);
        self.output.write(str.as_bytes()).unwrap();
        self.output.write(b"\n").unwrap();
    }

    fn to_string(&self, x: f64) -> String {
        use std::iter::{once, repeat};

        let above_max = x > self.range.max;
        let below_min = x < self.range.min;

        let bar = if above_max {
            once("[")
                .chain(repeat("#").take(self.width))
                .chain(once("X"))
                .collect::<String>()
        } else if below_min {
            once("X")
                .chain(repeat(" ").take(self.width))
                .chain(once("]"))
                .collect::<String>()
        } else {
            let bar_width = calculate_bar_width(x, self.range.min, self.range.max, self.width);
            let padding_width = self.width - bar_width;
            once("[")
                .chain(repeat("#").take(bar_width))
                .chain(repeat(" ").take(padding_width))
                .chain(once("]"))
                .collect::<String>()
        };

        [bar, format!("{}", x)].join(" ")
    }
}

pub(crate) struct StdoutTextMultiPlotter {
    opt: PlotterOpt,
}

impl MultiPlotter for StdoutTextMultiPlotter {
    fn new(opt: PlotterOpt) -> Self
    where
        Self: Sized,
    {
        StdoutTextMultiPlotter { opt }
    }

    fn spawn(&mut self) -> Box<dyn Plotter + Send> {
        Box::new(StdoutTextPlotter::new(self.opt, std::io::stdout()))
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
        let opt = PlotterOpt { width: 10, range };
        let mut tp = TextPlotter::new(opt, output);
        tp.update(5.0);
        tp.update(2.0);
        tp.update(-2.0);
        tp.update(12.0);
        insta::assert_snapshot!(String::from_utf8(tp.output).unwrap());
    }
}
