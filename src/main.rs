use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const DEBUG_MODE: bool = {
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
    #[cfg(target_os = "linux")]
    {
        std::option_env!("DEBUG_MODE").is_some()
    }
};

// Function to find the 'dell_smm' hwmon interface
fn find_dell_smm_hwmon() -> Option<PathBuf> {
    let hwmon_path = Path::new("/sys/class/hwmon");
    if hwmon_path.is_dir() {
        for entry in fs::read_dir(hwmon_path).unwrap() {
            let entry = entry.unwrap();
            let hwmon_name_file = entry.path().join("name");

            if let Ok(content) = fs::read_to_string(hwmon_name_file) {
                if content.trim() == "dell_smm" {
                    return Some(entry.path());
                }
            }
        }
    }
    if DEBUG_MODE {
        return Some(PathBuf::from("/sys/class/hwmon/hwmon-debug"));
    }
    None
}

// Helper function to write to pwmX
fn set_pwm(hwmon_path: &Path, pwm_number: u8, value: u8) -> io::Result<()> {
    if DEBUG_MODE {
        println!("Setting pwm{} to {}", pwm_number, value);
        return Ok(());
    }
    let pwm_path = hwmon_path.join(format!("pwm{}", pwm_number));
    let mut file = File::create(pwm_path)?;
    file.write_all(value.to_string().as_bytes())?;
    Ok(())
}

fn main() {
    // Find the dell_smm hwmon interface
    if let Some(hwmon_path) = find_dell_smm_hwmon() {
        println!(
            "Found dell_smm hwmon interface at: {}",
            hwmon_path.display()
        );

        // Set PWM values for pwm1, pwm2, and pwm3
        for pwm_number in 1..=3 {
            match set_pwm(&hwmon_path, pwm_number, 255) {
                Ok(_) => println!("Successfully set pwm{}", pwm_number),
                Err(e) => eprintln!("Failed to set pwm{}: {}", pwm_number, e),
            }
        }
    } else {
        println!("dell_smm hwmon interface not found.");
    }
}
