use crossbeam::channel::{Receiver, Sender};
use eframe::epaint::Color32;
use flexi_logger::{writers::LogWriter, Level};

pub struct Log {
    level: Level,
    target: String,
    msg: String,
}

impl Log {
    pub fn level(&self) -> Level {
        self.level
    }

    pub fn target(&self) -> &String {
        &self.target
    }

    pub fn msg(&self) -> &String {
        &self.msg
    }

    pub fn level_color(&self) -> Color32 {
        match self.level {
            Level::Error => Color32::RED,
            Level::Warn => Color32::YELLOW,
            Level::Info => Color32::BLUE,
            Level::Debug => Color32::GREEN,
            Level::Trace => Color32::KHAKI,
        }
    }
}

pub struct ChannelLog {
    sender: Sender<Log>,
}

impl ChannelLog {
    pub fn new() -> (Self, Receiver<Log>) {
        let (s, r) = crossbeam::channel::unbounded::<Log>();
        (Self { sender: s }, r)
    }
}

impl LogWriter for ChannelLog {
    fn write(
        &self,
        _: &mut flexi_logger::DeferredNow,
        record: &flexi_logger::Record,
    ) -> std::io::Result<()> {
        // let log_string = format!("[{} {}] {}", record.level(), record.target(), record.args());
        let log = Log {
            level: record.level(),
            target: record.target().to_string(),
            msg: record.args().to_string(),
        };

        self.sender
            .send(log)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
