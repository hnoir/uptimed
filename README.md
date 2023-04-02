## **uptimed**

**Warning**: This program may be used for some simple cases when the amount of URLs is too low. If you plan to monitor hundreds of targets you should consider more professional solutions that enable more configuration and distribute the fetching process on multiple hosts.

**uptimed**, which stands for **uptime d**aemon, is a Rust program that reads a file containing one one URL per line, fetches each one and sends a desktop notification when one of the URLs is unsuccessful.  
Timings are specified in the configuration.

The path to the configuration file can be specified using the `-c` flag, otherwise the default path ($HOME/.config/uptimed/config.yml) will be used.

Here's an example configuration file:
```yml
# Path to the file containing target URLs.
targets_path: "/path/to/targets"

# How much time between requests?
request_interval: 0s

# How much time between one complete scan and the next one?
scan_interval: 15m

# List of custom HTTP headers and values to use in every request.
custom_headers:
  - name: "X-MyHeader"
    value: "my-value"
  - name: "Authorization"
    value: "Bearer token"

```

## Notes
This program assumes that your OS is based on Linux and that your desktop environment follows the XDG specification (thus Gnome, KDE, XFCE, etc..).
