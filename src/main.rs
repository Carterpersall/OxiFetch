use std::io::Stdout;
use std::path::Path;

use libmacchina::traits::GeneralReadout;
use serde::*;
use sysinfo::{DiskExt, SystemExt};

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

    // Initialize libmacchina
    let general: libmacchina::GeneralReadout = libmacchina::traits::GeneralReadout::new();

    // Initialize sysinfo
    let mut sys: sysinfo::System = sysinfo::SystemExt::new_with_specifics(sysinfo::RefreshKind::new().with_disks());
    sys.refresh_disks_list();


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

    fn get_os(sys: &sysinfo::System) -> String {
        return format!("OS: {}", sys.long_os_version().unwrap());
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

    fn get_kernel_version() -> String {
        // Return the kernel version
        return format!("Kernel: {}", sys_info::os_release().unwrap());
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
        #[cfg(target_os = "linux")]{
            fn count_dpkg() -> usize {
                use rust_search::SearchBuilder;

                let dpkg_dir = Path::new("/var/lib/dpkg/info");

                SearchBuilder::default()
                    .location(dpkg_dir)
                    .search_input(".\\.list")
                    .build()
                    .count()
            }

            return format!("Packages: {} (Dpkg)", count_dpkg());
        }

        #[cfg(not(target_os = "linux"))] {
            use libmacchina::traits::PackageReadout;

            // Create vector to store the package information in
            let mut packageoutputarray = Vec::new();

            // Get all installed packages
            let packages: libmacchina::PackageReadout = libmacchina::traits::PackageReadout::new();
            for (packagemanager, packagecount) in packages.count_pkgs() {
                // Create string from package manager and package count and push to vector
                packageoutputarray.push(format!("{} ({})", packagecount, packagemanager.to_string()));
            }

            // Return the package information
            return format!("Packages: {}", packageoutputarray.join(", "));
        }
    }


    // Theme //

    fn get_theme() -> String {
        // Get and return current theme
        match dark_light::detect() {
            dark_light::Mode::Dark    => { "Theme: Dark".to_string()    },
            dark_light::Mode::Light   => { "Theme: Light".to_string()   },
            dark_light::Mode::Default => { "Theme: Unknown".to_string() }
        }
    }


    // CPU name //

    fn get_cpu_name(_general: &libmacchina::GeneralReadout) -> String {
        // https://github.com/GuillaumeGomez/sysinfo/blob/master/src/windows/cpu.rs#L388
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        fn get_vendor_id_and_brand() -> String {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::__cpuid;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::__cpuid;

            unsafe fn add_u32(v: &mut Vec<u8>, i: u32) {
                let i = &i as *const u32 as *const u8;
                v.push(*i);
                v.push(*i.offset(1));
                v.push(*i.offset(2));
                v.push(*i.offset(3));
            }

            unsafe {
                // First, we try to get the complete name.
                let res = __cpuid(0x80000000);
                let n_ex_ids = res.eax;
                let brand = if n_ex_ids >= 0x80000004 {
                    let mut extdata = Vec::with_capacity(5);

                    for i in 0x80000000..=n_ex_ids {
                        extdata.push(__cpuid(i));
                    }

                    // 4 * u32 * nb_entries
                    let mut out = Vec::with_capacity(4 * std::mem::size_of::<u32>() * 3);
                    // Iterate over extdata and create vector of utf-8 values
                    for data in extdata.iter().take(5).skip(2) {
                        add_u32(&mut out, data.eax);
                        add_u32(&mut out, data.ebx);
                        add_u32(&mut out, data.ecx);
                        add_u32(&mut out, data.edx);
                    }
                    let mut pos = 0;
                    for e in out.iter() {
                        if *e == 0 {
                            break;
                        }
                        pos += 1;
                    }
                    // Convert vector of utf-8 values to a string and return it
                    match std::str::from_utf8(&out[..pos]) {
                        Ok(s) => s.to_owned(),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };

                // Return the full name
                brand
            }
        }

        // Return the CPU name
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows"))]
        let output = format!("CPU: {} x {} @ {:.1}GHz", sys_info::cpu_num().unwrap(), get_vendor_id_and_brand().trim_end(), sys_info::cpu_speed().unwrap() as f64 / 1000.0);

        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(target_os = "windows")))]
        let output = format!("CPU: {} x {} @ {:.1}GHz", _general.cpu_cores().unwrap(), get_vendor_id_and_brand().trim_end(), sys_info::cpu_speed().unwrap() as f64 / 1000.0);

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let output = format!("CPU: {} x {} @ {:.1}GHz", _general.cpu_cores().unwrap(), _general.cpu_model_name().unwrap(), sys_info::cpu_speed().unwrap() as f64 / 1000.0);

        return output;
    }


    // GPU name //

    fn get_gpu_name() -> Vec<String> {
        // Create the output vector
        let mut output = Vec::new();

        // Welcome to match statement hell
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::System::Registry::{HKEY};
            let mut hkey = HKEY::default();
            // Open the location where some DirectX information is stored
            match windows_sys::Win32::System::Registry::RegOpenKeyW(
                windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Microsoft\\DirectX\\".encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr(),
                &mut hkey
            ) {
                windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                    // Get the parent key's LastSeen value
                    let mut lastseen = [0u8; 15];
                    let mut size = lastseen.len() as u32;
                    match windows_sys::Win32::System::Registry::RegQueryValueExW(
                        hkey,
                        "LastSeen".encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        lastseen.as_mut_ptr(),
                        &mut size,
                    ) {
                        windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                            // Get the parent key's subkey count and the maximum length of the subkeys
                            let mut key_count = 0;
                            let mut max_key_len = 0;
                            match windows_sys::Win32::System::Registry::RegQueryInfoKeyW(
                                hkey,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                &mut key_count,
                                &mut max_key_len,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                            ) {
                                windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                                    // Iterate over the parent key's subkeys and find the ones with the same LastSeen value
                                    for i in 1..key_count {
                                        let mut subkey = [0u16; 50];
                                        let mut size = max_key_len + 1;
                                        match windows_sys::Win32::System::Registry::RegEnumKeyExW(
                                            hkey,
                                            i,
                                            subkey.as_mut_ptr(),
                                            &mut size,
                                            std::ptr::null_mut(),
                                            std::ptr::null_mut(),
                                            std::ptr::null_mut(),
                                            std::ptr::null_mut(),
                                        ) {
                                            windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                                                // Open the subkey
                                                let mut subkey_hkey = HKEY::default();
                                                match windows_sys::Win32::System::Registry::RegOpenKeyW(
                                                    hkey,
                                                    subkey.as_mut_ptr(),
                                                    &mut subkey_hkey
                                                ) {
                                                    windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                                                        // Get the subkey's LastSeen value
                                                        let mut subkey_lastseen = [0u8; 15];
                                                        let mut size = subkey_lastseen.len() as u32;
                                                        match windows_sys::Win32::System::Registry::RegQueryValueExW(
                                                            subkey_hkey,
                                                            "LastSeen".encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr(),
                                                            std::ptr::null_mut(),
                                                            std::ptr::null_mut(),
                                                            subkey_lastseen.as_mut_ptr(),
                                                            &mut size
                                                        ) {
                                                            windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                                                                // If the subkey's LastSeen value is the same as the parent key's, get the subkey's Description value
                                                                if subkey_lastseen == lastseen {
                                                                    let mut description = [0u16; 50];
                                                                    let mut size = (description.len() + 100) as u32;
                                                                    match windows_sys::Win32::System::Registry::RegQueryValueExW(
                                                                        subkey_hkey,
                                                                        "Description".encode_utf16().chain([0u16]).collect::<Vec<u16>>().as_mut_ptr(),
                                                                        std::ptr::null_mut(),
                                                                        std::ptr::null_mut(),
                                                                        description.as_mut_ptr() as *mut u8,
                                                                        &mut size
                                                                    ) {
                                                                        windows_sys::Win32::Foundation::ERROR_SUCCESS => {
                                                                            let description_string = String::from_utf16_lossy(&description).trim().replace("\0", "");
                                                                            // Exclude the Microsoft Basic Render Driver
                                                                            if description_string != "Microsoft Basic Render Driver" {
                                                                                // Add the GPU name to the output vector
                                                                                output.push(description_string.to_string());
                                                                            }
                                                                        },
                                                                        e => { eprintln!("Error {}", e); }
                                                                    }
                                                                }
                                                            },
                                                            e => { eprintln!("Error {}", e); }
                                                        }
                                                    },
                                                    e => { eprintln!("Error {}", e); }
                                                }
                                            },
                                            e => { eprintln!("Error {}", e); }
                                        }
                                    }
                                },
                                e => { eprintln!("Error {}", e); }
                            }
                        },
                        e => { eprintln!("Error {}", e); }
                    }
                },
                e => { eprintln!("Error {}", e); }
            }

            // Close open key
            windows_sys::Win32::System::Registry::RegCloseKey(hkey);
        }

        #[cfg(not(target_os = "windows"))]
        output.push("Not Implemented".to_string());

        // Return the output vector
        output
    }


    // Processes //

    fn get_processes() -> String {
        // Get and return processes
        return format!("Processes: {}", sys_info::proc_total().unwrap()); //TODO: Add CPU usage
    }


    // RAM and Swap //

    fn get_ram() -> String {
        // Get the system's memory information
        let memory = sys_info::mem_info().unwrap();
        // Calculate the amount of memory used
        let used = (memory.total - memory.free) as f64 / 1048576.00;
        // Return the system's RAM
        return format!("Memory: {:.2} GB / {:.2} GB ({}%)", used, memory.total as f64 / 1048576.00, used as u64 * 100 / (memory.total / 1048576));
    }

    fn get_swap() -> String {
        // Get the system's memory information
        let swap = sys_info::mem_info().unwrap();

        // Return the system's swap
        return format!("Swap: {:.2} GB / {:.2} GB ({}%)", (swap.swap_total - swap.swap_free) as f64 / 1048576.00, swap.swap_total as f64 / 1048576.00, (swap.swap_total - swap.swap_free) * 100 / swap.swap_total);
    }


    // Disk information //

    fn get_disk_info(sys: &sysinfo::System) -> Vec<String> {
        // Create vector to store disk information in
        let mut diskoutput: Vec<String> = Vec::new();

        // Get all disks
        for disk in sys.disks() {
            // Create string from disk information and push to output string
            diskoutput.push(format!("Disk ({Disk}): {Used} GB / {Total} GB ({Percent}%)\n",
                Disk = disk.mount_point().to_str().unwrap().replace("\\", ""),
                Used = (disk.total_space() - disk.available_space()) / 1073741824,
                Total = disk.total_space() / 1073741824,
                Percent = (disk.total_space() - disk.available_space()) * 100 / disk.total_space()
            ));
        }

        // Return the output vector
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

    // Create stdout variable
    let mut stdout = stdout();

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

    // The current line
    let mut i = 0;

    // If there is an offset to the information, print the lines of the image before the information
    if config.info_offset != 0 {
        for j in 0..config.info_offset {
            print_image_line(j, &image, &stdout);
            queue!(stdout, style::Print("\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        }
        i = config.info_offset;
    }

    let user = if config.user || config.partition { get_user() } else { Vec::new() };

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
        queue!(stdout, style::Print(get_os(&sys) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.computer_name {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_computer_name(&general) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.kernel_version {
        print_image_line(i, &image, &stdout);
        queue!(stdout, style::Print(get_kernel_version() + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
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
        queue!(stdout, style::Print(get_cpu_name(&general) + "\n")).map_err(|e| eprintln!("Error: {}", e)).ok();
        i += 1;
    }
    if config.gpu_info {
        let gpu_info = get_gpu_name();
        for gpu in gpu_info {
            print_image_line(i, &image, &stdout);
            queue!(stdout, style::Print(format!("GPU: {}\n", gpu))).map_err(|e| eprintln!("Error: {}", e)).ok();
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
        for disk in get_disk_info(&sys) {
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