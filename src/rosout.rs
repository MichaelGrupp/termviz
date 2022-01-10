use ringbuf;
use rosrust;
use std::sync::{Arc, RwLock};

pub struct LogBuffer {
    producer: ringbuf::Producer<String>,
    consumer: ringbuf::Consumer<String>,
    is_buffering: bool,
}

impl LogBuffer {
    pub fn new(size: usize, start_buffering: bool) -> LogBuffer {
        let (producer, consumer) = ringbuf::RingBuffer::<String>::new(size).split();
        LogBuffer {
            producer,
            consumer,
            is_buffering: start_buffering,
        }
    }

    pub fn push_logstring(&mut self, logstring: String) {
        if self.producer.push(logstring).is_err() {
            // Ignore buffer overflow.
        }
    }

    pub fn drain_logstrings(&mut self) -> std::vec::Vec<String> {
        let mut logstrings = std::vec::Vec::<String>::new();
        while !self.consumer.is_empty() {
            logstrings.push(self.consumer.pop().unwrap());
        }
        return logstrings;
    }
}

fn format_logstring(level: i8, msg: &str, name: &str, stamp: &rosrust::Time) -> String {
    use colored::{Color, Colorize};
    use rosrust_msg::rosgraph_msgs::Log;

    let format_string = |levelname, color| {
        format!("[{} {} {}] {}", name, levelname, stamp.seconds(), msg).color(color)
    };

    match level {
        Log::DEBUG => format!("{}", format_string("DEBUG", Color::White)),
        Log::INFO => format!("{}", format_string("INFO", Color::White)),
        Log::WARN => format!("{}", format_string("WARN", Color::Yellow)),
        Log::ERROR => format!("{}", format_string("ERROR", Color::Red)),
        Log::FATAL => format!("{}", format_string("FATAL", Color::Red)),
        _ => panic!("invalid rosout log level"),
    }
}

pub struct RosoutListener {
    _subscriber: rosrust::Subscriber,
    pub log_buffer: Arc<RwLock<LogBuffer>>,
}

impl RosoutListener {
    pub fn new(buffer_size: usize, start_buffering: bool) -> RosoutListener {
        let log_buffer = Arc::new(RwLock::new(LogBuffer::new(buffer_size, start_buffering)));
        let cb_log_buffer = log_buffer.clone();

        let _subscriber = rosrust::subscribe(
            "/rosout",
            1,
            move |rosout_msg: rosrust_msg::rosgraph_msgs::Log| {
                if cb_log_buffer.read().unwrap().is_buffering {
                    cb_log_buffer
                        .write()
                        .unwrap()
                        .push_logstring(format_logstring(
                            rosout_msg.level,
                            &rosout_msg.msg,
                            &rosout_msg.name,
                            &rosout_msg.header.stamp,
                        ));
                }
            },
        )
        .unwrap();

        RosoutListener {
            _subscriber,
            log_buffer,
        }
    }

    pub fn toggle_buffering(&mut self) {
        self.log_buffer.write().unwrap().is_buffering =
            !self.log_buffer.read().unwrap().is_buffering;
    }

    pub fn drain_logstring_buffer(&self) -> std::vec::Vec<String> {
        return self.log_buffer.write().unwrap().drain_logstrings();
    }
}
