#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

#[cfg(windows)]
const DAEMON_BYTES: &[u8] = include_bytes!("../../../target/release/tomodachi-daemon.exe");
#[cfg(windows)]
const CLIENT_BYTES: &[u8] = include_bytes!("../../../target/release/tomodachi-client.exe");

#[cfg(not(windows))]
const DAEMON_BYTES: &[u8] = &[];
#[cfg(not(windows))]
const CLIENT_BYTES: &[u8] = &[];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Tomodachi Installer by 3otxe"),
        ..Default::default()
    };
    eframe::run_native(
        "Tomodachi Installer by 3otxe",
        options,
        Box::new(|_cc| Box::<InstallerApp>::default()),
    )
}

struct InstallerApp {
    install_startup: bool,
    install_ps: bool,
    install_zsh: bool,
    install_cmd: bool,
    
    installing: bool,
    status_text: String,
    done: bool,
}

impl Default for InstallerApp {
    fn default() -> Self {
        Self {
            install_startup: true,
            install_ps: true,
            install_zsh: true,
            install_cmd: true,
            installing: false,
            status_text: String::new(),
            done: false,
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🐾 Tomodachi Installer by 3otxe");
            ui.add_space(10.0);
            
            ui.label("Tomodachi will be installed to %LOCALAPPDATA%\\Tomodachi.");
            ui.label("Select the hooks you want to install:");
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut self.install_startup, "Windows Startup (Background Daemon)");
            ui.checkbox(&mut self.install_ps, "PowerShell Hooks");
            ui.checkbox(&mut self.install_zsh, "Zsh Hooks");
            ui.checkbox(&mut self.install_cmd, "Cmd (Clink) Hooks");
            
            ui.add_space(20.0);
            
            if self.installing {
                ui.spinner();
                ui.label(&self.status_text);
            } else if self.done {
                ui.label(egui::RichText::new("✅ Installation Complete!").color(egui::Color32::GREEN));
                ui.label("Tomodachi is now running in your system tray.");
                ui.label("Please restart your terminal to activate the shell hooks.");
                
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else {
                if ui.button("Install").clicked() {
                    self.installing = true;
                    self.status_text = "Extracting files...".to_string();
                    
                    if let Some(localappdata) = std::env::var_os("LOCALAPPDATA") {
                        let install_dir = PathBuf::from(localappdata).join("Tomodachi");
                        let _ = fs::create_dir_all(&install_dir);
                        
                        let daemon_path = install_dir.join("tomodachi-daemon.exe");
                        let client_path = install_dir.join("tomodachi-client.exe");
                        
                        let mut success = true;
                        if fs::write(&daemon_path, DAEMON_BYTES).is_err() {
                            success = false;
                        }
                        if fs::write(&client_path, CLIENT_BYTES).is_err() {
                            success = false;
                        }
                        
                        if success {
                            self.status_text = "Installing hooks...".to_string();
                            
                            let mut cmd = Command::new(&client_path);
                            cmd.arg("install");
                            if self.install_startup { cmd.arg("--startup"); }
                            if self.install_ps { cmd.arg("--ps"); }
                            if self.install_zsh { cmd.arg("--zsh"); }
                            if self.install_cmd { cmd.arg("--cmd"); }
                            
                            let _ = cmd.status();
                            
                            self.status_text = "Starting daemon...".to_string();
                            
                            #[cfg(windows)]
                            {
                                use std::os::windows::process::CommandExt;
                                const CREATE_NO_WINDOW: u32 = 0x08000000;
                                let _ = Command::new(&daemon_path).creation_flags(CREATE_NO_WINDOW).spawn();
                                
                                // Create Start Menu shortcut
                                let script = format!(
                                    "$wshell = New-Object -ComObject WScript.Shell; $shortcut = $wshell.CreateShortcut(\"$env:APPDATA\\Microsoft\\Windows\\Start Menu\\Programs\\Tomodachi by 3otxe.lnk\"); $shortcut.TargetPath = \"{}\"; $shortcut.Save()",
                                    daemon_path.display()
                                );
                                let _ = Command::new("powershell")
                                    .args(&["-NoProfile", "-Command", &script])
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .spawn();
                            }
                            #[cfg(not(windows))]
                            {
                                let _ = Command::new(&daemon_path).spawn();
                            }
                            
                            self.installing = false;
                            self.done = true;
                        } else {
                            self.installing = false;
                            self.status_text = "Failed to extract files! Make sure it's not already running.".to_string();
                        }
                    } else {
                        self.installing = false;
                        self.status_text = "Failed to find AppData!".to_string();
                    }
                }
            }
        });
    }
}
