use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let range = Range::new(0.0, 100.0);
    let tp = TerminalPlotter::new(80, range);

    loop {
        let (socket, _) = listener.accept().await?;
        let tp = tp.clone();

        tokio::spawn(async move {
            let mut server = Framed::new(socket, LinesCodec::new_with_max_length(1024));

            while let Some(Ok(line)) = server.next().await {
                if let Ok(x) = line.parse() {
                    tp.update(x);
                }
            }
        });
    }
}

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
struct TerminalPlotter {
    bar_capacity: usize,
    range: Range,
}

impl TerminalPlotter {
    fn new(bar_capacity: usize, range: Range) -> Self {
        TerminalPlotter {
            bar_capacity,
            range,
        }
    }

    fn update(&self, x: f64) {
        let bar_width = calculate_bar_width(x, self.range.min, self.range.max, self.bar_capacity);
        print_bar(bar_width);
    }
}

#[derive(Clone)]
struct Range {
    min: f64,
    max: f64,
}

impl Range {
    fn new(min: f64, max: f64) -> Self {
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
        let tp = TerminalPlotter::new(100, range);
        tp.update(50.0);
    }
}
