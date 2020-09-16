pub fn init() {
    target_logger::init();
}

#[cfg(not(target_os="android"))]
mod target_logger {

    use log::{debug, info, warn, error};
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Root};

    pub fn init() {

        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("log/output.log")
            .expect("unable to initialize log");

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder()
                       .appender("logfile") .build(LevelFilter::Trace))
                       .expect("unable to initialize log");

        log4rs::init_config(config)
            .expect("unable to initialize log");
    }
}


#[cfg(target_os="android")]
mod target_logger {

    extern crate android_logger;
    use log::Level;
    use android_logger::Config;

    pub fn init_logger() {

        android_logger::init_once(
            Config::default()
                .with_min_level(Level::Trace));
    }
}
