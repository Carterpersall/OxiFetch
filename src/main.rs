use std::path::Path;
use sysinfo::*;
use serde::*;

use machine_info::*;

// Config Struct //

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    user: bool,
    partition: bool,
    os: bool,
    computer_name: bool,
    kernel_version: bool,
    uptime: bool,
    resolution: bool,
    packages: bool,
    theme: bool,
    cpu_name: bool,
    gpu_info: bool,
    processes: bool,
    ram: bool,
    swap: bool,
    disk_info: bool,
    battery: bool,
    locale: bool,
    info_offset: usize
}


fn main() {
    // Configuration //

    let config: Config = confy::load_path(Path::new("./config.toml")).map_err(|e| eprintln!("Error: {}", e)).ok().unwrap();
    //let config: Config = confy::load("OxyFetch", None).unwrap();

    let image = vec![
        "                                ..,  ",
        "                    ....,,:;+ccllll  ",
        "      ...,,+:;  cllllllllllllllllll  ",
        ",cclllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "                                     ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "llllllllllllll  lllllllllllllllllll  ",
        "`'ccllllllllll  lllllllllllllllllll  ",
        "       `' \\*::  :ccllllllllllllllll  ",
        "                       ````''*::cll  ",
        "                                 ``  "
    ];
    let cursor_position = (image[0].len() + 1) as u16;


    // Initialization //

    let mut m = Machine::new();

    // Create stdout variable
    let mut stdout = stdout();

    use crossterm::style::Stylize;

    // Create space to print the image
    // Doesn't work in all terminals and may not be needed anymore
    //queue!(stdout, style::Print("\n".repeat(image.len())), cursor::MoveUp(image.len() as u16)).map_err(|e| eprintln!("Error: {}", e)).ok();
    //stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();

    // Create variables containing image information
    let final_position_y = cursor::position().unwrap().1 + (image.len() + 1) as u16;

    // Create System struct containing specified system information
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_frequency())
            .with_disks()
            .with_disks_list()
            .with_memory()
            .with_processes(ProcessRefreshKind::everything().with_user())
            .with_users_list()
    );

    // Update all needed information in the `System` struct.
    let refreshkind = RefreshKind::new()
        .with_cpu(CpuRefreshKind::new().with_frequency())
        .with_disks()
        .with_disks_list()
        .with_memory()
        .with_processes(ProcessRefreshKind::everything())
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
            dark_light::Mode::Dark    => { "Theme: Dark".to_string()  },
            dark_light::Mode::Light   => { "Theme: Light".to_string() },
            dark_light::Mode::Default => { "Unknown".to_string() },
        }
    }


    // CPU name //

    fn get_cpu_name(sys: &System) -> String {
        // Return the CPU name
        return format!("CPU: {} x {} @ {:.1}GHz", sys.cpus().len(), sys.cpus()[0].brand().trim_end(), sys.cpus()[0].frequency() as f64/ 1000.0);
    }


    // GPU name //

    fn get_gpu_name(mut m: Machine) -> Vec<GraphicCard> {
        let gpu = m.system_info().graphics;

        return gpu;
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

        return match (percentage, ac_state, health) {
            // If all battery information is available
            (Ok(percentage), Ok(ac_state), Ok(health)) => {
                format!("Battery: {}% ({})\n  Health: {}%", percentage, ac_state, health)
            }
            // If battery health is not available
            (Ok(percentage), Ok(ac_state), Err(_)) => {
                format!("Battery: {}% ({})", percentage, ac_state)
            }
            // If no battery information is available
            (Err(_), _, _) | (_, Err(_), _) => {
                "Battery: N/A".to_string()
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

    let mut i = 0;

    // If there is an offset to the information, print the lines of the image before the information
    if config.info_offset != 0 {
        for j in 0..config.info_offset {
            queue!(stdout, style::PrintStyledContent(image[j].cyan()), style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        }
        i = config.info_offset;
    }

    let user = get_user(&sys);

    if config.user           { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(format!("{}\n", &user[0]))).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.partition      { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(format!("{}\n", &user[1]))).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.os             { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_os(&sys) + "\n")                ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.computer_name  { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_computer_name() + "\n")         ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.kernel_version { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_kernel_version() + "\n")        ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.uptime         { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_uptime(&sys) + "\n")            ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.resolution     { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_resolution() + "\n")            ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.packages       { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_packages() + "\n")              ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.theme          { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_theme() + "\n")                 ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.cpu_name       { queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(get_cpu_name(&sys) + "\n")          ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.gpu_info       {
        let gpu_info = get_gpu_name(m);
        for gpu in gpu_info {
            queue!(stdout, style::PrintStyledContent(image[i].cyan()) , style::Print(format!("GPU: {}\n", gpu.name.to_string()))).map_err(|e| eprintln!("Error: {}", e)).ok();
            i += 1;
        }
    }
    if config.processes      { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_processes(&sys) + "\n")          ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.ram            { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_ram(&sys) + "\n")                ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.swap           { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_swap() + "\n")                   ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.disk_info      { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_disk_info(&sys) + "\n")          ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.battery        { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_battery() + "\n")                ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }
    if config.locale         { queue!(stdout, style::PrintStyledContent(image[i].cyan()), style::Print(get_locale() + "\n")                 ).map_err(|e| eprintln!("Error: {}", e)).ok(); i += 1; }

    // Queue the rest of the image
    for j in i..image.len() {
        queue!(stdout, style::PrintStyledContent(image[j].cyan()), style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
    }

    // Print the output queue
    stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();

    crossterm::execute!(stdout, cursor::MoveTo(cursor_position, final_position_y)).map_err(|e| eprintln!("Error: {}", e)).ok();
}