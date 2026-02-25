// ICS module - downloading and parsing ICS files

mod parser;

pub use parser::{fetch_ics, parse_ics, IcsError};
