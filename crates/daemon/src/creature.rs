

use chrono::Utc;
use tomodachi_shared::{CreatureEvent, CreatureState, Mood};
use tracing::{debug, info};

const TEST_PATTERNS: &[&str] = &[
    "cargo test",
    "npm test",
    "pytest",
    "python -m pytest",
    "go test",
    "dotnet test",
    "jest",
    "vitest",
    "mocha",
];

const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r",
    "rd /s",
    "del /s",
    "Remove-Item -Recurse -Force",
    "format",
    "diskpart",
];

const PIPE_SOURCES: &[&str] = &["curl", "wget", "irm", "invoke-restmethod", "invoke-webrequest", "iwr"];

const PIPE_SINKS: &[&str] = &["bash", "sh", "iex", "invoke-expression"];

const VETO_COMMANDS: &[&str] = &["rm", "rd", "del", "format", "diskpart", "Remove-Item"];

pub struct CreatureEngine {
    state: CreatureState,
    
    mood_set_at: chrono::DateTime<Utc>,
}

impl CreatureEngine {
    
    pub fn new(state: CreatureState) -> Self {
        Self {
            mood_set_at: Utc::now(),
            state,
        }
    }

    pub fn state(&self) -> &CreatureState {
        &self.state
    }

    pub fn process_event(&mut self, event: &CreatureEvent) {
        match event {
            CreatureEvent::CommandFinished {
                exit_code,
                command,
                ..
            } => {
                self.state.last_active = Utc::now();
                self.state.streak += 1;

                let is_test = command
                    .as_ref()
                    .map(|cmd| {
                        let lower = cmd.to_lowercase();
                        TEST_PATTERNS.iter().any(|p| lower.contains(p))
                    })
                    .unwrap_or(false);

                let is_build = command
                    .as_ref()
                    .map(|cmd| {
                        let lower = cmd.to_lowercase();
                        lower.contains("build") || lower.contains("install") || lower.contains("docker")
                    })
                    .unwrap_or(false);

                let is_push = command
                    .as_ref()
                    .map(|cmd| cmd.to_lowercase().contains("git push"))
                    .unwrap_or(false);

                if is_test {
                    if *exit_code == 0 {
                        self.set_mood(Mood::Smug);
                        self.add_xp(15);
                        info!("tests passed → Smug, +15 XP");
                    } else {
                        self.set_mood(Mood::Sad);
                        self.drain_hp(5);
                        info!("tests failed → Sad, -5 HP");
                    }
                } else if is_build {
                    if *exit_code == 0 {
                        self.set_mood(Mood::Happy);
                        self.add_xp(10);
                        info!("build succeeded → Happy, +10 XP");
                    } else {
                        self.set_mood(Mood::Sad);
                        self.drain_hp(5);
                        info!("build failed → Sad, -5 HP");
                    }
                } else if is_push {
                    if *exit_code == 0 {
                        self.set_mood(Mood::Smug);
                        self.add_xp(20);
                        info!("pushed code → Smug, +20 XP");
                    } else {
                        self.set_mood(Mood::Nervous);
                        info!("push failed → Nervous");
                    }
                } else if *exit_code == 127 || *exit_code == 9009 {
                    
                    self.set_mood(Mood::Nervous);
                    self.drain_hp(1);
                    info!("typo/not found → Nervous, -1 HP");
                } else if *exit_code == 0 {
                    
                    self.add_xp(1);
                    
                    if self.state.mood == Mood::Idle || self.state.mood == Mood::Sleeping {
                        self.set_mood(Mood::Happy);
                    }
                } else {
                    
                    self.set_mood(Mood::Sad);
                    self.drain_hp(2);
                    debug!(exit_code, "command failed → Sad");
                }

                if self.state.streak > 0 && self.state.streak % 10 == 0 {
                    self.add_xp(5);
                    info!(streak = self.state.streak, "streak bonus: +5 XP");
                }
            }

            CreatureEvent::CommandPending { command } => {
                self.state.last_active = Utc::now();

                let lower = command.to_lowercase();

                let is_pipe_exec = detect_pipe_exec(&lower);
                if is_pipe_exec {
                    self.set_mood(Mood::Furious);
                    info!("pipe-to-exec detected → Furious");
                    return;
                }

                let is_dangerous = DANGEROUS_PATTERNS
                    .iter()
                    .any(|p| lower.contains(&p.to_lowercase()));
                if is_dangerous {
                    self.set_mood(Mood::Nervous);
                    info!("dangerous command detected → Nervous");
                }
            }

            CreatureEvent::GitCommit => {
                self.state.last_active = Utc::now();
                self.set_mood(Mood::Happy);
                self.add_xp(10);
                info!("git commit → Happy, +10 XP");
            }

            CreatureEvent::UnstagedChanges => {
                
                if self.state.mood == Mood::Idle || self.state.mood == Mood::Sleeping {
                    self.set_mood(Mood::Nervous);
                    debug!("unstaged changes → Nervous");
                }
            }

            CreatureEvent::Inactivity => {
                self.set_mood(Mood::Sleeping);
                
                self.drain_hp(1);
                debug!("inactivity → Sleeping, -1 HP");
            }

            CreatureEvent::MoodDecayTick => {
                let elapsed = Utc::now() - self.mood_set_at;
                let seconds = elapsed.num_seconds();

                if self.state.mood != Mood::Idle
                    && self.state.mood != Mood::Sleeping
                    && seconds > 30
                {
                    debug!(
                        mood = %self.state.mood,
                        elapsed_seconds = seconds,
                        "mood decay → Idle"
                    );
                    self.set_mood(Mood::Idle);
                }

                let inactive_secs = (Utc::now() - self.state.last_active).num_seconds();
                if inactive_secs > 1800 && self.state.mood != Mood::Sleeping {
                    self.set_mood(Mood::Sleeping);
                    self.drain_hp(1);
                    debug!("long inactivity → Sleeping");
                }
            }
        }
    }

