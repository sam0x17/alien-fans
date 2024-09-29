use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;

const DEFAULT_CURVE: Curve = Curve([
    0,   // <= 9C
    0,   // 10C-19C
    5,   // 20-29C
    10,  // 30-39C
    25,  // 40-49C
    50,  // 50-59C
    60,  // 60-69C
    75,  // 70-79C
    100, // 80-89C
    100, // >= 91C
])
.normalize();

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

struct CurveParsingError;

impl Display for CurveParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Curve must be defined as a newline-delimited list of 11 integers between 0 and 100"
        )
    }
}

struct Curve([u8; 10]);

impl Curve {
    // scales values so that 100 becomes 254, rounding up
    const fn normalize(&self) -> Self {
        let mut curve = self.0;
        let mut i = 0;
        loop {
            curve[i] = curve[i].saturating_mul(254).saturating_add(50) / 100;
            i += 1;
            if i == curve.len() {
                break;
            }
        }
        Curve(curve)
    }
}

impl FromStr for Curve {
    type Err = CurveParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut curve = [0; 10];
        let mut i = 0;
        for line in s.lines() {
            if i >= curve.len() {
                return Err(CurveParsingError);
            }
            curve[i] = line.parse().map_err(|_| CurveParsingError)?;
            i += 1;
        }
        if i != curve.len() {
            return Err(CurveParsingError);
        }
        Ok(Curve(curve))
    }
}

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
    let curve = DEFAULT_CURVE;

    // Find the dell_smm hwmon interface
    let Some(hwmon_path) = find_dell_smm_hwmon() else {
        eprintln!("dell_smm hwmon interface not found.");
        exit(1);
    };
    println!(
        "Found dell_smm hwmon interface at: {}",
        hwmon_path.display()
    );

    loop {
        // CPU fan
    }
    // Set PWM values for pwm1, pwm2, and pwm3
    for pwm_number in 1..=3 {
        match set_pwm(&hwmon_path, pwm_number, 100) {
            Ok(_) => println!("Successfully set pwm{}", pwm_number),
            Err(e) => eprintln!("Failed to set pwm{}: {}", pwm_number, e),
        }
    }
}
