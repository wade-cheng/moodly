use chrono::Local;
use csv::Writer;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

/// The directory name moodly data is stored under.
const MOODLY_DIR_NAME: &str = "moodly";
/// The file name moodly mood data is stored under.
const DATA_CSV_FNAME: &str = "moodly_data.csv";
/// An environment variable key for manually setting the moodly directory's parent folder.
const DATA_DIR_PATH_ENV_KEY: &str = "MOODLY_DIR";
/// strftime format string for moodly date inputs
const DATE_FMT_IN: &str = "%Y%m%d";
/// strftime format string for moodly time inputs
const TIME_FMT_IN: &str = "%H%M";
/// strftime format string for moodly date outputs
const DATE_FMT_OUT: &str = "%Y-%m-%d";
/// strftime format string for moodly time outputs
const TIME_FMT_OUT: &str = "%H:%M";

use clap::{Parser, Subcommand};

/// A terminal mood-tracking program.
///
/// Run with no commands to track your current mood.
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // /// Deletes current data csv (saves a backup)
    // Clean {
    //     /// Also delete all backups.
    //     #[arg(short, long)]
    //     include_backups: bool,
    // },
    /// Print the most recent few entries to stdout
    Tail {
        /// Number of entries to print
        #[arg(short, default_value_t = 10)]
        n: usize,
    },
    /// Dump the entire mood recording csv file to stdout
    ///
    /// Consider paging the output, e.g. with `moodly dump | less`.
    Dump,
    /// Print the path to the moodly data directory
    Where,
}

impl Cli {
    pub fn run(&self) -> Result<(), std::io::Error> {
        match &self.command {
            // Some(Commands::Clean { include_backups: _ }) => {
            //     todo!()
            // }
            Some(Commands::Tail { n }) => print_tail_buffered(data_dir()?.join(DATA_CSV_FNAME), *n),
            Some(Commands::Dump) => print_head_buffered(data_dir()?.join(DATA_CSV_FNAME), None),
            Some(Commands::Where) => {
                println!("{}", data_dir()?.display().to_string());
                Ok(())
            }
            None => record(),
        }
    }
}

fn print_tail_buffered<P>(path: P, tailsize: usize) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // we seem to need to get file size from seek because we use this number
    // to seek later. using File metadata causes problems with seeking before 0.
    // (maybe metadata records size on disk instead of bytes used. haven't checked.)
    let file_size = reader.seek(SeekFrom::End(0))?;
    if file_size == 0 {
        return Ok(());
    }

    // Read chunks backwards to find the last n lines
    let chunk_size = 0x2000;
    let mut lines_found = 0;
    let mut position = file_size;
    let mut all_content = Vec::new();

    while position > 0 && lines_found < tailsize {
        let read_size = std::cmp::min(chunk_size, position);

        position = reader.seek(SeekFrom::Current(-1 * read_size as i64))?;
        let mut chunk = vec![0; read_size as usize];
        reader.read_exact(&mut chunk)?;

        // Count newlines in this chunk
        let newlines_in_chunk = chunk.iter().filter(|&&b| b == b'\n').count();

        // Prepend chunk to our content
        // TODO: this is stupid and should be changed.
        chunk.extend(all_content);
        all_content = chunk;

        lines_found += newlines_in_chunk;
    }

    // Convert to string and split into lines
    let content = String::from_utf8_lossy(&all_content);
    let all_lines: Vec<&str> = content.lines().collect();

    // Take the last n lines
    let start_idx = if all_lines.len() > tailsize {
        all_lines.len() - tailsize
    } else {
        0
    };

    for line in &all_lines[start_idx..] {
        println!("{}", line);
    }

    Ok(())
}

fn print_head_buffered<P>(path: P, headsize: Option<usize>) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for (i, line) in BufReader::new(file).lines().enumerate() {
        if let Some(l) = headsize {
            if l == i {
                break;
            }
        }
        writeln!(writer, "{}", line?)?;
    }

    writer.flush()?;
    Ok(())
}

/// Try to get the data directory, creating it if it doesn't exist
///
/// Looks for the value of an environment variable matching the key
/// [`DATA_DIR_PATH_ENV_KEY`], then [`dirs::data_local_dir`]. Otherwise, `Err`.
///
/// The path should look something like `/home/alice/.local/share/moodly/`.
pub fn data_dir() -> Result<PathBuf, std::io::Error> {
    for (key, value) in std::env::vars_os() {
        if key == DATA_DIR_PATH_ENV_KEY {
            let dir = PathBuf::from(value).join(MOODLY_DIR_NAME);
            std::fs::create_dir_all(&dir)?;
            return Ok(dir);
        }
    }

    if let Some(dir) = dirs::data_local_dir() {
        let dir = dir.join(MOODLY_DIR_NAME);
        std::fs::create_dir_all(&dir)?;
        return Ok(dir);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "data file could not be found.",
    ))
}

/// Starts an interactive session to record the user's current mood.
pub fn record() -> Result<(), std::io::Error> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir().map(|p| p.join(DATA_CSV_FNAME))?)?;

    let mut wtr = Writer::from_writer(file);
    // date, time, mood, description
    let date = user_input(
        format!(
            "date (default: {}): ",
            Local::now().date_naive().format(DATE_FMT_IN).to_string()
        ),
        Some(Local::now().date_naive().format(DATE_FMT_OUT).to_string()),
        None,
        Some(parse_date),
    );
    let time = user_input(
        format!(
            "time (default: {}): ",
            Local::now().naive_local().format(TIME_FMT_IN).to_string()
        ),
        Some(Local::now().naive_local().format(TIME_FMT_OUT).to_string()),
        None,
        Some(parse_time),
    );
    let mood = user_input("mood (1-5): ".to_string(), None, None, Some(parse_mood));
    let descr = user_input(
        "description (default: empty): ".to_string(),
        Some("".to_string()),
        None,
        None,
    );
    wtr.write_record(&[date, time, mood, descr])?;

    Ok(())
}

fn parse_date(s: &str) -> Option<String> {
    let date = chrono::NaiveDate::parse_from_str(s, DATE_FMT_IN).ok()?;
    Some(date.format(DATE_FMT_OUT).to_string())
}

fn parse_time(s: &str) -> Option<String> {
    let time = chrono::NaiveDateTime::parse_from_str(s, TIME_FMT_IN).ok()?;
    Some(time.format(TIME_FMT_OUT).to_string())
}

fn parse_mood(s: &str) -> Option<String> {
    if ["1", "2", "3", "4", "5"].contains(&s) {
        return Some(s.to_string());
    } else {
        None
    }
}

pub fn user_input(
    prompt: String,
    default_response: Option<String>,
    repeat_prompt: Option<String>,
    parse: Option<fn(&str) -> Option<String>>,
) -> String {
    let repeat_prompt = repeat_prompt.unwrap_or("reenter ".to_string());
    let parse = parse.unwrap_or(|s| Some(s.to_string()));

    let mut ans = String::new();
    loop {
        // print prompt
        print!("{prompt}");
        std::io::stdout().flush().unwrap();

        // get new user input, trimmed
        ans.clear();
        std::io::stdin().read_line(&mut ans).unwrap();
        let ans = ans.trim();

        // if user input was whitespace and we were provided a default, return it.
        // waiting on if let chains here...
        if ans.is_empty() {
            if let Some(s) = default_response {
                return s;
            }
        }

        // if we can parse the input, return the parsed input.
        if let Some(s) = parse(&ans) {
            return s;
        }

        // if nothing worked, repeat
        print!("{}", repeat_prompt);
    }
}
