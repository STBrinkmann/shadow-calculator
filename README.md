# 🌞 Shadow Calculator

> *"Because apparently, sticking a stick in the ground and checking the shadow wasn't complicated enough."*

A desktop application that takes the ancient art of shadow-telling and completely overengineers it with Rust, mathematics, and an unhealthy amount of precision. Calculate and visualize shadows from terrain and buildings using Digital Terrain Models (DTM) and Digital Surface Models (DSM). 

Perfect for vampire real estate agents, urban planners who've given up on sundials, and anyone who's ever wondered "but what if I need to know the *exact* shadow at 2:37 PM on a Tuesday six months from now?"

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-red.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-1.5-blue.svg)](https://tauri.app/)

## ✨ What This Thing Actually Does

- 🔥 **Blazingly Fast™** shadow calculations (it's Rust, we're legally required to say "blazingly fast")
- 🎯 **Sub-pixel precision** because regular pixel precision is for peasants  
- 🗺️ **Interactive mapping** - draw areas and watch shadows dance across time
- 📊 **Time-series analysis** - animate shadows like it's a Netflix show
- 🎨 **Pretty visualizations** with hover tooltips that show you exactly how doomed your picnic spot is
- 📤 **Export everything** as GeoTIFF or CSV, because data wants to be free
- 🏃‍♂️ **Real-time progress** so you know the computer is actually working and not just having an existential crisis

## 🚀 Getting Started (AKA "The Ritual")

### What You'll Need

Before we can calculate shadows with the precision of a Swiss watch, you'll need:

- **Rust** (1.70+) - [Get it here](https://rustup.rs/) because memory safety is sexy
- **Node.js** (16+) - [Download](https://nodejs.org/) for the frontend magic
- **GDAL** (3.0+) - The geospatial Swiss Army knife (installation below)

### Installing GDAL (The Fun Part™)

**Ubuntu/Debian** (Easy Mode):
```bash
sudo apt-get update
sudo apt-get install -y gdal-bin libgdal-dev \
    build-essential pkg-config libssl-dev \
    libwebkit2gtk-4.1-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev
```

**macOS** (Homebrew to the Rescue):
```bash
brew install gdal pkg-config
```

**Windows** (Choose Your Pain):
- Option A: Use WSL2 with Ubuntu (recommended for sanity)
- Option B: Install from [OSGeo4W](https://trac.osgeo.org/osgeo4w/) and question your life choices

### Installation

1. **Clone this masterpiece**:
```bash
git clone https://github.com/your-username/shadow-calculator.git
cd shadow-calculator
```

2. **Install the dependencies**:
```bash
npm install
```

3. **Build the Rust backend** (grab a coffee, it's compiling):
```bash
cd src-tauri
cargo build --release
cd ..
```

4. **Launch the app**:
```bash
npm run tauri dev
```

And voilà! You now have a shadow calculator that would make ancient sundial makers weep with either joy or existential dread.

## 📖 How to Use This Beautiful Monster

### Step 1: Feed It Data
Load your DTM (terrain) and DSM (terrain + buildings/trees) files. Both should be GeoTIFF format and match in resolution. Think of DTM as "the world if it were really flat and boring" and DSM as "the world with all the fun stuff that casts shadows."

### Step 2: Draw Your Area of Interest
Use the polygon tool to draw where you want shadows calculated. Pro tip: drawing clockwise is preferred, but the app won't judge you if you go counterclockwise (much).

### Step 3: Configure the Madness
- **Time Range**: When do you want shadows? Next Tuesday? The entire summer of 2025?
- **Interval**: How often? Every 30 minutes if you're obsessive, every few hours if you value your sanity
- **Buffer**: Extra area to catch shadows from tall stuff nearby (default: 50m, or "about half a football field")
- **Shadow Quality**: From "Fast" (good enough) to "Scientific" (pixel-perfect paranoia)

### Step 4: Click "Calculate Shadows" 
Then sit back and watch the progress bar. The app will:
1. Calculate sun positions (using actual astronomy, not guesswork)
2. Cast virtual rays from every pixel toward the sun
3. Check if terrain blocks each ray (the computationally expensive part)
4. Apply fancy edge smoothing (because sharp shadows are so last century)

### Step 5: Marvel at Your Results
- **Hover over the map** to see exact shadow percentages
- **Click anywhere** for detailed shadow statistics in a beautiful popup
- **Use time controls** to animate through your data like a shadow time-lord
- **Export results** in multiple formats for further analysis or bragging rights

## 🎯 Shadow Quality Modes

- **Fast**: Binary shadows only. "Is it shadow? Yes/No." Simple and speedy.
- **Normal**: 2×2 sub-sampling at edges. Good balance of quality and performance.
- **High**: 4×4 sub-sampling at edges. For when "good enough" isn't good enough.
- **Scientific**: 8×8 full sub-sampling. For when you need PhD-level shadow precision.

## 🔧 The Technical Wizardry Behind the Curtain

### Shadow Algorithm
1. **Ray Marching**: Cast rays from each pixel toward the sun (like reverse photon tracing)
2. **Terrain Intersection**: Check if the ray hits any terrain on its way to the sun
3. **Edge Refinement**: Sub-sample pixels at shadow boundaries for smooth edges
4. **Parallel Processing**: Uses all your CPU cores because why have them idle?

### Sun Position Calculations
- Julian day calculations for astronomical accuracy
- Equation of time corrections (because Earth's orbit isn't a perfect circle)
- Solar declination based on date (fancy astronomy math)
- Local solar time adjustments (timezone handling that actually works)

### Performance Optimizations
- Pre-computed sun positions with intelligent caching
- Angle rounding to reduce redundant calculations
- Parallel processing across time points
- Memory-efficient streaming for large datasets

## 📊 Performance Expectations

For a typical 1 hectare area at 0.5m resolution:
- **40,000 pixels** to process
- **~5,800 time points** (8 months, hourly)
- **~900 MB memory** for full results
- **2-10 minutes** processing time (depending on quality settings and how patient you are)

## 🐛 When Things Go Wrong

### "GDAL Not Found" Errors
```bash
# Check if GDAL is properly installed
gdal-config --version

# If that fails, GDAL isn't installed or isn't in your PATH
# Go back to the installation section and try again
```

### Memory Issues
Your computer is trying to tell you something. Consider:
- Reducing the time range
- Using a coarser time interval
- Switching to "Fast" quality mode
- Processing smaller areas
- Downloading more RAM (just kidding, that doesn't work)

### Build Errors
When all else fails, the nuclear option:
```bash
# Clear everything and start fresh
rm -rf node_modules src-tauri/target
npm install
cd src-tauri && cargo clean && cargo build --release
```

## 📁 Project Structure

```
shadow-calculator/
├── src-tauri/                # The Rust backend (where the magic happens)
│   ├── src/
│   │   ├── main.rs          # Tauri app entry point
│   │   ├── shadow_engine.rs # Core shadow calculation engine
│   │   ├── sun_position.rs  # Solar position calculations
│   │   └── raster_io.rs     # GDAL file I/O operations
├── src/                     # React frontend (the pretty face)
│   ├── components/
│   │   ├── LeafletMapView.tsx    # Interactive map with all the bells and whistles
│   │   ├── TimeControls.tsx      # Date/time configuration
│   │   ├── FileUpload.tsx        # File selection interface
│   │   ├── ResultsPanel.tsx      # Results visualization
│   │   └── ProgressModal.tsx     # Progress display (so you know it's working)
│   ├── App.tsx              # Main application component
│   └── types.ts             # TypeScript definitions
└── test_data/               # Sample data for testing
```

## 🤝 Contributing

Found a bug? Have an idea? Want to make shadows even more complicated? Contributions are welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/even-better-shadows`)
3. Make your changes
4. Add tests if you're feeling responsible
5. Submit a pull request

Please check the [Contributing Guidelines](CONTRIBUTING.md) for more details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 

TL;DR: You can use this code for whatever you want, just don't blame us if your shadow calculations achieve sentience and start demanding workers' rights.

## 🙏 Standing on the Shoulders of Giants

This project wouldn't exist without these amazing technologies:

- **[Tauri](https://tauri.app/)** - For making desktop apps that don't require a PhD in C++
- **[Rust](https://www.rust-lang.org/)** - For memory safety and that sweet, sweet performance
- **[Leaflet](https://leafletjs.com/)** - For maps that just work without requiring a mortgage
- **[React](https://react.dev/)** - For making UIs less painful than they used to be
- **[GDAL](https://gdal.org/)** - The Swiss Army knife of geospatial data
- **[OpenStreetMap](https://www.openstreetmap.org/)** - For free map tiles because we're not made of money

## 💡 Use Cases (Why Would Anyone Want This?)

- **Solar panel planning** - Find the best spots that aren't in shadow all day
- **Architecture and urban planning** - Ensure your building doesn't accidentally create a perpetual winter
- **Agriculture** - Optimize crop placement because plants are picky about sunlight
- **Photography** - Plan that perfect golden hour shot
- **Vampire real estate** - Find properties with maximum shadow coverage
- **Academic research** - When you need to publish something about shadows
- **Because you can** - The most honest reason of all

## 🆘 Support & Questions

Having trouble? Don't suffer in silence:

- 📋 [Open an issue](https://github.com/your-username/shadow-calculator/issues) on GitHub
- 🔍 Check existing issues first (someone might have had the same problem)
- 📊 Include sample data if you're reporting a bug
- 🌍 Be descriptive about your environment and what you were trying to do

---

<div align="center">

**Built with ☕ and mild obsession for accurate shadow analysis**

*"Making simple things complicated since 2024"*

</div>