//use ferris_says::say;
//use std::io::{stdout, BufWriter};
use sysinfo::*;


fn main() {
    // Configuration //

    let user           = true;
    let partition      = true;
    let os             = true;
    let computer_name  = true;
    let kernel_version = true;
    let uptime         = true;
    let resolution     = true;
    let packages       = true;
    let theme          = true;
    let cpu_name       = true;
    let processes      = true;
    let ram            = true;
    let swap           = true;
    let disk_info      = true;
    let battery        = true;
    let locale         = true;
    let mut stdout = stdout();
    let image = vec![
        "                                .., ",
        "                    ....,,:;+ccllll ",
        "      ...,,+:;  cllllllllllllllllll ",
        ",cclllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "                                    ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "llllllllllllll  lllllllllllllllllll ",
        "`'ccllllllllll  lllllllllllllllllll ",
        "       `' \\*::  :ccllllllllllllllll",
        "                       ````''*::cll ",
        "                                 `` "
    ];
    let cursor_position = (image[0].len() + 1) as u16;

    // Initialization //

    use crossterm::style::Stylize;

    // Queue and print the image
    queue!(stdout, style::Print("\n".repeat(image.len())), cursor::MoveUp(image.len() as u16)).map_err(|e| eprintln!("Error: {}", e)).ok();
    stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();

    let initial_position_y = cursor::position().unwrap().1 + 1;
    let final_position_y = cursor::position().unwrap().1 + (image.len() + 1) as u16;

    for line in &image {
        queue!(stdout, style::PrintStyledContent(line.cyan()), style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
    }
    stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();

    // Create System struct containing all available system information
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(sysinfo::CpuRefreshKind::new().with_frequency())
            .with_disks()
            .with_disks_list()
            .with_memory()
            .with_processes(sysinfo::ProcessRefreshKind::everything().with_user())
            .with_users_list()
    );

    // Update all needed information in the `System` struct.
    let refreshkind = RefreshKind::new()
        .with_cpu(sysinfo::CpuRefreshKind::new().with_frequency())
        .with_disks()
        .with_disks_list()
        .with_memory()
        .with_processes(sysinfo::ProcessRefreshKind::everything())
        .with_users_list()
    ;
    sys.refresh_specifics(refreshkind);
    //sys.refresh_all();


    // Functions //

    // User and Hostname //

    fn get_user(sys: &System) -> Vec<String> {
        // Create output string
        let mut output_string = Vec::new();

        // Get the current process's PID
        match get_current_pid() {
            Ok(pid) => {
                // Get information about the current process
                match sys.process(pid) {
                    Some(process) => {
                        // Get the username of the owner of the current process
                        match process.user_id() {
                            Some(uid) => {
                                // Create a string with the user and hostname, and push it to the output string
                                output_string.push(format!("{User}@{HostName}", User = sys.get_user_by_id(uid).unwrap().name(), HostName = sys.host_name().unwrap()));
                            }
                            None => {
                                // If the user ID is not available, only push the hostname to the output string
                                output_string.push(sys.host_name().unwrap());
                            }
                        }
                    }
                    None => {
                        // If the current process has no information, only push the hostname to the output string
                        output_string.push(sys.host_name().unwrap());
                    }
                }
            }
            Err(_) => {
                // If the current process's PID is not available, only push the hostname to the output string
                output_string.push(format!("{}", sys.host_name().unwrap()));
            }
        }

        // Create a partition with the length of the user and hostname
        output_string.push(format!("{}", "-".repeat(output_string[0].len())));

        output_string
    }


    // OS name //

    fn get_os (sys: &System) -> String {
        // Return the OS name
        return format!("OS: {}", sys.long_os_version().unwrap());
    }


    // Computer name //

    fn get_computer_name() -> String {
        use libmacchina::traits::GeneralReadout;
        let general:libmacchina::GeneralReadout = libmacchina::traits::GeneralReadout::new();

        // Return the computer name
        return format!("Host: {}", general.machine().unwrap());
    }


    // Kernel version //

    fn get_kernel_version() -> String {
        // Return the kernel version
        return format!("Kernel: {}", os_info::get().version());
    }


    // Uptime //

    fn get_uptime(sys: &System) -> String {
        // Get the uptime
        let uptime = sys.uptime();

        // Return the uptime
        return format!("Uptime: {Days}{Hours}{Minutes}",
            Days =
                match uptime / 86400 {
                    0 => "".to_string(),
                    1 => "1 day ".to_string(),
                    n => format!("{} days ", n),
                },
            Hours =
                match (uptime % 86400) / 3600 {
                    0 => "".to_string(),
                    1 => "1 hour ".to_string(),
                    n => format!("{} hours ", n),
                },
            Minutes =
                match (uptime % 3600) / 60 {
                    0 => "".to_string(),
                    1 => "1 minute ".to_string(),
                    n => format!("{} minutes ", n),
                },
        );
    }


    // Resolution //

    fn get_resolution() -> String {
        // Create Window
        let window = winit::window::WindowBuilder::new()
            .with_visible(false) // Hide window
            .build(&winit::event_loop::EventLoop::new())
            .unwrap();
        // Create Vector to store resolutions in
        let mut resoutputarray = Vec::new();
        // Get all connected monitors
        window.available_monitors().for_each(|monitor| {
            // Create string from monitor resolution and push to vector
            resoutputarray.push(format!("{}x{}", monitor.size().width, monitor.size().height));
        });

        // Return the resolution
        return format!("Resolution: {}", resoutputarray.join(", "));
    }


    // Packages //

    fn get_packages() -> String {
        use libmacchina::traits::PackageReadout;
        // Create vector to store the package information in
        let mut packageoutputarray = Vec::new();

        // Get all installed packages
        let packages:libmacchina::PackageReadout = libmacchina::traits::PackageReadout::new();
        for (packagemanager, packagecount) in packages.count_pkgs() {
            // Create string from package manager and package count and push to vector
            packageoutputarray.push(format!("{} ({})", packagecount, packagemanager.to_string()));
        }

        // Return the package information
        return format!("Packages: {}", packageoutputarray.join(", "));
    }


    // Theme //

    fn get_theme() -> String {
        // Get current theme
        match dark_light::detect() {
            dark_light::Mode::Dark  => { return "Theme: Dark".to_string()  },
            dark_light::Mode::Light => { return "Theme: Light".to_string() },
        }
    }


    // CPU name //

    fn get_cpu_name(sys: &System) -> String {
        // Return the CPU name
        return format!("CPU: {} x {} @ {:.1}GHz", sys.cpus().len(), sys.cpus()[0].brand().trim_end(), sys.cpus()[0].frequency() as f64/ 1000.0);
    }


    // Processes //

    fn get_processes(sys: &System) -> String {
        // Get and return processes
        return format!("Processes: {}", sys.processes().len()); //TODO: Add CPU usage
    }


    // RAM and Swap //

    fn get_ram(sys: &System) -> String {
        // Return the system's RAM
        return format!("Memory: {:.2} GB / {:.2} GB ({}%)", sys.used_memory() as f64/ 1073741824.00, sys.total_memory() as f64/ 1073741824.00, sys.used_memory() * 100 / sys.total_memory());
    }

    fn get_swap() -> String {
        // Get the system's memory information
        let swap = sys_info::mem_info().unwrap();

        // Return the system's swap
        return format!("Swap: {:.2} GB / {:.2} GB ({}%)", (swap.swap_total - swap.swap_free) as f64/ 1048576.00, swap.swap_total as f64/ 1048576.00, (swap.swap_total - swap.swap_free) * 100 / swap.swap_total);
    }


    // Disk information //

    fn get_disk_info(sys: &System) -> String {
        // Create vector to store disk information in
        let mut diskoutput = String::from("");

        // Get all disks
        for disk in sys.disks() {
            // Create string from disk information and push to output string
            diskoutput.push_str(&format!("Disk ({Disk}): {Used} GB / {Total} GB ({Percent}%)",
                Disk    = disk.mount_point().to_str().unwrap(),
                Used    = (disk.total_space() - disk.available_space()) / 1073741824,
                Total   = disk.total_space() / 1073741824,
                Percent = (disk.total_space() - disk.available_space()) * 100 / disk.total_space()
            ).as_str());

            // If the disk has a name, add it to the output string
            //if !disk.name().is_empty() { diskoutput.push_str(&format!("\n  Name: {}", disk.name().to_str().unwrap())); }

            // Add the disk's file system to the output string
            //diskoutput.push_str(&format!("\n  File System: {}", std::str::from_utf8(disk.file_system()).unwrap()));
        }

        diskoutput
    }


    // Battery //

    fn get_battery() -> String {
        use libmacchina::traits::BatteryReadout;

        // Create Battery trait
        let battery:libmacchina::BatteryReadout = libmacchina::traits::BatteryReadout::new();

        // Get battery information
        let percentage = battery.percentage();
        let ac_state = battery.status();
        let health = battery.health();

        match (percentage, ac_state, health) {
            // If all battery information is available
            (Ok(percentage), Ok(ac_state), Ok(health)) => {
                return format!("Battery: {}% ({})\n  Health: {}%", percentage, ac_state, health);
            }
            // If battery health is not available
            (Ok(percentage), Ok(ac_state), Err(_)) => {
                return format!("Battery: {}% ({})", percentage, ac_state);
            }
            // If no battery information is available
            (Err(_), _, _) | (_, Err(_), _) => {
                return "Battery: N/A".to_string();
            }
        }
    }


    // Locale //

    fn get_locale() -> String {
        // Get and Return the system's locale
        return format!("Locale: {}", sys_locale::get_locale().unwrap());
    }


    // Execution //

    // Add functions to output queue

    use std::io::{stdout, Write};
    use crossterm::{queue, style::{self}, cursor};

    if user           { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 0),  style::Print(&get_user(&sys)[0])).map_err(|e| eprintln!("Error: {}", e)).ok();   }
    if partition      { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 1),  style::Print(&get_user(&sys)[1])).map_err(|e| eprintln!("Error: {}", e)).ok();   }
    if os             { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 2),  style::Print(get_os(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();         }
    if computer_name  { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 3),  style::Print(get_computer_name())).map_err(|e| eprintln!("Error: {}", e)).ok();  }
    if kernel_version { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 4),  style::Print(get_kernel_version())).map_err(|e| eprintln!("Error: {}", e)).ok(); }
    if uptime         { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 5),  style::Print(get_uptime(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();     }
    if resolution     { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 6),  style::Print(get_resolution())).map_err(|e| eprintln!("Error: {}", e)).ok();     }
    if packages       { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 7),  style::Print(get_packages())).map_err(|e| eprintln!("Error: {}", e)).ok();       }
    if theme          { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 8),  style::Print(get_theme())).map_err(|e| eprintln!("Error: {}", e)).ok();          }
    if cpu_name       { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 9),  style::Print(get_cpu_name(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();   }
    if processes      { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 10), style::Print(get_processes(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();  }
    if ram            { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 11), style::Print(get_ram(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();        }
    if swap           { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 12), style::Print(get_swap())).map_err(|e| eprintln!("Error: {}", e)).ok();           }
    if disk_info      { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 13), style::Print(get_disk_info(&sys))).map_err(|e| eprintln!("Error: {}", e)).ok();  }
    if battery        { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 14), style::Print(get_battery())).map_err(|e| eprintln!("Error: {}", e)).ok();        }
    if locale         { queue!(stdout, cursor::MoveTo(cursor_position, initial_position_y + 15), style::Print(get_locale())).map_err(|e| eprintln!("Error: {}", e)).ok();         }

    stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();

    crossterm::execute!(stdout, cursor::MoveTo(cursor_position, final_position_y)).map_err(|e| eprintln!("Error: {}", e)).ok();

/*
    // Network interfaces
    println!("Network Adapters:");
    for (interface_name, data) in sys.networks() {
        println!("  {}: {}/{} B", interface_name, data.received(), data.transmitted());
    }
*/
    // Print CPU usage
    /*
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_secs(5));
    let mut cpuusage = 0.0;
    for cpu in sys.cpus() {
        cpuusage += cpu.cpu_usage();
    }
    println!("CPU Usage: {:.2}%", cpuusage / sys.cpus().len() as f32);
    */
}