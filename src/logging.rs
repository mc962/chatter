use std::path::PathBuf;
use std::{env, fs};

pub fn setup_logger(environment: &str, config_log_path: &str) -> Result<(), fern::InitError> {
    let log_path: PathBuf = if config_log_path.is_empty() {
        [env::current_dir().unwrap(), PathBuf::from("tmp")]
            .iter()
            .collect()
    } else {
        PathBuf::from(config_log_path)
    };

    fs::create_dir_all(log_path).unwrap();

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!(
            "tmp/{}.log",
            environment.to_lowercase()
        ))?)
        .apply()?;
    Ok(())
}
