# Debug with Wireshark

## Decrypt TLS session

```toml
# .cargo/config.toml

[env]
SSLKEYLOGFILE = { value = "./target/sslkeylog.log", relative = true }
```

Cargo will generate `./target/sslkeylog.log` file.

In Wireshark, Press: `Ctrl+Shift+P`:

```
Preferences → Protocols → TLS → (Pre)-Master-Secret log filename
```

Click Browse... and select your `<...>/target/sslkeys.log` file.

## Load SETU Lua plugin

```sh
mkdir ~/.local/lib/wireshark/plugins
```

Create link:

```sh
# NU
ln -s $"($env.PWD)/tools/wireshark/setu.lua" ~/.local/lib/wireshark/plugins/setu.lua

# Bash
ln -s "$PWD/tools/wireshark/setu.lua" ~/.local/lib/wireshark/plugins/setu.lua
```
