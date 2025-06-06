use crate::{clock::Timestamp, pin_state::WireState};
use arrayvec::ArrayString;
use flate2::{write::GzEncoder, Compression};
use kanal::{Receiver, Sender};
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    fs::File,
    io::{BufWriter, Write},
    thread::JoinHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VcdEvent {
    pub t: Timestamp,
    pub signal_id: i32,
    pub new_value: ArrayString<32>,
}

pub enum VcdSignal {
    Scope {
        name: String,
        children: Vec<VcdSignal>,
    },
    Signal {
        name: String,
        id: i32,
        size: i32,
    },
}

pub trait VcdSender {
    fn register_vcd(&mut self, sender: Sender<VcdEvent>, start_id: i32) -> (Vec<VcdSignal>, i32);
    fn vcd_sender(&self) -> Option<&Sender<VcdEvent>>;

    fn send_vcd(&self, t: Timestamp, signal_id: i32, value: &[WireState]) {
        if let Some(sender) = self.vcd_sender() {
            let mut str = ArrayString::new();
            for v in value {
                let c = match v {
                    WireState::Low | WireState::WeakLow => '0',
                    WireState::High | WireState::WeakHigh => '1',
                    WireState::Z => 'Z',
                    WireState::Error => 'X',
                };
                str.push(c);
            }
            let _ = sender.send(VcdEvent {
                t,
                signal_id,
                new_value: str,
            });
        }
    }
}

enum VcdWriter {
    None,
    Raw(BufWriter<File>),
    Gz(GzEncoder<BufWriter<File>>),
}

impl Write for VcdWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            VcdWriter::None => Ok(buf.len()),
            VcdWriter::Raw(buf_writer) => buf_writer.write(buf),
            VcdWriter::Gz(gz_encoder) => gz_encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            VcdWriter::None => Ok(()),
            VcdWriter::Raw(buf_writer) => buf_writer.flush(),
            VcdWriter::Gz(gz_encoder) => gz_encoder.flush(),
        }
    }
}

pub struct VcdReceiver {
    pub sender: Sender<VcdEvent>,
    receiver: Receiver<VcdEvent>,
    signals: Vec<VcdSignal>,
    signal_count: i32,
    queue: PriorityQueue<VcdEvent, Reverse<Timestamp>>,
    writer: VcdWriter,
    ns_per_step: i64,
}

pub struct DeployedVcdReceiver {
    sender: Sender<VcdEvent>,
    thread: Option<JoinHandle<()>>,
    disabled: bool,
}

impl VcdReceiver {
    pub fn new_dummy() -> Self {
        let (sender, receiver) = kanal::bounded(128);

        Self {
            sender,
            receiver,
            signals: Vec::new(),
            signal_count: 0,
            queue: PriorityQueue::new(),
            writer: VcdWriter::None,
            ns_per_step: 1,
        }
    }
    pub fn new(freq: i64, compressed: bool) -> Self {
        let (sender, receiver) = kanal::bounded(128);
        let filename = if compressed { "out.vcd.gz" } else { "out.vcd" };
        let file = File::create(filename).expect("Couldn't create file out.vcd");
        let buf_writer = BufWriter::new(file);
        let writer = if compressed {
            VcdWriter::Gz(GzEncoder::new(buf_writer, Compression::default()))
        } else {
            VcdWriter::Raw(buf_writer)
        };
        Self {
            sender,
            receiver,
            signals: Vec::new(),
            signal_count: 0,
            queue: PriorityQueue::new(),
            writer,
            ns_per_step: 1_000_000_000 / freq,
        }
    }

    pub fn register<S>(&mut self, vcd_sender: &mut S, name: &str)
    where
        S: VcdSender + ?Sized,
    {
        let (new_signals, count) = vcd_sender.register_vcd(self.sender.clone(), self.signal_count);
        self.signals.push(VcdSignal::Scope {
            name: name.to_string(),
            children: new_signals,
        });
        self.signal_count += count;
    }

    fn write_id(w: &mut impl Write, signal_id: i32) {
        if signal_id == 0 {
            write!(w, "!").unwrap();
            return;
        }

        let mut x = signal_id;
        while x > 0 {
            let c = (x % 92) as u8 + '!' as u8;
            write!(w, "{}", c as char).unwrap();
            x /= 92;
        }
    }

    fn write_signal_header(w: &mut impl Write, s: &VcdSignal) {
        match s {
            VcdSignal::Scope { name, children } => {
                writeln!(w, "$scope module {} $end", name).unwrap();
                for c in children {
                    Self::write_signal_header(w, c);
                }
                writeln!(w, "$upscope $end").unwrap();
            }
            VcdSignal::Signal { name, id, size } => {
                write!(w, "$var wire {} ", size).unwrap();
                Self::write_id(w, *id);
                writeln!(w, " {} $end", name).unwrap();
            }
        }
    }

    pub fn write_header(&mut self) {
        writeln!(&mut self.writer, "$version Amber 1.0\n$end").unwrap();
        writeln!(&mut self.writer, "$timescale 1 ns\n$end").unwrap();
        for s in &self.signals {
            Self::write_signal_header(&mut self.writer, s);
        }
        writeln!(&mut self.writer, "$enddefinitions $end").unwrap();
    }

    pub fn write_up_to(&mut self, max_size: usize) {
        let mut current_t = 0;
        while self.queue.len() > max_size {
            let (e, t) = self.queue.peek().unwrap();

            if t.0 > current_t {
                current_t = t.0;
                writeln!(&mut self.writer, "#{}", t.0 * self.ns_per_step).unwrap();
            }
            if e.new_value.len() > 1 {
                write!(&mut self.writer, "b{} ", e.new_value).unwrap();
            } else {
                write!(&mut self.writer, "{}", e.new_value).unwrap();
            }
            Self::write_id(&mut self.writer, e.signal_id);
            writeln!(&mut self.writer).unwrap();
            self.queue.pop();
        }
    }

    pub fn run(&mut self) {
        if let VcdWriter::None = self.writer {
            return;
        }

        self.write_header();
        while let Ok(e) = self.receiver.recv() {
            if e.signal_id == -1 {
                break;
            } else {
                self.queue.push(e, Reverse(e.t));
                if self.queue.len() > 32 * 1024 {
                    self.write_up_to(24 * 1024);
                }
            }
        }
        self.write_up_to(0);
    }

    pub fn deploy(mut self) -> DeployedVcdReceiver {
        let disabled = if let VcdWriter::None = self.writer {
            true
        } else {
            false
        };

        DeployedVcdReceiver {
            sender: self.sender.clone(),
            thread: Some(std::thread::spawn(move || self.run())),
            disabled,
        }
    }
}

impl Drop for DeployedVcdReceiver {
    fn drop(&mut self) {
        if self.disabled {
            self.thread.take().unwrap().join().unwrap();
        } else {
            self.sender
                .send(VcdEvent {
                    t: 0,
                    signal_id: -1,
                    new_value: ArrayString::new(),
                })
                .unwrap();
            self.thread.take().unwrap().join().unwrap();
            println!("VCD successfully written");
        }
    }
}
