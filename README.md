```
  ____ _____   ____  ____  _   _ ____
 / ___|  ___| |  _ \|  _ \| \ | / ___|
| |   | |_    | | | | | | |  \| \___ \
| |___|  _|   | |_| | |_| | |\  |___) |
 \____|_|     |____/|____/|_| \_|____/
```

# Cloudflare DDNS Updater

Automatically update Cloudflare A records when your IP changes.

## Features

- Reliable IP detection using multiple sources for consensus
- Support for multiple DNS records
- Systemd service with security hardening
- Configurable update intervals
- Detailed logging
- Flexible configuration via file or command-line arguments

## Usage

You can configure the updater either through a config file or command-line arguments:

```bash
# Using config file
cloudflare-ddns -c /path/to/config.toml

# Using command-line arguments
cloudflare-ddns --auth-email your@email.com --auth-key your_api_key --zone-id your_zone_id --records example.com --records subdomain.example.com

# Mix both (command-line args override config file)
cloudflare-ddns -c /path/to/config.toml --records override.example.com
```

### Command-line Options

```
Options:
  -c, --config <CONFIG>
          Path to config file (optional if all other args are provided)
  -e, --auth-email <AUTH_EMAIL>
          Cloudflare account email
  -k, --auth-key <AUTH_KEY>
          Cloudflare API key or API token
  -z, --zone-id <ZONE_ID>
          Cloudflare zone ID from domain overview page
  -i, --reload-interval <RELOAD_INTERVAL>
          Update interval in seconds [default: 300]
  -r, --records <RECORDS>
          DNS records to update (can be specified multiple times)
  -d, --debug
          Enable debug logging
  -h, --help
          Print help
  -V, --version
          Print version
```

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/tn3w/cloudflare-ddns
cd cloudflare-ddns
```

### 2. Build the Binary

```bash
cargo build --release
```

### 3. Create System User

Create a dedicated system user and group for the service:
```bash
sudo useradd -r -s /bin/false cloudflare-ddns
```

### 4. Create Required Directories

```bash
# Create configuration and log directories
sudo mkdir -p /etc/cloudflare-ddns
sudo mkdir -p /var/log/cloudflare-ddns

# Set proper ownership
sudo chown cloudflare-ddns:cloudflare-ddns /var/log/cloudflare-ddns
```

### 5. Install Files

```bash
# Copy binary
sudo cp target/release/cloudflare-ddns /usr/local/bin/
sudo chmod 755 /usr/local/bin/cloudflare-ddns

# Copy and configure service file
sudo cp cloudflare-ddns.service /etc/systemd/system/
sudo chmod 644 /etc/systemd/system/cloudflare-ddns.service

# Copy and configure config file
sudo cp config.toml.example /etc/cloudflare-ddns/config.toml
sudo chmod 644 /etc/cloudflare-ddns/config.toml
sudo chown cloudflare-ddns:cloudflare-ddns /etc/cloudflare-ddns/config.toml
```

### 6. Configure the Service

Edit the configuration file with your Cloudflare credentials:
```bash
sudo nano /etc/cloudflare-ddns/config.toml
```

Example configuration:
```toml
# Required fields (unless provided via command-line)
auth_email = "your@email.com"
auth_key = "your_cloudflare_api_key"
zone_id = "your_zone_id"

# Optional: Update interval in seconds (default: 300 = 5 minutes)
reload_interval = 300

# Required: List of DNS records to update
records = [
    "example.com",
    "subdomain.example.com"
]
```

### 7. Start the Service

```bash
# Reload systemd to recognize the new service
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable cloudflare-ddns

# Start the service
sudo systemctl start cloudflare-ddns
```

### 8. Verify Installation

Check if the service is running properly:
```bash
# Check service status
sudo systemctl status cloudflare-ddns

# View logs
sudo journalctl -u cloudflare-ddns -f
```

## License

Copyright 2025 TN3W

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.