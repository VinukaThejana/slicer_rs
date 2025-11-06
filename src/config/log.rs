use envmode::EnvMode;
use fern::colors::{Color, ColoredLevelConfig};
use std::{env, time::SystemTime};

pub fn setup() {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| EnvMode::Dev.into());

    if EnvMode::is_dev(&environment) {
        let colors_line = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::White)
            .debug(Color::White)
            .trace(Color::BrightBlack);
        let colors_level = colors_line.info(Color::Green);

        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}[{date} {level} {target} {color_line}] {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                    ),
                    date = humantime::format_rfc3339_seconds(SystemTime::now()),
                    target = record.target(),
                    level = colors_level.color(record.level()),
                    message = message,
                ))
            })
            .level(log::LevelFilter::Trace)
            .chain(std::io::stderr())
            .apply()
            .unwrap();
    } else {
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message,
                ))
            })
            .level(log::LevelFilter::Debug)
            .chain(std::io::stderr())
            .apply()
            .unwrap();
    }
}
