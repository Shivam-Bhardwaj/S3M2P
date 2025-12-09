# Sensors Tool

A WebAssembly tool for accessing device sensors (Accelerometer, Gyroscope) and Camera via standard Web APIs.

## Overview
Demonstrates how to access:
- **DeviceMotion**: Accelerometer data.
- **DeviceOrientation**: Gyroscope/Compass data.
- **UserMedia**: Camera stream.

## Features
- **Real-time Graphing**: Visualizes accelerometer data (X, Y, Z) on an HTML Canvas.
- **Permission Handling**: Helper logic for iOS 13+ permission requests.
- **Camera Stream**: Displays live camera feed.

## Dependencies
- `web-sys`: For `DeviceMotionEvent`, `MediaStream`, etc.
- `wasm-bindgen`
