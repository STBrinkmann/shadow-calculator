# Shadow Calculator

A high-performance desktop application for calculating and visualizing shadows cast by terrain and objects using Digital Terrain Models (DTM) and Digital Surface Models (DSM). Built with Tauri (Rust + React), it provides accurate shadow analysis with interactive visualization and export capabilities.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-red.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-1.5-blue.svg)](https://tauri.app/)

## Features

- **High-performance shadow calculations** using Rust for optimal speed and memory safety
- **Sub-pixel precision** with configurable quality levels from fast to scientific accuracy
- **Interactive mapping** with area-of-interest selection and real-time visualization
- **Time-series analysis** with animation controls for temporal shadow data
- **Detailed visualizations** including hover tooltips and statistical popups
- **Multiple export formats** supporting GeoTIFF and CSV output
- **Real-time progress tracking** during computation

## Getting Started

### Prerequisites

The following software is required:

- **Rust** (1.70+) - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** (16+) - [Download from nodejs.org](https://nodejs.org/)
- **GDAL** (3.0+) - Geospatial Data Abstraction Library for raster processing

### Installing GDAL

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install -y gdal-bin libgdal-dev \
    build-essential pkg-config libssl-dev \
    libwebkit2gtk-4.1-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev \
    libsoup2.4-dev libjavascriptcoregtk-4.0-dev
```

**macOS**:
```bash
brew install gdal pkg-config
```

**Windows**:
- Use WSL2 with Ubuntu (recommended)
- Or install from [OSGeo4W](https://trac.osgeo.org/osgeo4w/)

### Installation

1. **Clone the repository**:
```bash
git clone https://github.com/STBrinkmann/shadow-calculator.git
cd shadow-calculator
```

2. **Install dependencies**:
```bash
npm install
```

3. **Build the Rust backend**:
```bash
cd src-tauri
cargo build --release
cd ..
```

4. **Launch the application**:
```bash
npm run tauri dev
```

## Usage Guide

### Step 1: Load Raster Data
Load your DTM (terrain) and DSM (terrain + buildings/trees) files. Both should be in GeoTIFF format with matching resolution and spatial coverage.

### Step 2: Define Area of Interest
Use the polygon tool to draw the area where you want shadows calculated. The polygon should be drawn clockwise for optimal results.

### Step 3: Configure Parameters
- **Time Range**: Specify start and end dates for shadow analysis
- **Interval**: Set time intervals (e.g., 30 minutes, 1 hour)
- **Buffer**: Extra area around polygon to capture shadows from nearby objects (default: 50m)
- **Shadow Quality**: Choose from Fast, Normal, High, or Scientific quality levels

### Step 4: Calculate Shadows
Start the calculation process. The application will:
1. Calculate precise sun positions using astronomical algorithms
2. Cast rays from each pixel toward the sun
3. Test for terrain intersection along each ray path
4. Apply sub-pixel refinement for edge smoothing

### Step 5: Analyze Results
- **Hover over the map** for instant shadow percentage data
- **Click locations** for detailed statistical popups
- **Use time controls** to animate through temporal data
- **Export results** in GeoTIFF or CSV format

## Shadow Quality Modes

- **Fast**: Binary shadow detection for maximum speed
- **Normal**: 2×2 sub-sampling at shadow edges for balanced quality and performance
- **High**: 4×4 sub-sampling at edges for improved accuracy
- **Scientific**: 8×8 full sub-sampling for research-grade precision

## Technical Implementation

### Shadow Algorithm
1. **Ray Marching**: Cast rays from each pixel toward the sun position
2. **Terrain Intersection**: Test for terrain obstruction along the ray path
3. **Edge Refinement**: Sub-sample pixels at shadow boundaries for smooth transitions
4. **Parallel Processing**: Multi-threaded computation utilizing all available CPU cores

### Sun Position Calculations
- Julian day calculations for astronomical precision
- Equation of time corrections for orbital variations
- Solar declination computations based on date
- Local solar time adjustments with timezone support

### Performance Optimizations
- Pre-computed sun positions with angle-based caching
- Intelligent angle rounding to reduce redundant calculations
- Parallel processing across temporal data points
- Memory-efficient data streaming for large datasets

## Performance Expectations

For a typical 1 hectare area at 0.5m resolution:
- **40,000 pixels** to process
- **~5,800 time points** (8 months, hourly intervals)
- **~900 MB memory** for complete dataset
- **Processing time**: 2-10 minutes depending on quality settings and hardware configuration

## Troubleshooting

### GDAL Installation Issues
```bash
# Check if GDAL is properly installed
gdal-config --version

# If command not found, GDAL isn't installed or not in PATH
# Refer to the installation section above
```

### Memory Limitations
If experiencing memory issues, consider:
- Reducing the time range or area of interest
- Using coarser time intervals
- Switching to "Fast" quality mode
- Processing data in smaller chunks

### Build Errors
To resolve build issues:
```bash
# Clean and rebuild
rm -rf node_modules src-tauri/target
npm install
cd src-tauri && cargo clean && cargo build --release
```

## 📁 Project Structure

```
shadow-calculator/
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri app entry point
│   │   ├── shadow_engine.rs # Core shadow calculation engine
│   │   ├── sun_position.rs  # Solar position calculations
│   │   └── raster_io.rs     # GDAL file I/O operations
├── src/                     # React frontend
│   ├── components/
│   │   ├── LeafletMapView.tsx    # Interactive map interface
│   │   ├── TimeControls.tsx      # Date/time configuration
│   │   ├── FileUpload.tsx        # File selection interface
│   │   ├── ResultsPanel.tsx      # Results visualization
│   │   └── ProgressModal.tsx     # Progress display
│   ├── App.tsx              # Main application component
│   └── types.ts             # TypeScript definitions
└── test_data/               # Sample data for testing
```

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes
4. Add tests where appropriate
5. Submit a pull request

Please check the [Contributing Guidelines](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project is built with:

- **[Tauri](https://tauri.app/)** - Cross-platform desktop application framework
- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Leaflet](https://leafletjs.com/)** - Interactive mapping library
- **[React](https://react.dev/)** - Frontend JavaScript library
- **[GDAL](https://gdal.org/)** - Geospatial Data Abstraction Library
- **[OpenStreetMap](https://www.openstreetmap.org/)** - Open-source map data

## Use Cases

- **Solar panel planning** - Optimize placement for maximum sunlight exposure
- **Architecture and urban planning** - Assess shadow impacts of new developments
- **Agriculture** - Plan crop placement based on sunlight availability
- **Photography** - Schedule shoots for optimal lighting conditions
- **Environmental studies** - Analyze microclimate effects of shadow patterns
- **Academic research** - Support studies requiring precise shadow analysis

## Support

For questions or issues:

- [Open an issue](https://github.com/STBrinkmann/shadow-calculator/issues) on GitHub
- Check existing issues first for similar problems
- Include sample data and error messages when reporting bugs
- Describe your environment and steps to reproduce the issue