use regex::Regex;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum PikChannel {
    DownloadStats(DownloadStats),
    InfoMessage(String),
    Status(bool),
}

#[derive(Debug, PartialEq)]
pub struct DownloadStats {
    pub total_percent: f64,
    pub total_downloaded: (f64, String),
    pub total_size: (f64, String),
    pub total_speed: (f64, String),
    pub package_name: String,
    pub package_percent: f64,
    pub package_downloaded: (f64, String),
    pub package_size: (f64, String),
    pub package_speed: (f64, String),
}

pub fn parse_download_stats(input: &str) -> Option<DownloadStats> {
    // Main regex pattern to match the entire string
    let re = Regex::new(r"^Total: (\d+\.\d+)% \(([\d\.]+) ([A-Za-z]+) of ([\d\.]+) ([A-Za-z]+), ([\d\.]+) ([A-Za-z]+)/s\) - ([^:]+): (\d+\.\d+)% \(([\d\.]+) ([A-Za-z]+) of ([\d\.]+) ([A-Za-z]+), ([\d\.]+) ([A-Za-z]+)/s\)$").unwrap();

    let caps = re.captures(input)?;

    Some(DownloadStats {
        total_percent: caps.get(1)?.as_str().parse().ok()?,
        total_downloaded: (
            caps.get(2)?.as_str().parse().ok()?,
            caps.get(3)?.as_str().to_string(),
        ),
        total_size: (
            caps.get(4)?.as_str().parse().ok()?,
            caps.get(5)?.as_str().to_string(),
        ),
        total_speed: (
            caps.get(6)?.as_str().parse().ok()?,
            format!("{}/s", caps.get(7)?.as_str()),
        ),
        package_name: caps.get(8)?.as_str().trim().to_string(),
        package_percent: caps.get(9)?.as_str().parse().ok()?,
        package_downloaded: (
            caps.get(10)?.as_str().parse().ok()?,
            caps.get(11)?.as_str().to_string(),
        ),
        package_size: (
            caps.get(12)?.as_str().parse().ok()?,
            caps.get(13)?.as_str().to_string(),
        ),
        package_speed: (
            caps.get(14)?.as_str().parse().ok()?,
            format!("{}/s", caps.get(15)?.as_str()),
        ),
    })
}

fn process_socket_output(line: String, log_loop_sender: async_channel::Sender<PikChannel>) {
    if line.contains("no packages found to download")
        || line.contains("failed to simulate")
        || line.contains("ERROR:")
        || line.contains("No packages found")
    {
        log_loop_sender
            .send_blocking(PikChannel::InfoMessage(line))
            .expect("Channel needs to be opened.");
        log_loop_sender
            .send_blocking(PikChannel::Status(false))
            .expect("Channel needs to be opened.");
    } else if line.contains("No upgrades needed")
        || line.contains("No packages to upgrade")
        || line.contains("Installation complete")
        || line.contains("Operation completed successfully")
        || line.contains("Connection closed by server")
    {
        log_loop_sender
            .send_blocking(PikChannel::InfoMessage(line))
            .expect("Channel needs to be opened.");
        log_loop_sender
            .send_blocking(PikChannel::Status(true))
            .expect("Channel needs to be opened.");
    } else if line.starts_with("Total: ") {
        log_loop_sender
            .send_blocking(PikChannel::DownloadStats(
                parse_download_stats(&line).expect("invalid pik output"),
            ))
            .expect("Channel needs to be opened.");
    } else {
        log_loop_sender
            .send_blocking(PikChannel::InfoMessage(line))
            .expect("Channel needs to be opened.");
    }
}

pub fn wrap_text(text: &str, max_length: usize) -> String {
    let mut result = String::new();
    let mut current_line_length = 0;

    for word in text.split_whitespace() {
        let word_length = word.chars().count();

        if current_line_length + word_length > max_length {
            // Don't add newline if this is the first word in line
            if current_line_length > 0 {
                result.push('\n');
                current_line_length = 0;
            }
        } else if current_line_length > 0 {
            result.push(' ');
            current_line_length += 1;
        }

        result.push_str(word);
        current_line_length += word_length;
    }

    result
}

pub fn get_current_font() -> String {
    let settings = gtk::Settings::default().unwrap();
    settings.gtk_font_name().unwrap().to_string()
}

pub fn run_addon_command(
    expr: duct::Expression,
    log_loop_sender: async_channel::Sender<PikChannel>,
    log_file_path: &str,
) -> Result<(), std::boxed::Box<dyn std::error::Error + Send + Sync>> {
    if !Path::new(&log_file_path).exists() {
        match std::fs::File::create(&log_file_path) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Warning: {} file couldn't be created", log_file_path);
            }
        };
    }
    let mut log_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&log_file_path)
        .unwrap();
    let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
    let child = expr.stderr_to_stdout().stdout_file(pipe_writer).start()?;
    for line in BufReader::new(pipe_reader).lines() {
        let line = line?;
        let line = if let Some(pos) = line.find("Total") {
            line[pos..].trim_start().to_string()
        } else {
            line
        };
        println!("{}", line);
        if line.len() < 220 {
            process_socket_output(line.to_string(), log_loop_sender.clone());
        }
        if let Err(e) = writeln!(
            log_file,
            "[{}] {}",
            chrono::offset::Local::now().format("%Y/%m/%d_%H:%M"),
            line
        ) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    child.wait()?;

    Ok(())
}
