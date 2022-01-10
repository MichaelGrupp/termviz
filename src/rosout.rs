use ringbuf;
use rosrust;

pub struct LogBuffer {
    producer: ringbuf::Producer<String>,
    consumer: ringbuf::Consumer<String>,
}

impl LogBuffer {
    pub fn new(size: usize) -> LogBuffer {
        let (mut producer, consumer) = ringbuf::RingBuffer::<String>::new(size).split();
        LogBuffer { producer, consumer }
    }
}

pub struct RosoutListener {
    _subscriber: rosrust::Subscriber,
    log_buffer: LogBuffer,
}

// TODO: return string, make part of Rosoutlistener and add filter
fn format_logstring(level: i8, msg: &str, name: &str, stamp: &rosrust::Time) -> String {
    use colored::{Color, Colorize};
    use rosrust_msg::rosgraph_msgs::Log;

    let format_string = |levelname, color| {
        format!("[{} {} {}] {}", name, levelname, stamp.seconds(), msg).color(color)
    };

    match level {
        Log::DEBUG => println!("{}", format_string("DEBUG", Color::White)),
        Log::INFO => println!("{}", format_string("INFO", Color::White)),
        Log::WARN => eprintln!("{}", format_string("WARN", Color::Yellow)),
        Log::ERROR => eprintln!("{}", format_string("ERROR", Color::Red)),
        Log::FATAL => eprintln!("{}", format_string("FATAL", Color::Red)),
        _ => panic!("invalid rosout log level"),
    }
}

fn push_logstring(&log_buffer: &LogBuffer, &logstring: &String) {
    log_buffer.producer.push(logstring);
}

impl RosoutListener {
    pub fn new(&self, buffer_size: usize) -> RosoutListener {
        let log_buffer = LogBuffer::new(buffer_size);
        let log_buffer_handle = &mut log_buffer;

        let _subscriber = rosrust::subscribe(
            "/rosout",
            1,
            move |rosout_msg: rosrust_msg::rosgraph_msgs::Log| {
                push_logstring(
                    log_buffer_handle,
                    &format_logstring(
                        rosout_msg.level,
                        &rosout_msg.msg,
                        &rosout_msg.name,
                        &rosout_msg.header.stamp,
                    ),
                );
            },
        )
        .unwrap();

        RosoutListener {
            _subscriber,
            log_buffer,
        }
    }
}
