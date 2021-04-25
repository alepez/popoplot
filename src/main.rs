use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut server = Framed::new(socket, LinesCodec::new_with_max_length(1024));

            while let Some(Ok(line)) = server.next().await {
                if let Ok(x) = line.parse() {
                    let bar_width = calculate_bar_width(x, 0.0, 100.0, 80.0) as usize;
                    print_bar(bar_width);
                }
            }
        });
    }
}

fn calculate_bar_width(x: f64, min: f64, max: f64, bar_capacity: f64) -> f64 {
    let y = ((x - min) / (max - min)) * bar_capacity;
    if y < 0.0 {
        0.0
    } else {
        y
    }
}

fn bar_to_string(bar_width: usize) -> String {
    std::iter::repeat("-").take(bar_width).collect::<String>()
}

fn print_bar(bar_width: usize) {
    println!("{}", bar_to_string(bar_width));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_bar_width_test() {
        assert_eq!(calculate_bar_width(10.0, 0.0, 100.0, 50.0), 5.0);
        assert_eq!(calculate_bar_width(50.0, 30.0, 130.0, 100.0), 20.0);
        assert_eq!(calculate_bar_width(10.0, 30.0, 100.0, 50.0), 0.0);
    }

    #[test]
    fn bar_to_string_test() {
        assert_eq!(bar_to_string(8), "--------");
        assert_eq!(bar_to_string(10), "----------");
    }
}

