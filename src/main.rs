mod terminal_plotter;
mod text_plotter;

use std::io::Stdout;
use std::net::SocketAddr;
use structopt::StructOpt;
use text_plotter::TextPlotter;
use tokio::net::{TcpListener, TcpStream};
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

    #[structopt(long)]
    multiple_connections: bool,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let listener = TcpListener::bind(opt.bind).await?;

    let range = Range::new(opt.min, opt.max);

    let mult_conn = opt.multiple_connections;

    if mult_conn {
        launch_multiple_connections_server(opt, listener, range).await
    } else {
        launch_single_connection_server(opt, listener, range).await
    }
}

async fn launch_multiple_connections_server(
    opt: Opt,
    listener: TcpListener,
    range: Range,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (socket, _) = listener.accept().await?;
        let range = range.clone();
        let bar_capacity = opt.bar_capacity;

        tokio::spawn(async move {
            let server = Framed::new(socket, LinesCodec::new_with_max_length(1024));
            let tp = TextPlotter::new(bar_capacity, range, std::io::stdout());
            process_incoming_data(tp, server).await
        });
    }
}

async fn launch_single_connection_server(
    opt: Opt,
    listener: TcpListener,
    range: Range,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (socket, _) = listener.accept().await?;
        let server = Framed::new(socket, LinesCodec::new_with_max_length(1024));
        let tp = TextPlotter::new(opt.bar_capacity, range.clone(), std::io::stdout());
        process_incoming_data(tp, server).await
    }
}

async fn process_incoming_data(
    mut tp: TextPlotter<Stdout>,
    mut server: Framed<TcpStream, LinesCodec>,
) {
    while let Some(Ok(line)) = server.next().await {
        if let Ok(x) = line.parse() {
            tp.update(x);
        }
    }
}