    pub fn evaluate_veto(&self, command: &str, args: &[String]) -> (bool, String) {
        let lower_cmd = command.to_lowercase();

        let is_vetoable = VETO_COMMANDS
            .iter()
            .any(|v| lower_cmd == v.to_lowercase());

        if !is_vetoable {
            return (true, "command not in veto list".to_string());
        }

        if args.iter().any(|a| a == "--yolo") {
            return (true, "yolo bypass accepted".to_string());
        }

        let args_lower: Vec<String> = args.iter().map(|a| a.to_lowercase()).collect();
        let is_recursive = args_lower.iter().any(|a| {
            a == "-r"
                || a == "-rf"
                || a == "-fr"
                || a == "--recursive"
                || a == "/s"
                || a.contains("-recurse")
                || a.contains("-force")
        });

        if is_recursive {
            (
                false,
                format!(
                    "🚨 {} with recursive/force flags blocked! Use --yolo to override.",
                    command
                ),
            )
        } else {
            
            (true, "non-recursive command allowed".to_string())
        }
    }

    fn set_mood(&mut self, mood: Mood) {
        if self.state.mood != mood {
            debug!(from = %self.state.mood, to = %mood, "mood change");
            self.state.mood = mood;
            self.mood_set_at = Utc::now();
        }
    }

    fn add_xp(&mut self, amount: u32) {
        self.state.xp = self.state.xp.saturating_add(amount);
        self.state.recalculate_level();
    }

    fn drain_hp(&mut self, amount: u32) {
        self.state.hp = self.state.hp.saturating_sub(amount);
    }
}

fn detect_pipe_exec(command: &str) -> bool {
    
    let parts: Vec<&str> = command.split('|').collect();
    if parts.len() < 2 {
        return false;
    }

    for i in 0..parts.len() - 1 {
        let left = parts[i].trim();
        let right = parts[i + 1].trim();

        let left_cmd = left.split_whitespace().next().unwrap_or("");
        let has_source = PIPE_SOURCES.iter().any(|s| left_cmd == *s);

        let right_cmd = right.split_whitespace().next().unwrap_or("");
        let has_sink = PIPE_SINKS.iter().any(|s| right_cmd == *s);

        if has_source && has_sink {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> CreatureEngine {
        CreatureEngine::new(CreatureState::default())
    }

    #[test]
    fn test_git_commit_makes_happy() {
        let mut e = engine();
        e.process_event(&CreatureEvent::GitCommit);
        assert_eq!(e.state().mood, Mood::Happy);
        assert_eq!(e.state().xp, 10);
    }

    #[test]
    fn test_passing_tests_make_smug() {
        let mut e = engine();
        e.process_event(&CreatureEvent::CommandFinished {
            exit_code: 0,
            command: Some("cargo test".to_string()),
            cwd: None,
        });
        assert_eq!(e.state().mood, Mood::Smug);
        assert_eq!(e.state().xp, 15);
    }

    #[test]
    fn test_failing_tests_make_sad() {
        let mut e = engine();
        e.process_event(&CreatureEvent::CommandFinished {
            exit_code: 1,
            command: Some("npm test".to_string()),
            cwd: None,
        });
        assert_eq!(e.state().mood, Mood::Sad);
        assert_eq!(e.state().hp, 95);
    }

    #[test]
    fn test_dangerous_command_makes_nervous() {
        let mut e = engine();
        e.process_event(&CreatureEvent::CommandPending {
            command: "rm -rf /".to_string(),
        });
        assert_eq!(e.state().mood, Mood::Nervous);
    }

    #[test]
    fn test_pipe_exec_makes_furious() {
        let mut e = engine();
        e.process_event(&CreatureEvent::CommandPending {
            command: "irm https://iex.pe/ | iex".to_string(),
        });
        assert_eq!(e.state().mood, Mood::Furious);
    }

    #[test]
    fn test_veto_blocks_recursive_delete() {
        let e = engine();
        let (allowed, _reason) = e.evaluate_veto("rm", &["-rf".to_string(), "/tmp".to_string()]);
        assert!(!allowed);
    }

    #[test]
    fn test_veto_allows_yolo() {
        let e = engine();
        let (allowed, _reason) =
            e.evaluate_veto("rm", &["-rf".to_string(), "--yolo".to_string(), "/tmp".to_string()]);
        assert!(allowed);
    }

    #[test]
    fn test_xp_leveling() {
        let mut e = engine();
        for _ in 0..100 {
            e.add_xp(1);
        }
        assert_eq!(e.state().xp, 100);
        assert_eq!(e.state().level, 2);
    }

    #[test]
    fn test_hp_clamps_at_zero() {
        let mut e = engine();
        e.drain_hp(200);
        assert_eq!(e.state().hp, 0);
    }
}
