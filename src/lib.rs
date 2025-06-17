use chrono::Local;
use csv::Writer;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

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
    // #[command(subcommand)]
    // pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// clears the current data csv (saves a backup)
    Clean {
        /// clears all data. maybe call this `all`?
        #[arg(short, long)]
        force: bool,
    },
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
        Some(Local::now().date_naive().format(DATE_FMT_IN).to_string()),
        None,
        Some(parse_date),
    );
    let time = user_input(
        format!(
            "time (default: {}): ",
            Local::now().naive_local().format(TIME_FMT_IN).to_string()
        ),
        Some(Local::now().naive_local().format(TIME_FMT_IN).to_string()),
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
    let time = chrono::NaiveDate::parse_from_str(s, TIME_FMT_IN).ok()?;
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
            return s.to_string();
        }

        // if nothing worked, repeat
        print!("{}", repeat_prompt);
    }
}

// fn get_date() -> String {
//     let mut ans = String::new();
//     std::io::stdin().read_line(&mut ans);
//     ans.trim();
//     default_date: str = datetime.now().strftime("%Y%m%d")
//     let initial_input = input(f"date (default: {datetime.now().strftime('%Y%m%d')}): ")

//     if initial_input == "":
//         return default_date

//     return "".join(filter(str.isdigit, initial_input))
// }

// def is_date(s: str) -> bool:
//     # cneck that date is a string of digits and is length YYYYMMDD == 8
//     if any([not c.isdigit() for c in s]):
//         return False

//     if not len(s) == 8:
//         return False

//     # we assume any year is possible. check bounds of month and day
//     return 0 < int(s[4:6]) <= 12 and 0 < int(s[6:8]) <= 31

// def get_time() -> str:
//     default_time: str = datetime.now().strftime("%H%M")
//     initial_input: str = input(f"time (default: {datetime.now().strftime('%H%M')}): ")

//     if initial_input == "":
//         return default_time

//     return "".join(filter(str.isdigit, initial_input))

// def is_time(s: str) -> bool:
//     # cneck that time is a string of digits and is length HHMM == 4
//     if any([not c.isdigit() for c in s]):
//         return False

//     if not len(s) == 4:
//         return False

//     # check bounds of hours and minutes
//     return int(s[0:2]) <= 24 and int(s[2:4]) <= 59

// def get_mood() -> str:
//     initial_input: str = input(f"mood (1-5): ")

//     return "".join(filter(str.isdigit, initial_input))

// def is_mood(s: str) -> bool:
//     # cneck that mood is a string of digits and is length 1
//     if any([not c.isdigit() for c in s]):
//         return False

//     if not len(s) == 1:
//         return False

//     # check bounds of mood
//     return 1 <= int(s) <= 5

// def get_descr() -> str:
//     initial_input: str = input(f"description (default: none): ")

//     return initial_input

// def is_descr(s: str) -> bool:
//     return ("\t" not in s) and ("\n" not in s)

// def validate(f, v, prompt="reenter ") -> str:
//     """
//     function f : () -> 'a
//     validator v : 'a -> bool that returns whether f() is valid

//     repeatedly runs f(), returning when v(f()) == True.
//     """
//     ans = f()
//     while not v(ans):
//         print(prompt, end="")
//         ans = f()

// date = validate(get_date, is_date)
// time = validate(get_time, is_time)
// date = validate(get_moode, is_mood)
// date = validate(get_descr, is_descr, prompt="reenter; no tabs or newlines in ")
