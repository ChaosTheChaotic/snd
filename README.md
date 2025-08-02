# Snd

Snd is a tool built in Rust and C that allows you to send files over the local network via UDP with varying amounts of reliability and speed. It aims to be fast, configurable and reliable. It is primarily a CLI tool.
This is only supported by me as a linux only tool due to the fact that I only use linux and other major operating systems do not have such a big focus on the terminal.

# Dependencies
ncurses - for the TUI in the snd mode to select a hostname (This is needed for running too because I am not linking static ncurses bro. Most distros have this installed by default (although you might not have the header files) so you shouldnt have to worry about it, if you get an error trying installing ncurses first to see if it runs)
