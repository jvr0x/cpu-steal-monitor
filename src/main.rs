/// CPU Steal Time Monitor
///
/// Monitors CPU steal time on Linux VPS instances and sends alerts when thresholds are exceeded.
/// Useful for detecting "noisy neighbor" issues on shared CPU cloud instances.

use std::process::Command;
use std::time::Duration;
use std::thread;

/// Thresholds for steal time alerts
const STEAL_WARNING_THRESHOLD: f32 = 5.0;  // Alert if steal > 5%
const STEAL_CRITICAL_THRESHOLD: f32 = 15.0; // Critical alert if steal > 15%

/// Sampling configuration
const SAMPLE_INTERVAL_SECONDS: u64 = 60;    // Check every minute
const MOVING_AVORAGE_SAMPLES: usize = 5;    // Average over 5 samples

fn main() {
    println!("🔍 CPU Steal Time Monitor Started");
    println!("   Warning threshold: >{}%", STEAL_WARNING_THRESHOLD);
    println!("   Critical threshold: >{}%", STEAL_CRITICAL_THRESHOLD);
    println!("   Sample interval: {}s", SAMPLE_INTERVAL_SECONDS);
    println!();

    let mut recent_readings: Vec<f32> = Vec::with_capacity(MOVING_AVORAGE_SAMPLES);

    loop {
        match get_steal_time() {
            Some(steal) => {
                recent_readings.push(steal);
                if recent_readings.len() > MOVING_AVORAGE_SAMPLES {
                    recent_readings.remove(0);
                }

                let avg_steal: f32 = recent_readings.iter().sum::<f32>() / recent_readings.len() as f32;

                let status = if avg_steal >= STEAL_CRITICAL_THRESHOLD {
                    "🚨 CRITICAL"
                } else if avg_steal >= STEAL_WARNING_THRESHOLD {
                    "⚠️  WARNING"
                } else {
                    "✅ OK"
                };

                println!(
                    "[{}] Steal: {:.1}% (avg over {}) {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    steal,
                    recent_readings.len(),
                    status
                );

                if avg_steal >= STEAL_CRITICAL_THRESHOLD {
                    eprintln!(
                        "   🔥 CRITICAL: Sustained high steal time detected! Consider upgrading to dedicated CPU."
                    );
                } else if avg_steal >= STEAL_WARNING_THRESHOLD {
                    eprintln!(
                        "   ⚠️  Warning: Elevated steal time. Monitor for patterns."
                    );
                }
            }
            None => {
                eprintln!("❌ Failed to read CPU stats - are you running on Linux?");
                thread::sleep(Duration::from_secs(SAMPLE_INTERVAL_SECONDS));
                continue;
            }
        }

        thread::sleep(Duration::from_secs(SAMPLE_INTERVAL_SECONDS));
    }
}

/// Extracts CPU steal time from /proc/stat on Linux
///
/// Returns the steal percentage as a float, or None if unable to parse.
/// Steal time represents the time the CPU wanted to run but the hypervisor
/// scheduled another VM instead (noisy neighbor problem).
fn get_steal_time() -> Option<f32> {
    let output = Command::new("cat")
        .arg("/proc/stat")
        .output()
        .ok()?;

    let content = String::from_utf8(output.stdout).ok()?;

    // /proc/stat first line format:
    // cpu  user nice system idle iowait irq softirq steal guest guest_nice
    let first_line = content.lines().next()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 9 {
        return None;
    }

    // steal is at index 8 (0-indexed after "cpu")
    let steal_jiffies: u64 = parts[8].parse().ok()?;

    // For simplicity, we read total CPU time and calculate percentage
    // In production, you'd want delta tracking between reads
    let mut total_jiffies: u64 = 0;
    for (i, part) in parts.iter().enumerate().skip(1) {
        if i <= 9 {
            // Sum user, nice, system, idle, iowait, irq, softirq, steal
            total_jiffies += part.parse::<u64>().unwrap_or(0);
        }
    }

    if total_jiffies == 0 {
        return Some(0.0);
    }

    let steal_percent = (steal_jiffies as f32 / total_jiffies as f32) * 100.0;
    Some(steal_percent)
}
