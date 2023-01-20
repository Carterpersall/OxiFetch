use std::io::Stdout;
use std::path::Path;

use libmacchina::traits::GeneralReadout;
use machine_info::*;
use serde::*;

// Config Structs //

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    image_name: String,
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

#[derive(Default, Debug, Serialize, Deserialize)]
struct Images {
    windows_10: Vec<String>,
    windows_11: Vec<String>
}


fn main() {
    // Configuration //

    // Load config files
    let config: Config = confy::load_path(Path::new("./config.toml")).map_err(|e| eprintln!("Error: {}", e)).ok().unwrap();
    //let config: Config = confy::load("OxyFetch", None).unwrap();
    let images: Images = confy::load_path(Path::new("./image.toml")).map_err(|e| eprintln!("Error: {}", e)).ok().unwrap();

    // Set image based on config
    let image = match config.image_name.as_str() {
        "windows_10" => images.windows_10,
        "windows_11" => images.windows_11,
        _ => images.windows_10
    };


    // Initialization //

    let mut m = Machine::new();
    let info = m.system_info();

    let general:libmacchina::GeneralReadout = libmacchina::traits::GeneralReadout::new();

    // Create stdout variable
    let mut stdout = stdout();


    // Functions //

    // User and Hostname //

    fn get_user() -> Vec<String> {
        // Create output string
        let mut output_string = Vec::new();

        // Get the current user and hostname, and push it to the output string
        output_string.push(whoami::username() + "@" + whoami::hostname().as_str());

        // Create a partition with the length of the user and hostname
        output_string.push(format!("{}", "-".repeat(output_string[0].len())));

        output_string
    }


    // OS name //

    fn get_os() -> String {
        // Return the OS name
        return match os_info::get().edition() {
            Some(edition) => format!("OS: {}", edition),
            None => format!("OS: {} {}", os_info::get().os_type(), os_info::get().version())
        };
    }


    // Computer name //

    fn get_computer_name(general: &libmacchina::GeneralReadout) -> String {
        // Return the computer name
        return match general.machine() {
            Ok(machine) => format!("Computer: {}", machine),
            Err(_) => "Computer: Unknown".to_string()
        };
    }


    // Kernel version //

    fn get_kernel_version(info: &SystemInfo) -> String {
        // Return the kernel version

        // If on Linux
        #[cfg(not(target_os = "windows"))]
        return format!("Kernel: {}", info.kernel_version);

        // If on Windows
        #[cfg(target_os = "windows")]
        return format!("Kernel: {}", os_info::get().version());
    }


    // Uptime //

    fn get_uptime(general: &libmacchina::GeneralReadout) -> String {
        // Get the uptime
        let uptime = match general.uptime() {
            Ok(uptime) => uptime,
            Err(_) => return "Uptime: Unknown".to_string()
        };

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
        // Create Vector to store resolutions in
        let mut output = Vec::new();

        for display in display_info::DisplayInfo::all().unwrap() {
            output.push(format!("{}x{}", (display.width as f32 * display.scale_factor), (display.height as f32 * display.scale_factor)));
        }

        return "Resolution: ".to_string() + output.join(", ").as_str();
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
            dark_light::Mode::Dark    => { "Theme: Dark".to_string()    },
            dark_light::Mode::Light   => { "Theme: Light".to_string()   },
            dark_light::Mode::Default => { "Theme: Unknown".to_string() }
        }
    }


    // CPU name //

    fn get_cpu_name(info: &SystemInfo) -> String {
        // Return the CPU name
        return format!("CPU: {} x {} @ {:.1}GHz", info.total_processors, info.processor.brand.trim_end(), sys_info::cpu_speed().unwrap() as f64 / 1000.0);
    }


    // GPU name //

    fn get_gpu_name(mut m: Machine) -> Vec<GraphicCard> {
        // Get the GPU information vector and return it
        return m.system_info().graphics;
    }


    // Processes //

    fn get_processes() -> String {
        // Get and return processes
        return format!("Processes: {}", sys_info::proc_total().unwrap()); //TODO: Add CPU usage
    }


    // RAM and Swap //

    fn get_ram() -> String {
        // Return the system's RAM
        let memory = sys_info::mem_info().unwrap();
        let used = (memory.total - memory.free) as f64 / 1048576.00;
        return format!("Memory: {:.2} GB / {:.2} GB ({}%)", used, memory.total as f64 / 1048576.00, used as u64 * 100 / (memory.total / 1048576));
    }

    fn get_swap() -> String {
        // Get the system's memory information
        let swap = sys_info::mem_info().unwrap();

        // Return the system's swap
        return format!("Swap: {:.2} GB / {:.2} GB ({}%)", (swap.swap_total - swap.swap_free) as f64 / 1048576.00, swap.swap_total as f64 / 1048576.00, (swap.swap_total - swap.swap_free) * 100 / swap.swap_total);
    }


    // Disk information //

    fn get_disk_info(info: &SystemInfo) -> Vec<String> {
        // Create vector to store disk information in
        let mut diskoutput: Vec<String> = Vec::new();

        // Get all disks
        for disk in info.disks.iter() {
            diskoutput.push(format!("Disk ({Disk}): {Used} GB / {Total} GB ({Percent}%)\n",
                Disk    = disk.mount_point.replace("\\", ""),
                Used    = (disk.size - disk.available) / 1073741824,
                Total   = disk.size / 1073741824,
                Percent = (disk.size - disk.available) * 100 / disk.size
            ));
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
                format!("Battery: {}% ({}) ({}% Health)", percentage, ac_state, health)
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

    // Function to print line of image
    fn print_image_line(index: usize, image: &Vec<String>, mut stdout: &Stdout) {
        // Check if the index is in bounds
        if index < image.len() {
            match queue!(stdout, style::PrintStyledContent(image[index].as_str().cyan())) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error: {}", e) }
            }
        } else {
            match queue!(stdout, style::Print(" ".repeat(image[0].len()))) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error: {}", e) }
            }
        }
    }

    // Add functions to output queue

    use crossterm::style::Stylize;
    use std::io::{stdout, Write};
    use crossterm::{queue, style::{self}};

    let mut i = 0;

    // If there is an offset to the information, print the lines of the image before the information
    if config.info_offset != 0 {
        for j in 0..config.info_offset {
            print_image_line(j, &image, &stdout);
            queue!(stdout, style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        }
        i = config.info_offset;
    }

    let user = get_user();

    if config.user {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(format!("{}\n", &user[0]))).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.partition {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(format!("{}\n", &user[1]))).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.os {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_os() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.computer_name {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_computer_name(&general) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.kernel_version {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_kernel_version(&info) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.uptime {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_uptime(&general) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.resolution {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_resolution() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.packages {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_packages() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.theme {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_theme() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.cpu_name {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_cpu_name(&info) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.gpu_info {
        let gpu_info = get_gpu_name(m);
        for gpu in gpu_info {
            print_image_line(i, &image, &stdout);
            queue!(stdout, style::Print(format!("GPU: {}\n", gpu.name.to_string()))).map_err(|e| eprintln!("Error: {}", e)).ok();
            i += 1;
        }
    }
    if config.processes {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_processes() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.ram {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_ram() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.swap {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_swap() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.disk_info {
        for disk in get_disk_info(&info) {
            print_image_line(i, &image, &stdout);
            queue!(stdout, style::Print(disk)).map_err(|e| eprintln!("Error: {}", e)).ok();
            i += 1;
        }
    }
    if config.battery {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_battery() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.locale {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_locale() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }

    // Queue the rest of the image
    if i < image.len() {
        for j in i..image.len() {
            print_image_line(j, &image, &stdout);
            queue!(stdout, style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        }
    }

    // Print the output queue
    stdout.flush().map_err(|e| eprintln!("Error: {}", e)).ok();
}