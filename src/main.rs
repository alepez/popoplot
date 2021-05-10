mod terminal_plotter;
mod text_plotter;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use terminal_plotter::TerminalMultiPlotter;
use text_plotter::StdoutTextMultiPlotter;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

impl FromStr for PlotterType {
    type Err = &'static str;

    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "text" => Ok(PlotterType::Text),
            "tui" => Ok(PlotterType::Terminal),
            _ => Err("Invalid plotter type"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum PlotterType {
    Text,
    Terminal,
}

impl Default for PlotterType {
    fn default() -> Self {
        PlotterType::Text
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, default_value = "0")]
    min: f64,

    #[structopt(long, default_value = "100")]
    max: f64,

    #[structopt(long, default_value = "100")]
    width: usize,

    #[structopt(long, default_value = "127.0.0.1:9999")]
    bind: SocketAddr,

    #[structopt(long)]
    multiple_connections: bool,

    #[structopt(long, default_value = "text")]
    view: PlotterType,
}

#[derive(Clone, Copy)]
struct PlotterOpt {
    range: Range,
    width: usize,
}

impl From<Opt> for PlotterOpt {
    fn from(opt: Opt) -> Self {
        Self {
            range: Range::new(opt.min, opt.max),
            width: opt.width,
        }
    }
}

#[derive(Clone, Copy)]
struct Range {
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

    let mult_conn = opt.multiple_connections;

    if mult_conn {
        launch_multiple_connections_server(opt, listener).await
    } else {
        launch_single_connection_server(opt, listener).await
    }
}

fn make_multi_plotter(plotter_type: PlotterType, opt: PlotterOpt) -> Box<dyn MultiPlotter + Send> {
    match plotter_type {
        PlotterType::Text => Box::new(StdoutTextMultiPlotter::new(opt)),
        PlotterType::Terminal => Box::new(TerminalMultiPlotter::new(opt)),
    }
}

async fn launch_multiple_connections_server(
    opt: Opt,
    listener: TcpListener,
) -> Result<(), Box<dyn std::error::Error>> {
    let plotter_type = opt.view;
    let plotter_opt = opt.into();
    let multi_plotter = make_multi_plotter(plotter_type, plotter_opt);
    let multi_plotter = Arc::new(Mutex::new(multi_plotter));

    loop {
        let multi_plotter = multi_plotter.clone();
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let server = Framed::new(socket, LinesCodec::new_with_max_length(1024));
            let tp = multi_plotter.lock().unwrap().spawn();
            process_incoming_data(tp, server).await
        });
    }
}

async fn launch_single_connection_server(
    opt: Opt,
    listener: TcpListener,
) -> Result<(), Box<dyn std::error::Error>> {
    let plotter_type = opt.view;
    let plotter_opt = opt.into();
    let mut multi_plotter = make_multi_plotter(plotter_type, plotter_opt);

    loop {
        let (socket, _) = listener.accept().await?;
        let server = Framed::new(socket, LinesCodec::new_with_max_length(1024));
        let tp = multi_plotter.spawn();
        process_incoming_data(tp, server).await
    }
}

async fn process_incoming_data(
    mut tp: Box<dyn Plotter + Send>,
    mut server: Framed<TcpStream, LinesCodec>,
) {
    while let Some(Ok(line)) = server.next().await {
        if let Ok(x) = line.parse() {
            tp.update(x);
        }
    }
}

trait Plotter {
    fn new(opt: PlotterOpt) -> Self
    where
        Self: Sized;
    fn update(&mut self, y: f64);
}

trait MultiPlotter {
    fn new(opt: PlotterOpt) -> Self
    where
        Self: Sized;

    fn spawn(&mut self) -> Box<dyn Plotter + Send>;
}
