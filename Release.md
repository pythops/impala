## 0.6.0 - 2025-11-27

- Show all the configured networks
- Encode network names correctly
- Share network with QR code
- Breaking change: remove config for connect. set to space or enter.

## 0.5.0 - 2025-11-21

- Support for WPA2 Entreprise
- Support for Eduroam
- Make help banner responsive
- Improve error messages
- Fix crash when wifi adapter is switched off
- Reset terminal on error

## v0.4.1 - 2025-10-23

- Fix display issue on light mode background

## v0.4 - 2025-10-16

- Adjust new network section size.
- Rearrange sections for better layout.
- Update Rust edition.
- Replace help popup with help banner.
- Enable network connect/disconnect with `Enter` key.

## v0.3.0 - 2025-07-10

- Added an option to sensor password, thanks to [OneRandom1509](https://github.com/OneRandom1509)
- Bump dependencies
- Minor fixes

## v0.2.4 - 2024-11-17

### Added

- Detect when the device is soft/hard blocked

### Fix

- fg color for light mode background

## v0.2.3 - 2024-08-29

### Changed

- Remove uid check before starting the app.

## v0.2.2 - 2024-08-28

### Update

- Responsive layout

### Changed

- using stdout instead of stderr for the terminal handler
- set tick rate to 2sec

## v0.2.1 - 2024-06-27

### Added

- Signal strength in %
- Show Security and Frequency for connected network
- Choose startup mode from cli or config

## v0.2 - 2024-06-17

### Added

- Access Point mode
- Show connected devices on Access Point mode
- Turn On/Off device
- Switch between AP and Station mode
- Enable/Disable auto connect for known networks

## v0.1.1 - 2024-06-10

### Fixed

- Crash when the vendor or the model of the adapter are absent

## v0.1 - 2024-06-09

First release 🎉
