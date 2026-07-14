

use std::time::Duration;

use clap::{Parser, Subcommand};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::time::timeout;
use tomodachi_shared::{ClientMessage, DaemonResponse, PIPE_NAME};

#[derive(Parser)]
#[command(name = "tomodachi-client")]
#[command(about = "Talk to the tomodachi daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    
    Notify {
        
        #[arg(long)]
        exit: Option<i32>,

        #[arg(long)]
        cwd: Option<String>,

        #[arg(long)]
        pending: Option<String>,

        #[arg(long)]
        shell: Option<String>,
    },

    Veto {
        
        command: String,

        args: Vec<String>,
    },

    Status,

    Roast,

    Install {
        #[arg(long)]
        startup: bool,
        #[arg(long)]
        ps: bool,
        #[arg(long)]
        zsh: bool,
        #[arg(long)]
        cmd: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    let is_veto = matches!(cli.command, Command::Veto { .. });

    let msg = match cli.command {
        Command::Notify {
            exit,
            cwd,
            pending,
            shell,
        } => ClientMessage::Notify {
            exit_code: exit,
            cwd,
            pending_cmd: pending,
            shell,
        },
        Command::Veto { command, args } => ClientMessage::Veto { command, args },
        Command::Status => ClientMessage::Status,
        Command::Roast => ClientMessage::Roast,
        Command::Install { startup, ps, zsh, cmd } => {
            install_hooks(startup, ps, zsh, cmd);
            return;
        }
    };

    match timeout(Duration::from_millis(500), send_message(&msg)).await {
        Ok(Ok(response)) => handle_response(&msg, response),
        Ok(Err(_)) => {
            
            if is_veto {
                exec_passthrough(&msg);
            }
        }
        Err(_) => {
            
            if is_veto {
                exec_passthrough(&msg);
            }
        }
    }
}

async fn send_message(msg: &ClientMessage) -> anyhow::Result<DaemonResponse> {
    
    let pipe = loop {
        match ClientOptions::new().open(PIPE_NAME) {
            Ok(pipe) => break pipe,
            Err(e) if e.raw_os_error() == Some(windows_sys::Win32::Foundation::ERROR_PIPE_BUSY as i32) => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => return Err(e.into()),
        }
    };

    let (reader, mut writer) = tokio::io::split(pipe);

    let mut json = serde_json::to_string(msg)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;
    writer.flush().await?;

    let mut lines = BufReader::new(reader).lines();
    if let Some(line) = lines.next_line().await? {
        let response: DaemonResponse = serde_json::from_str(&line)?;
        Ok(response)
    } else {
        Err(anyhow::anyhow!("daemon closed connection"))
    }
}

fn handle_response(msg: &ClientMessage, response: DaemonResponse) {
    match response {
        DaemonResponse::Ok => {
            
        }
        DaemonResponse::VetoResult { allowed, reason } => {
            if !allowed {
                eprintln!("{}", reason);
                std::process::exit(1);
            }
            
            exec_passthrough(msg);
        }
        DaemonResponse::State { creature } => {
            println!("🐾 Tomodachi Status");
            println!("-------------------");
            println!("Mood:   {}", creature.mood);
            println!("Level:  {}", creature.level);
            println!("XP:     {}", creature.xp);
            println!("HP:     {}/100", creature.hp);
            println!("Streak: {}", creature.streak);
        }
        DaemonResponse::RoastText { text } => {
            println!("{}", text);
        }
        DaemonResponse::Error { message } => {
            eprintln!("daemon error: {}", message);
        }
    }
}

fn exec_passthrough(msg: &ClientMessage) {
    if let ClientMessage::Veto { command, args } = msg {
        exec_real_command(command, args);
    }
}

fn exec_real_command(command: &str, args: &[String]) {
    use std::process::Command as ProcCommand;

    let status = ProcCommand::new(command)
        .args(args.iter().filter(|a| *a != "--yolo"))
        .status();

    match status {
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("failed to execute {}: {}", command, e);
            std::process::exit(1);
        }
    }
}

