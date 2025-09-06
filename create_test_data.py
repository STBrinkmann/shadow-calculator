#!/usr/bin/env python3
"""
Create test DTM and DSM raster files for the Shadow Calculator
Requires: GDAL Python bindings (pip install gdal or conda install gdal)
"""

import numpy as np
from osgeo import gdal, osr
import os
import sys

def create_test_rasters(output_dir="test_data"):
    """Create test DTM and DSM files with realistic terrain and buildings"""
    
    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    
    # Raster dimensions and resolution
    width, height = 200, 200  # 200x200 pixels
    pixel_size = 0.5  # 0.5 meters per pixel
    
    # Create coordinate system (Swiss CH1903+ / LV95)
    srs = osr.SpatialReference()
    srs.ImportFromEPSG(2056)  # Swiss coordinate system
    # Alternative: Use WGS84 UTM Zone 32N
    # srs.ImportFromEPSG(32632)
    
    # Set origin (example coordinates for Zurich area)
    origin_x = 2683000  # Swiss coordinates
    origin_y = 1248000
    
    # Create geotransform
    geotransform = [
        origin_x,        # top left x
        pixel_size,      # pixel width
        0,               # rotation (0 for north-up)
        origin_y,        # top left y
        0,               # rotation (0 for north-up)
        -pixel_size      # pixel height (negative for north-up)
    ]
    
    print("Creating test rasters...")
    print(f"  Dimensions: {width}x{height} pixels")
    print(f"  Resolution: {pixel_size}m per pixel")
    print(f"  Area covered: {width*pixel_size}m x {height*pixel_size}m = {(width*pixel_size/100):.1f} hectares")
    
    # Create DTM (Digital Terrain Model) - ground elevation
    print("\n1. Creating DTM (terrain)...")
    
    # Generate realistic terrain with some hills
    x = np.linspace(0, 10, width)
    y = np.linspace(0, 10, height)
    X, Y = np.meshgrid(x, y)
    
    # Base elevation (100m) with gentle hills
    dtm_data = 100 + \
               5 * np.sin(X/3) * np.cos(Y/3) + \
               3 * np.sin(X/5) + \
               2 * np.cos(Y/4) + \
               np.random.normal(0, 0.1, (height, width))  # Small random variations
    
    # Save DTM
    dtm_path = os.path.join(output_dir, "test_dtm.tif")
    driver = gdal.GetDriverByName('GTiff')
    dtm_dataset = driver.Create(
        dtm_path,
        width,
        height,
        1,  # number of bands
        gdal.GDT_Float32
    )
    
    dtm_dataset.SetGeoTransform(geotransform)
    dtm_dataset.SetProjection(srs.ExportToWkt())
    
    dtm_band = dtm_dataset.GetRasterBand(1)
    dtm_band.WriteArray(dtm_data)
    dtm_band.SetNoDataValue(-9999)
    dtm_band.ComputeStatistics(False)
    
    dtm_dataset = None  # Close dataset
    print(f"  ✓ Created: {dtm_path}")
    
    # Create DSM (Digital Surface Model) - includes buildings and trees
    print("\n2. Creating DSM (surface with buildings)...")
    
    dsm_data = dtm_data.copy()
    
    # Add some buildings (rectangular structures)
    buildings = [
        # (row_start, row_end, col_start, col_end, height)
        (40, 60, 40, 60, 15),    # Central tall building (15m)
        (80, 100, 50, 70, 10),   # Office building (10m)
        (120, 140, 80, 100, 12), # Another tall building (12m)
        (30, 45, 100, 115, 8),   # Residential building (8m)
        (150, 170, 150, 170, 20), # Tower (20m)
    ]
    
    for r1, r2, c1, c2, h in buildings:
        dsm_data[r1:r2, c1:c2] += h
    
    # Add some trees (smaller circular features)
    tree_positions = [
        (70, 30, 5),   # (row, col, height)
        (90, 120, 6),
        (110, 40, 5),
        (140, 130, 7),
        (60, 150, 6),
        (100, 160, 5),
    ]
    
    for row, col, h in tree_positions:
        # Create circular tree crown
        for i in range(max(0, row-3), min(height, row+4)):
            for j in range(max(0, col-3), min(width, col+4)):
                dist = np.sqrt((i-row)**2 + (j-col)**2)
                if dist < 3:
                    dsm_data[i, j] += h * np.exp(-dist)
    
    # Save DSM
    dsm_path = os.path.join(output_dir, "test_dsm.tif")
    dsm_dataset = driver.Create(
        dsm_path,
        width,
        height,
        1,
        gdal.GDT_Float32
    )
    
    dsm_dataset.SetGeoTransform(geotransform)
    dsm_dataset.SetProjection(srs.ExportToWkt())
    
    dsm_band = dsm_dataset.GetRasterBand(1)
    dsm_band.WriteArray(dsm_data)
    dsm_band.SetNoDataValue(-9999)
    dsm_band.ComputeStatistics(False)
    
    dsm_dataset = None  # Close dataset
    print(f"  ✓ Created: {dsm_path}")
    
    # Calculate and display statistics
    print("\n3. Raster Statistics:")
    print(f"  DTM elevation range: {dtm_data.min():.1f}m - {dtm_data.max():.1f}m")
    print(f"  DSM elevation range: {dsm_data.min():.1f}m - {dsm_data.max():.1f}m")
    
    height_diff = dsm_data - dtm_data
    print(f"  Maximum object height: {height_diff.max():.1f}m")
    print(f"  Objects present: {np.sum(height_diff > 0.5)} pixels")
    
    print("\n✅ Test data created successfully!")
    print("\nYou can now:")
    print("1. Start the application: npm run tauri dev")
    print("2. Load these files:")
    print(f"   - DTM: {os.path.abspath(dtm_path)}")
    print(f"   - DSM: {os.path.abspath(dsm_path)}")
    print("3. Draw a polygon on the map to define your AOI")
    print("4. Click 'Calculate Shadows' to run the analysis")
    
    return dtm_path, dsm_path

if __name__ == "__main__":
    try:
        create_test_rasters()
    except ImportError:
        print("Error: GDAL Python bindings not found!")
        print("\nPlease install GDAL Python bindings:")
        print("  pip install gdal")
        print("  or")
        print("  conda install -c conda-forge gdal")
        sys.exit(1)
    except Exception as e:
        print(f"Error creating test data: {e}")
        sys.exit(1)