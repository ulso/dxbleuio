# Example Dioxus/Rust application monitoring sensor devices using the [BleuIO](https://bleuio.com) dongle

This is my first try to develop a desktop application in Rust using the [Dioxus](https://dioxuslabs.com) framework.

The kinds of sensor that could be monitored by this application are the ones from [Smart Sensor Devices AB](https://smartsensordevices.com), specifically the [HibouAIR](https://smartsensordevices.com/our-products-and-solutions/) sensors.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app 
│  ├─ bleuio.rs # bleuio.rs contains some enums and utility functions for BleuIO access
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

Run the following command in the root of your project to start developing with the default platform:

```bash
dx run
```
