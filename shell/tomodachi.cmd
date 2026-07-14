@echo off
REM tomodachi — cmd.exe integration
REM Run this script to set up doskey macros for the current session.
REM Add to your AutoRun registry key for persistence:
REM   HKCU\Software\Microsoft\Command Processor\AutoRun
REM
REM Requires: tomodachi-client.exe in PATH

REM ── Veto mode ──────────────────────────────────────────────────────────
REM Shadow dangerous commands with doskey macros.
REM The client checks with the daemon before executing.

doskey rd=tomodachi-client.exe veto rd $*
doskey rmdir=tomodachi-client.exe veto rmdir $*
doskey del=tomodachi-client.exe veto del $*
doskey format=tomodachi-client.exe veto format $*

REM ── Note ────────────────────────────────────────────────────────────────
REM cmd.exe has no precmd/preexec hooks natively.
REM The creature will still react via filesystem watching (commits, etc.)
REM but won't see individual commands unless you install Clink:
REM   winget install clink
REM Then place the tomodachi Clink script in your Clink scripts directory.

echo [tomodachi] cmd.exe veto macros loaded. Use --yolo to bypass.
