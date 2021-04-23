use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

fn calculate_bar_capacity(x: f64, min: f64, max: f64, bar_capacity: f64) -> f64 {
    let y = ((x - min) / (max - min)) * bar_capacity;
    if y < 0.0 { 0.0 } else { y }
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
    fn calculate_bar_capacity_test() {
        assert_eq!(calculate_bar_capacity(10.0, 0.0, 100.0, 50.0), 5.0);
        assert_eq!(calculate_bar_capacity(50.0, 30.0, 130.0, 100.0), 20.0);
        assert_eq!(calculate_bar_capacity(10.0, 30.0, 100.0, 50.0), 0.0);
    }

    #[test]
    fn bar_to_string_test() {
        assert_eq!(bar_to_string(8), "--------");
        assert_eq!(bar_to_string(10), "----------");
    }
}