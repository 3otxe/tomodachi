

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    
    Notify {
        
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        pending_cmd: Option<String>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        shell: Option<String>,
    },

    Veto {
        
        command: String,
        
        args: Vec<String>,
    },

    Status,

    Ping,

    Roast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    
    Ok,

    VetoResult {
        allowed: bool,
        reason: String,
    },

    State {
        #[serde(flatten)]
        creature: CreatureState,
    },

    Error {
        message: String,
    },

    RoastText {
        text: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Idle,
    Happy,
    Smug,
    Sad,
    Nervous,
    Furious,
    Sleeping,
}

impl Default for Mood {
    fn default() -> Self {
        Mood::Idle
    }
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mood::Idle => write!(f, "Idle"),
            Mood::Happy => write!(f, "Happy"),
            Mood::Smug => write!(f, "Smug"),
            Mood::Sad => write!(f, "Sad"),
            Mood::Nervous => write!(f, "Nervous"),
            Mood::Furious => write!(f, "Furious"),
            Mood::Sleeping => write!(f, "Sleeping"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureState {
    pub mood: Mood,
    pub xp: u32,
    pub hp: u32,
    pub level: u32,
    pub streak: u32,
    pub last_active: DateTime<Utc>,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self {
            mood: Mood::Idle,
            xp: 0,
            hp: 100,
            level: 1,
            streak: 0,
            last_active: Utc::now(),
        }
    }
}

impl CreatureState {
    
    pub fn recalculate_level(&mut self) {
        self.level = 1 + (self.xp / 100);
    }
}

#[derive(Debug, Clone)]
pub enum CreatureEvent {
    
    CommandFinished {
        exit_code: i32,
        command: Option<String>,
        cwd: Option<String>,
    },
    
    CommandPending {
        command: String,
    },
    
    GitCommit,
    
    UnstagedChanges,
    
    Inactivity,
    
    MoodDecayTick,
}

pub const PIPE_NAME: &str = r"\\.\pipe\tomodachi";

pub const APP_NAME: &str = "tomodachi";
pub const APP_QUALIFIER: &str = "dev";
pub const APP_ORG: &str = "tomodachi";
