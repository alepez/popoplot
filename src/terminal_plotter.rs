use super::Plotter;
use super::PlotterOpt;
use crate::MultiPlotter;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};
use std::collections::vec_deque::VecDeque;
use std::error::Error;

type Sender = tokio::sync::mpsc::UnboundedSender<HistoryRecord>;

// FIXME Is this valid?
unsafe impl Send for TerminalPlotter {}

#[derive(Default)]
struct History(VecDeque<Record>);

type Record = (f64, f64);

struct HistoryRecord {
    history_id: usize,
    record: Record,
}

pub struct TerminalPlotter {
    history_id: usize,
    tx: Sender,
}

impl Plotter for TerminalPlotter {
    fn update(&mut self, y: f64) {
        let record = (0.0, y);
        let hr = HistoryRecord {
            record,
            history_id: self.history_id,
        };
        self.tx.send(hr);
    }
}

impl TerminalPlotter {
    fn new(tx: Sender, history_id: usize) -> Self {
        TerminalPlotter { tx, history_id }
    }
}

#[derive(Copy, Clone)]
enum PixelState {
    Empty,
    HLine,
    VLine,
    Cross,
    Pixel,
    Text(char),
    Circle(bool),
}

impl PixelState {
    fn to_char(self) -> char {
        match self {
            Self::Empty => ' ',
            Self::HLine => '-',
            Self::VLine => '|',
            Self::Cross => '+',
            Self::Pixel => '.',
            Self::Text(c) => c,
            Self::Circle(filled) => {
                if filled {
                    '@'
                } else {
                    'O'
                }
            }
        }
    }

    fn update(&mut self, new_state: PixelState) {
        let next_state = match (*self, new_state) {
            (Self::HLine, Self::VLine) => Self::Cross,
            (Self::VLine, Self::HLine) => Self::Cross,
            (_, Self::Circle(what)) => Self::Circle(what),
            (Self::Circle(what), _) => Self::Circle(what),
            (_, Self::Pixel) => Self::Pixel,
            (Self::Pixel, _) => Self::Pixel,
            (_, new) => new,
        };

        *self = next_state;
    }
}

pub struct TextDrawingBackend {
    state: Vec<PixelState>,
    width: usize,
}

impl DrawingBackend for TextDrawingBackend {
    type ErrorType = std::io::Error;

    fn get_size(&self) -> (u32, u32) {
        (self.width as u32, 30)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        let w = self.width as usize;
        for r in 0..30 {
            let mut buf = String::new();
            for c in 0..w {
                buf.push(self.state[r * w + c].to_char());
            }
            println!("{}", buf);
        }

        self.state.fill(PixelState::Empty);

        Ok(())
    }

    fn draw_pixel(
        &mut self,
        pos: (i32, i32),
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<std::io::Error>> {
        let w = self.width as i32;
        if color.alpha > 0.3 {
            self.state[(pos.1 * w + pos.0) as usize].update(PixelState::Pixel);
        }
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let w = self.width as i32;
        if from.0 == to.0 {
            let x = from.0;
            let y0 = from.1.min(to.1);
            let y1 = from.1.max(to.1);
            for y in y0..y1 {
                self.state[(y * w + x) as usize].update(PixelState::VLine);
            }
            return Ok(());
        }

        if from.1 == to.1 {
            let y = from.1;
            let x0 = from.0.min(to.0);
            let x1 = from.0.max(to.0);
            for x in x0..x1 {
                self.state[(y * w + x) as usize].update(PixelState::HLine);
            }
            return Ok(());
        }

        plotters_backend::rasterizer::draw_line(self, from, to, style)
    }

    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: (i32, i32),
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (width, height) = self.estimate_text_size(text, style)?;
        let (width, height) = (width as i32, height as i32);
        let dx = match style.anchor().h_pos {
            HPos::Left => 0,
            HPos::Right => -width,
            HPos::Center => -width / 2,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0,
            VPos::Center => -height / 2,
            VPos::Bottom => -height,
        };
        let w = self.width as i32;
        let offset = (pos.1 + dy).max(0) * w + (pos.0 + dx).max(0);
        for (idx, chr) in (offset..).zip(text.chars()) {
            self.state[idx as usize].update(PixelState::Text(chr));
        }
        Ok(())
    }

    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        _: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.len() as u32, 1))
    }
}

// FIXME Is this portable?
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub(crate) struct TerminalMultiPlotter {
    tx: Sender,
    children_count: usize,
    _thread: std::thread::JoinHandle<()>,
}

struct Worker {
    opt: PlotterOpt,
    drawing_area: DrawingArea<TextDrawingBackend, plotters::coord::Shift>,
    histories: Vec<History>,
}

impl MultiPlotter for TerminalMultiPlotter {
    fn new(opt: PlotterOpt) -> Self
    where
        Self: Sized,
    {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let thread = std::thread::spawn(move || loop {
            let backend = TextDrawingBackend {
                state: vec![PixelState::Empty; 5000],
                width: opt.width,
            };

            let drawing_area = backend.into_drawing_area();

            let mut worker = Worker {
                opt,
                drawing_area,
                histories: Vec::default(),
            };

            while let Some(hr) = rx.blocking_recv() {
                worker.update_history(hr);
            }
        });

        TerminalMultiPlotter {
            tx,
            children_count: 0,
            _thread: thread,
        }
    }

    fn spawn(&mut self) -> Box<dyn Plotter + Send> {
        let tx = self.tx.clone();
        self.children_count += 1;
        let history_id = self.children_count;
        let plotter = TerminalPlotter::new(tx, history_id);

        Box::new(plotter)
    }
}

impl Worker {
    fn draw_chart(&mut self) -> Result<(), Box<dyn Error>> {
        let drawing_area = &mut self.drawing_area;

        clear_screen();

        let width = self.opt.width;
        let range = self.opt.range;

        let x_range = (-(width as f64))..0f64;
        let y_range = range.min..range.max;
        let y_label_size = (5i32).percent_width();
        let x_label_size = (10i32).percent_height();

        let mut chart = ChartBuilder::on(drawing_area)
            .margin(1)
            .set_label_area_size(LabelAreaPosition::Left, y_label_size)
            .set_label_area_size(LabelAreaPosition::Bottom, x_label_size)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        for history in &self.histories {
            let history = history.0.clone();
            chart.draw_series(LineSeries::new(history.into_iter(), &RED))?;
        }

        drawing_area.present()?;

        Ok(())
    }

    fn update_history(&mut self, hr: HistoryRecord) {
        let HistoryRecord { history_id, record } = hr;

        if history_id >= self.histories.len() {
            self.histories.push(History::default());
        }

        if let Some(history) = self.histories.get_mut(history_id) {
            let history = &mut history.0;

            history.push_back(record);

            for (x, _) in history.iter_mut() {
                *x = *x - 1.0;
            }

            if history.len() > self.opt.width {
                history.pop_front();
            }

            self.draw_chart().unwrap();
        }
    }
}
