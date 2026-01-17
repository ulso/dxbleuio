# Example Dioxus/Rust application monitoring sensor devices using the [BleuIO](https://bleuio.com) dongle

This is my first try to develop a desktop application in Rust using the [Dioxus](https://dioxuslabs.com) framework.

The kinds of sensor that could be monitored by this application are the ones from [Smart Sensor Devices AB](https://smartsensordevices.com), specifically the [HibouAIR](https://smartsensordevices.com/our-products-and-solutions/) sensors.

## How it works
1. The app starts by trying to find a USB device with the Vendor ID and Product ID of the BlueIO dongle.
2. If valid device found, it tries to open the corresponding USB Serial port.
3. If open succeeded, turns echo off with the 'ATE0' command.
4. Enables verbose mode with the 'ATV1' command.
5. Starts scanning for sensor advertisment with the 'AT+FINDSCANDATA=FF5B07' command.

## Screenshot
![Screenshot](/img/SCR-20260117-jaec.png)

The application has only been tested on macos so far but should at least work on Linux as well.
