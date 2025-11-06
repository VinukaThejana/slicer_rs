# slicer_rs

A Rust-based 3D model slicer API designed for instant quotation of 3D printing costs.

## Overview

slicer_rs is a high-performance API that analyzes 3D models to calculate the estimated cost of 3D printing. It was developed to power the instant quotation system for [polyvoxel.com](https://polyvoxel.com), enabling users to upload their 3D models and receive immediate pricing information.

> **Note**: This repository is shared for knowledge purposes only. The API is for internal use within polyvoxel.com.

## Features

- **STL File Support**: Currently processes STL files to extract printing metrics

- **Cost Calculation**: Analyzes models to determine printing costs based on various parameters

- **Performance Focused**: Built in Rust for optimal speed and reliability

## Project Roadmap

- [x] STL file format support
- [ ] OBJ file format support
- [ ] Additional 3D file formats
- [x] Basic cost calculation with signed volumes and material density
- [ ] Automatic model repair for zero faces
- [ ] Automatic model repair for watertight issues
- [ ] Enhanced cost breakdown with material-specific calculations
- [ ] Multi-material printing cost estimation (with MTL files)

## Current Limitations

- Only processes STL files
- Does not attempt to repair models with zero faces or watertight issues
- Limited to specific printer parameters (will be configurable in future versions)

## Contributing

This repository is primarily shared for educational purposes. If you have suggestions or improvements, please feel free to reach me on my [email](mailto:vinuka.t@icloud.com), or open an issue here.
