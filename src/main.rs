use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;

const POLLING_INTERVAL: Duration = Duration::from_millis(5000);

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

const DEBUG_MODE: bool = cfg!(not(target_os = "linux")) || std::option_env!("DEBUG_MODE").is_some();

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

fn find_hwmon_device(device_name: &str) -> Option<PathBuf> {
    let hwmon_path = Path::new("/sys/class/hwmon");
    if hwmon_path.is_dir() {
        for entry in fs::read_dir(hwmon_path).unwrap() {
            let entry = entry.unwrap();
            let hwmon_name_file = entry.path().join("name");

            if let Ok(content) = fs::read_to_string(hwmon_name_file) {
                if content.trim().contains(device_name) {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

// Struct to manage PWM file descriptors
struct PwmController {
    pwm_files: Vec<io::Result<File>>,
}

impl PwmController {
    fn new(hwmon_path: &Path) -> io::Result<Self> {
        let mut pwm_files = Vec::new();
        for pwm_number in 1..=3 {
            let pwm_path = hwmon_path.join(format!("pwm{}", pwm_number));
            let file = OpenOptions::new().write(true).open(pwm_path);
            pwm_files.push(file);
        }
        Ok(PwmController { pwm_files })
    }

    fn set_pwm(&mut self, pwm_number: usize, value: u8) -> io::Result<()> {
        if DEBUG_MODE {
            println!("Setting pwm{} to {}", pwm_number, value);
            return Ok(());
        }

        if let Some(file) = self.pwm_files.get_mut(pwm_number - 1) {
            if let Ok(file) = file {
                file.write_all(value.to_string().as_bytes())?;
                file.flush()?; // Flush to ensure it's written to the hardware
            }
        }
        Ok(())
    }
}

fn read_coretemp_temp(hwmon_path: &Path) -> io::Result<u64> {
    let temp_path = hwmon_path.join("temp1_input");
    let temp_str = fs::read_to_string(temp_path)?;
    let temp = temp_str
        .trim()
        .parse::<u64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(temp / 1000)
}

fn main() {
    let curve = DEFAULT_CURVE;

    // Find the dell_smm hwmon interface
    let Some(dell_smm) = find_hwmon_device("dell_smm") else {
        eprintln!("dell_smm hwmon interface not found.");
        exit(1);
    };
    println!("Found dell_smm hwmon interface at: {}", dell_smm.display());

    let Some(coretemp) = find_hwmon_device("coretemp") else {
        eprintln!("coretemp hwmon interface not found.");
        exit(1);
    };
    println!("Found coretemp hwmon interface at: {}", coretemp.display());

    // Initialize PwmController
    let mut pwm_controller = PwmController::new(&dell_smm).unwrap();

    loop {
        let cpu_temp = read_coretemp_temp(&coretemp).unwrap();
        println!("CPU temperature: {}C", cpu_temp);

        // Adjust PWM based on temperature
        //let pwm_value = curve.0[(cpu_temp / 10) as usize];
        pwm_controller.set_pwm(1, 200).unwrap();

        std::thread::sleep(POLLING_INTERVAL);
    }
}
