mod text_plotter;

use std::net::SocketAddr;
use std::sync::Arc;
use structopt::StructOpt;
use text_plotter::{Range, TextPlotter};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, default_value = "0")]
    min: f64,

    #[structopt(long, default_value = "100")]
    max: f64,

    #[structopt(long, default_value = "100")]
    bar_capacity: usize,

    #[structopt(long, default_value = "127.0.0.1:9999")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let listener = TcpListener::bind(opt.bind).await?;

    let range = Range::new(opt.min, opt.max);
    let output = std::io::stdout();
    let tp = TextPlotter::new(opt.bar_capacity, range, output);
    let tp = Arc::new(tp);

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
