# 🌞 Shadow Calculator

A high-performance desktop application for calculating and visualizing shadows cast by terrain and objects (buildings, trees) using Digital Terrain Models (DTM) and Digital Surface Models (DSM).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Node](https://img.shields.io/badge/node-16%2B-green.svg)

## ✨ Features

- **High-Performance Shadow Calculation**: Rust-based engine with parallel processing
- **Continuous Shadow Values**: Sub-pixel accuracy with configurable quality levels
- **Pre-computed Sun Positions**: Optimized with angle rounding for faster calculations  
- **Interactive Map Interface**: Draw areas of interest directly on satellite imagery
- **Time Series Analysis**: Animate shadows throughout the day/year
- **Multiple Export Formats**: GeoTIFF with summary layers, CSV for analysis
- **Real-time Visualization**: See shadows overlaid on the map

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70 or higher ([Install Rust](https://rustup.rs/))
- **Node.js**: 16.x or higher ([Install Node](https://nodejs.org/))
- **GDAL**: 3.0 or higher (see installation below)

### System Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    gdal-bin libgdal-dev \
    build-essential pkg-config libssl-dev \
    libwebkit2gtk-4.1-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev
```

#### macOS
```bash
brew install gdal pkg-config
```

#### Windows
Use WSL2 with Ubuntu, or install GDAL from [OSGeo4W](https://trac.osgeo.org/osgeo4w/)

### Installation

1. **Clone the repository** (or create new project)
```bash
mkdir shadow-calculator
cd shadow-calculator
```

2. **Run the automated setup**
```bash
chmod +x setup.sh
./setup.sh
```

Or manually:
```bash
# Install Node dependencies
npm install

# Build Rust backend
cd src-tauri
cargo build --release
cd ..
```

3. **Configure Mapbox**
   - Get a free token at [mapbox.com](https://www.mapbox.com/)
   - Add to `.env.local`:
```bash
VITE_MAPBOX_TOKEN=your_token_here
```

4. **Create test data** (optional)
```bash
python3 create_test_data.py
```

5. **Run the application**
```bash
npm run tauri dev
```

## 📖 Usage Guide

### 1. Load Raster Data
- Click "Select DTM file" to load your Digital Terrain Model
- Click "Select DSM file" to load your Digital Surface Model
- Both files must have the same resolution (e.g., 0.5m)

### 2. Define Area of Interest
- Use the polygon tool on the map to draw your area
- The tool is in the top-left corner of the map
- Draw clockwise for best results

### 3. Configure Parameters

#### Time Settings
- **Date Range**: Select start and end dates for analysis
- **Interval**: Choose time step (30 min to 24 hours)

#### Advanced Settings
- **Buffer**: Extra area around AOI to catch shadows from tall objects (default: 50m)
- **Angle Precision**: Round sun angles for caching (0.01° to 1.0°)
- **Shadow Quality**:
  - Fast: Binary shadows only
  - Normal: 2×2 sub-sampling at edges
  - High: 4×4 sub-sampling at edges
  - Scientific: 8×8 full sub-sampling

### 4. Calculate Shadows
Click "Calculate Shadows" to start processing. Progress will be shown.

### 5. Visualize Results
- Use playback controls to animate through time
- Adjust playback speed (0.5x to 5x)
- View statistics in real-time

### 6. Export Results

#### GeoTIFF Format
- Layer 0: Total shadow hours
- Layer 1: Average shadow percentage
- Layer 2: Maximum consecutive shadow hours
- Layer 3: Morning shadow hours (before noon)
- Layer 4: Afternoon shadow hours (after noon)
- Layers 5+: Individual timestamps

#### CSV Format
Long format with columns:
- cell_id, lat, lon, datetime, shadow_fraction

## 🏗️ Project Structure

```
shadow-calculator/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri application entry
│   │   ├── types.rs         # Type definitions
│   │   ├── shadow_engine.rs # Core shadow calculations
│   │   ├── sun_position.rs  # Solar position calculator
│   │   └── raster_io.rs     # GDAL raster I/O
│   └── Cargo.toml
├── src/                 # React frontend
│   ├── components/
│   │   ├── MapView.tsx      # Mapbox map interface
│   │   ├── TimeControls.tsx # Date/time configuration
│   │   ├── FileUpload.tsx   # Raster file selection
│   │   └── ResultsPanel.tsx # Results visualization
│   ├── App.tsx
│   └── main.tsx
├── test_data/           # Test raster files
├── package.json
└── README.md
```

## 🔧 Development

### Frontend Development
```bash
# Hot-reload for UI changes
npm run dev

# Type checking
npm run type-check

# Build for production
npm run build
```

### Backend Development
```bash
cd src-tauri

# Check compilation
cargo check

# Run tests
cargo test

# Build optimized
cargo build --release
```

### Creating a Release Build
```bash
npm run tauri build
```

The installer will be in `src-tauri/target/release/bundle/`

## 🧮 Algorithm Details

### Shadow Calculation
1. **Ray Marching**: Cast rays from each cell toward the sun
2. **Terrain Intersection**: Check if terrain blocks the ray
3. **Edge Refinement**: Sub-sample cells at shadow boundaries
4. **Parallel Processing**: Use all CPU cores via Rayon

### Sun Position
- Julian day calculation for accurate solar positioning
- Equation of time correction
- Solar declination based on date
- Local solar time adjustment

### Performance Optimizations
- Pre-computed sun positions with caching
- Angle rounding to reduce unique calculations
- Parallel timestamp processing
- Bilinear interpolation for smooth terrain sampling

## 📊 Performance

For a 1 hectare area at 0.5m resolution:
- **Pixels**: 200×200 = 40,000 cells
- **Time points**: ~5,800 (8 months hourly)
- **Memory**: ~900 MB for full results
- **Processing time**: 2-10 minutes depending on quality

## 🐛 Troubleshooting

### GDAL Installation Issues
```bash
# Verify GDAL is installed
gdal-config --version

# Set environment variables if needed
export GDAL_DATA=/usr/share/gdal
```

### Mapbox Token Issues
- Ensure token is in `.env.local`
- Restart the app after adding token
- Check console for error messages

### Memory Issues
For large areas, consider:
- Reducing temporal resolution
- Processing in chunks
- Using "Fast" quality mode initially

### Build Errors
```bash
# Clear caches and rebuild
rm -rf node_modules src-tauri/target
npm install
cd src-tauri && cargo clean && cargo build
```

## 📚 Data Requirements

### Input Rasters
- **Format**: GeoTIFF (`.tif`, `.tiff`)
- **Resolution**: 0.5m recommended, must match between DTM/DSM
- **Projection**: Any supported by GDAL
- **Size**: Up to 1000×1000 pixels performs well

### Coordinate Systems
Tested with:
- Swiss CH1903+ / LV95 (EPSG:2056)
- WGS84 / UTM zones (EPSG:326xx)
- Local projected systems

## 🤝 Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop app framework
- [Mapbox](https://www.mapbox.com/) - Map visualization
- [GDAL](https://gdal.org/) - Geospatial data processing
- [Rust](https://www.rust-lang.org/) - Systems programming
- [React](https://react.dev/) - User interface

## 📮 Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Check existing issues first
- Provide sample data if reporting bugs

---

Built with ❤️ for accurate shadow analysis in landscape planning, solar installations, and urban development.