fn install_hooks(install_startup: bool, install_ps: bool, install_zsh: bool, install_cmd: bool) {
    use std::path::PathBuf;
    use std::io::Write;

    let exe_path = std::env::current_exe().expect("Failed to get current executable path");
    let daemon_path = exe_path.with_file_name("tomodachi-daemon.exe");
    let client_path = exe_path.to_string_lossy().to_string().replace("\\", "\\\\");
    
    let mut success_count = 0;

    if install_startup {
        if let Some(appdata) = std::env::var_os("APPDATA") {
        let startup_dir = PathBuf::from(appdata)
            .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
        if startup_dir.exists() {
            let vbs_path = startup_dir.join("tomodachi.vbs");
            let vbs_content = format!(
                "Set WshShell = CreateObject(\"WScript.Shell\")\n\
                 WshShell.Run \"\"\"{}\"\"\", 0, False",
                daemon_path.display()
            );
            if std::fs::write(&vbs_path, vbs_content).is_ok() {
                println!("✅ Installed automatic startup script to {}", vbs_path.display());
                success_count += 1;
            }
        }
    }

    if install_ps {
        if let Some(userprofile) = std::env::var_os("USERPROFILE") {
            let ps_profile_dir = PathBuf::from(&userprofile).join("Documents\\PowerShell");
            std::fs::create_dir_all(&ps_profile_dir).ok();
            let ps_profile = ps_profile_dir.join("Microsoft.PowerShell_profile.ps1");
            
            let ps_hook = format!(
                "\n# Tomodachi Hook\n\
                 $prevPrompt = $function:prompt\n\
                 function prompt {{\n\
                     $exitCode = $LASTEXITCODE\n\
                     & \"{}\" notify --exit $exitCode | Out-Null\n\
                     if ($prevPrompt) {{ & $prevPrompt }}\n\
                 }}\n\
                 if (Get-Module -Name PSReadLine -ListAvailable) {{\n\
                     Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {{\n\
                         $line = $null; $cursor = $null\n\
                         [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)\n\
                         if ($line) {{ & \"{}\" notify --pending $line | Out-Null }}\n\
                         [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()\n\
                     }}\n\
                 }}\n",
                client_path, client_path
            );

            let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&ps_profile).unwrap();
            if file.write_all(ps_hook.as_bytes()).is_ok() {
                println!("✅ Appended PowerShell hooks to {}", ps_profile.display());
                success_count += 1;
            }
        }
    }

    if install_zsh {
        let zshrc = PathBuf::from(&home).join(".zshrc");
        let zsh_client_path = exe_path.to_string_lossy().to_string().replace("\\", "/");
        let zsh_hook = format!(
            "\n# Tomodachi Hook\n\
             autoload -Uz add-zsh-hook\n\
             _tomodachi_precmd()  {{ \"{}\" notify --exit $? --cwd \"$PWD\" &! }}\n\
             _tomodachi_preexec() {{ \"{}\" notify --pending \"$1\" &! }}\n\
             add-zsh-hook precmd  _tomodachi_precmd\n\
             add-zsh-hook preexec _tomodachi_preexec\n",
            zsh_client_path, zsh_client_path
        );

        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&zshrc).unwrap();
        if file.write_all(zsh_hook.as_bytes()).is_ok() {
            println!("✅ Appended zsh hooks to {}", zshrc.display());
            success_count += 1;
        }
    }

    if install_cmd {
        if let Some(localappdata) = std::env::var_os("LOCALAPPDATA") {
            let clink_dir = PathBuf::from(&localappdata).join("clink");
            if clink_dir.exists() {
                let clink_lua = clink_dir.join("tomodachi.lua");
                let lua_client_path = client_path.replace("\\", "\\\\");
                let lua_hook = format!(
                    "local function onbeginedit()\n\
                     \tos.execute('\"{}\" notify >nul 2>nul')\n\
                     end\n\
                     local function onendedit(line)\n\
                     \tlocal safe = string.gsub(line, '\"', '\\\"')\n\
                     \tos.execute('\"{}\" notify --pending \"' .. safe .. '\" >nul 2>nul')\n\
                     end\n\
                     if clink.onbeginedit then\n\
                     \tclink.onbeginedit(onbeginedit)\n\
                     \tclink.onendedit(onendedit)\n\
                     end\n",
                    lua_client_path, lua_client_path
                );
                if std::fs::write(&clink_lua, lua_hook).is_ok() {
                    println!("✅ Installed Clink integration to {}", clink_lua.display());
                    success_count += 1;
                }
            }
        }
    }

    if success_count > 0 {
        println!("Install complete! Restart your shells for hooks to take effect.");
    } else {
        println!("Install didn't find any supported shell profiles.");
    }
}
