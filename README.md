# CPU Steal Time Monitor

Lightweight Linux service that monitors CPU steal time (`%st`) on VPS instances and alerts when "noisy neighbor" issues occur.

## Why This Matters

On shared CPU instances (Vultr Regular, DigitalOcean Standard, etc.), your VM shares physical CPU cores with other customers. When the hypervisor gives your CPU time to someone else, that shows as **steal time**.

- **< 5% steal**: Normal, acceptable
- **5-15% steal**: Elevated — monitor for patterns, consider upgrade
- **> 15% steal**: Critical — noisy neighbors are impacting performance consistently

## How It Works

- Reads `/proc/stat` every 60 seconds
- Calculates moving average over 5 samples (5 minutes)
- Prints color-coded status:
  - 🟢 `✅ OK` — steal below threshold
  - 🟡 `⚠️ WARNING` — steal > 5%
  - 🔴 `🚨 CRITICAL` — steal > 15%

## Installation on Your Vultr Server

```bash
# Clone or copy the project to your server
ssh root@your-vultr-ip
git clone <your-repo-url> /opt/cpu-steal-monitor
cd /opt/cpu-steal-monitor

# Build release binary
cargo build --release

# The binary will be at:
# ./target/release/cpu-steal-monitor
```

## Quick Test

```bash
# Run it manually to test
./target/release/cpu-steal-monitor
```

Expected output:
```
🔍 CPU Steal Time Monitor Started
   Warning threshold: >5%
   Critical threshold: >15%
   Sample interval: 60s

[2025-01-15 14:32:01] Steal: 0.3% (avg over 1) ✅ OK
[2025-01-15 14:33:01] Steal: 0.5% (avg over 2) ✅ OK
[2025-01-15 14:34:01] Steal: 6.2% (avg over 3) ⚠️ WARNING
   ⚠️ Warning: Elevated steal time. Monitor for patterns.
```

## Run as a Service (systemd)

Create a systemd service to run it automatically:

```bash
cat > /etc/systemd/system/cpu-steal-monitor.service << 'EOF'
[Unit]
Description=CPU Steal Time Monitor
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/cpu-steal-monitor
ExecStart=/opt/cpu-steal-monitor/target/release/cpu-steal-monitor
Restart=always
RestartSec=10

# Logging to journal
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
systemctl daemon-reload
systemctl enable cpu-steal-monitor
systemctl start cpu-steal-monitor

# Check it's running
systemctl status cpu-steal-monitor

# View logs
journalctl -u cpu-steal-monitor -f
```

## Viewing Logs

```bash
# Follow logs in real-time
journalctl -u cpu-steal-monitor -f

# View last 100 lines
journalctl -u cpu-steal-monitor -n 100

# View since today
journalctl -u cpu-steal-monitor --since today
```

## Configuration

Edit `src/main.rs` to customize thresholds:

```rust
const STEAL_WARNING_THRESHOLD: f32 = 5.0;   // Alert when steal > 5%
const STEAL_CRITICAL_THRESHOLD: f32 = 15.0; // Critical when steal > 15%
const SAMPLE_INTERVAL_SECONDS: u64 = 60;    // Check every 60 seconds
const MOVING_AVORAGE_SAMPLES: usize = 5;    // Average over 5 samples
```

Rebuild after changes:
```bash
cargo build --release
systemctl restart cpu-steal-monitor
```

## Manual Check (one-liner)

For a quick one-time check without the monitor:

```bash
# Show current steal time (reads directly from /proc/stat)
grep 'cpu ' /proc/stat | awk '{print "Steal: " 100*$8/($2+$3+$4+$5+$6+$7+$8+$9) "%"}'

# Alternative: using mpstat if available
mpstat 1 1 | awk '/Average/ {print "Steal: " $(NF-4) "%"}'
```

## Interpretation

| Steal Time | Meaning | Action |
|------------|---------|--------|
| 0-2% | Excellent | No action needed |
| 2-5% | Normal | Acceptable for shared CPU |
| 5-10% | Elevated | Monitor closely; consider upgrade during sustained periods |
| 10-20% | High | Performance impacted; upgrade recommended |
| >20% | Severe | Urgent — noisy neighbors degrading service |

## When to Upgrade

**Upgrade to Dedicated CPU when:**
- Steal time consistently > 10% during your peak hours
- You see "CRITICAL" alerts multiple times per day
- Your customers report slow analysis times that correlate with high steal

**Stay on Shared CPU if:**
- Occasional spikes to 5-10% but quickly returns to < 5%
- Most readings are under 3%
- You're cost-conscious and can tolerate occasional slowdown

## Troubleshooting

**"Failed to read CPU stats - are you running on Linux?"**
- This only works on Linux (your Vultr server)
- Won't work on macOS locally

**"Permission denied" reading /proc/stat**
- Run as root or with sudo (not needed if running as systemd service with User=root)

**Service won't start**
- Check `journalctl -u cpu-steal-monitor -n 50` for errors
- Verify binary path: `ls -la /opt/cpu-steal-monitor/target/release/cpu-steal-monitor`
- Check SELinux/AppArmor if you're using them

## License

MIT
