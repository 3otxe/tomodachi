

use directories::ProjectDirs;
use rusqlite::{params, Connection};
use tomodachi_shared::{CreatureState, Mood, APP_NAME, APP_ORG, APP_QUALIFIER};
use tracing::info;

pub struct StateStore {
    conn: Connection,
}

impl StateStore {
    
    pub fn open() -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))?;

        let data_dir = dirs.data_local_dir();
        std::fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("state.db");
        info!(path = %db_path.display(), "opening state database");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS creature (
                id INTEGER PRIMARY KEY DEFAULT 1,
                mood TEXT NOT NULL DEFAULT 'Idle',
                xp INTEGER NOT NULL DEFAULT 0,
                hp INTEGER NOT NULL DEFAULT 100,
                level INTEGER NOT NULL DEFAULT 1,
                streak INTEGER NOT NULL DEFAULT 0,
                last_active TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS command_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                command TEXT,
                exit_code INTEGER,
                cwd TEXT,
                shell TEXT
            );

            -- Ensure there's always a row in creature
            INSERT OR IGNORE INTO creature (id, mood, xp, hp, level, streak, last_active)
            VALUES (1, 'Idle', 0, 100, 1, 0, datetime('now'));
            ",
        )?;
        Ok(())
    }

    pub fn load_creature(&self) -> anyhow::Result<CreatureState> {
        let mut stmt = self.conn.prepare(
            "SELECT mood, xp, hp, level, streak, last_active FROM creature WHERE id = 1",
        )?;

        let state = stmt.query_row([], |row| {
            let mood_str: String = row.get(0)?;
            let mood = match mood_str.as_str() {
                "Happy" => Mood::Happy,
                "Smug" => Mood::Smug,
                "Sad" => Mood::Sad,
                "Nervous" => Mood::Nervous,
                "Furious" => Mood::Furious,
                "Sleeping" => Mood::Sleeping,
                _ => Mood::Idle,
            };

            let last_active_str: String = row.get(5)?;
            let last_active = chrono::DateTime::parse_from_rfc3339(&last_active_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(CreatureState {
                mood,
                xp: row.get(1)?,
                hp: row.get(2)?,
                level: row.get(3)?,
                streak: row.get(4)?,
                last_active,
            })
        })?;

        Ok(state)
    }

    pub fn save_creature(&self, state: &CreatureState) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE creature SET mood = ?1, xp = ?2, hp = ?3, level = ?4, streak = ?5, last_active = ?6 WHERE id = 1",
            params![
                state.mood.to_string(),
                state.xp,
                state.hp,
                state.level,
                state.streak,
                state.last_active.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn log_command(
        &self,
        command: Option<&str>,
        exit_code: Option<i32>,
        cwd: Option<&str>,
        shell: Option<&str>,
    ) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO command_log (command, exit_code, cwd, shell) VALUES (?1, ?2, ?3, ?4)",
            params![command, exit_code, cwd, shell],
        )?;
        Ok(())
    }

    pub fn get_recent_commands(&self, n: usize) -> anyhow::Result<Vec<CommandLogEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, command, exit_code, cwd, shell FROM command_log ORDER BY id DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![n as i64], |row| {
                Ok(CommandLogEntry {
                    timestamp: row.get(0)?,
                    command: row.get(1)?,
                    exit_code: row.get(2)?,
                    cwd: row.get(3)?,
                    shell: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn generate_roast(&self) -> anyhow::Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT command, exit_code FROM command_log WHERE timestamp >= datetime('now', '-7 days') ORDER BY id DESC LIMIT 1000"
        )?;

        let mut fail_count = 0;
        let mut total_count = 0;
        let mut rm_rf_count = 0;
        let mut pipe_bash_count = 0;
        let mut git_commits = 0;
        let mut typos = 0;

        let rows = stmt.query_map([], |row| {
            let cmd: Option<String> = row.get(0)?;
            let exit: Option<i32> = row.get(1)?;
            Ok((cmd, exit))
        })?;

        for row in rows {
            if let Ok((cmd, exit)) = row {
                total_count += 1;
                if let Some(code) = exit {
                    if code != 0 {
                        fail_count += 1;
                        if code == 127 || code == 9009 {
                            typos += 1;
                        }
                    }
                }
                if let Some(c) = cmd {
                    let lower = c.to_lowercase();
                    if lower.contains("rm -rf") || lower.contains("rd /s") || lower.contains("remove-item -recurse -force") {
                        rm_rf_count += 1;
                    }
                    if (lower.contains("curl") || lower.contains("irm")) && (lower.contains("bash") || lower.contains("iex")) {
                        pipe_bash_count += 1;
                    }
                    if lower.contains("git commit") {
                        git_commits += 1;
                    }
                }
            }
        }

        if total_count == 0 {
            return Ok("You literally haven't done anything. Do you even code?".to_string());
        }

        let fail_rate = (fail_count as f32 / total_count as f32) * 100.0;
        let mut roast = format!("🐾 Tomodachi's Weekly Roast 🐾\n\nYou ran {} commands this week.\n", total_count);

        if fail_rate > 30.0 {
            roast.push_str(&format!("You failed {:.1}% of them. Have you considered reading the documentation?\n", fail_rate));
        } else if fail_rate == 0.0 {
            roast.push_str("0% failure rate? Either you're a god, or you only ran 'ls' all week.\n");
        } else {
            roast.push_str(&format!("A {:.1}% failure rate. Perfectly mediocre.\n", fail_rate));
        }

        if typos > 5 {
            roast.push_str(&format!("You had {} typos (command not found). Slow down and use tab completion.\n", typos));
        }

        if rm_rf_count > 0 {
            roast.push_str(&format!("You tried to recursively delete things {} times. Living dangerously.\n", rm_rf_count));
        }

        if pipe_bash_count > 0 {
            roast.push_str("You piped an internet URL straight into bash/iex. I am calling the police.\n");
        }

        if git_commits < 2 && total_count > 50 {
            roast.push_str("Over 50 commands and barely any commits. What exactly are you doing all day?\n");
        } else if git_commits > 20 {
            roast.push_str("Calm down with the git commits. Squash your history, weirdo.\n");
        }

        Ok(roast)
    }
}

#[derive(Debug)]
pub struct CommandLogEntry {
    pub timestamp: String,
    pub command: Option<String>,
    pub exit_code: Option<i32>,
    pub cwd: Option<String>,
    pub shell: Option<String>,
}
