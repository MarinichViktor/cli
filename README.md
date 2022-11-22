# Cli applications manager
Utility application for running multiple terminal commands inside the single terminal window. The utility allows you to track each terminal process's progress and navigate the numerous concurrently running processes.
Todo:
- [x] Create a layout using the tui-rs crate
- [x] Provide a sidebar with the processes list and the running status
- [x] Provide real-time process output in the console window
- [x] Add shortcut keys
- [x] Add a basic console scroll 
- [ ] Optimize console output rendering logic on each tick
- [ ] Provide more advanced scroll functionality
- [ ] Introduce caching to avoid the whole output rebuild
- [ ] Add a signal logic handler on the terminal close
