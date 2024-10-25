use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    General,
    Beeline,
    Mgts,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub mode: Mode,
}