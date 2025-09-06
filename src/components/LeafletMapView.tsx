import React, { useEffect, useRef, useState } from 'react';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import 'leaflet-draw/dist/leaflet.draw.css';
import 'leaflet-draw';

// Fix Leaflet icon issue
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
  iconUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png',
  iconRetinaUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png',
  shadowUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png',
});

interface RasterBounds {
  min_lon: number;
  max_lon: number;
  min_lat: number;
  max_lat: number;
}

interface MapViewProps {
  onAOIDrawn: (coordinates: number[][]) => void;
  shadowData: number[][];
  hasResults: boolean;
  rasterBounds?: RasterBounds | null;
}

const LeafletMapView: React.FC<MapViewProps> = ({ 
  onAOIDrawn, 
  shadowData, 
  hasResults,
  rasterBounds 
}) => {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<L.Map | null>(null);
  const drawnItems = useRef<L.FeatureGroup | null>(null);
  const shadowLayer = useRef<L.ImageOverlay | null>(null);
  const rasterBoundsRect = useRef<L.Rectangle | null>(null);

  useEffect(() => {
    if (!mapContainer.current || map.current) return;

    // Initialize map with OpenStreetMap tiles (no API key needed!)
    map.current = L.map(mapContainer.current).setView([47.3769, 8.5417], 13); // Default view

    // Add OpenStreetMap tile layer
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap contributors',
      maxZoom: 19,
    }).addTo(map.current);

    // Add a satellite layer option
    const satelliteLayer = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', {
      attribution: 'Tiles © Esri',
      maxZoom: 19,
    });

    // Add layer control
    L.control.layers({
      'OpenStreetMap': L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png'),
      'Satellite': satelliteLayer
    }).addTo(map.current);

    // Initialize the FeatureGroup to store editable layers
    drawnItems.current = new L.FeatureGroup();
    map.current.addLayer(drawnItems.current);

    // Initialize the draw control and pass it the FeatureGroup of editable layers
    const drawControl = new (L.Control as any).Draw({
      edit: {
        featureGroup: drawnItems.current,
      },
      draw: {
        polygon: {
          shapeOptions: {
            color: '#3b82f6',
            fillColor: '#3b82f6',
            fillOpacity: 0.2,
          },
        },
        polyline: false,
        rectangle: false,
        circle: false,
        marker: false,
        circlemarker: false,
      },
    });
    map.current.addControl(drawControl);

    // Handle polygon creation
    map.current.on(L.Draw.Event.CREATED, (e: any) => {
      const layer = e.layer;
      
      // Clear previous drawings
      drawnItems.current?.clearLayers();
      
      // Add new polygon
      drawnItems.current?.addLayer(layer);
      
      // Get coordinates and notify parent
      const latlngs = layer.getLatLngs()[0];
      const coordinates = latlngs.map((latlng: L.LatLng) => [latlng.lng, latlng.lat]);
      onAOIDrawn(coordinates);
    });

    // Handle polygon deletion
    map.current.on(L.Draw.Event.DELETED, () => {
      onAOIDrawn([]);
    });

    // Add instructions
    const info = L.control({ position: 'topright' });
    info.onAdd = function () {
      const div = L.DomUtil.create('div', 'info');
      div.style.backgroundColor = 'white';
      div.style.padding = '10px';
      div.style.borderRadius = '5px';
      div.style.boxShadow = '0 0 15px rgba(0,0,0,0.2)';
      div.style.maxWidth = '250px';
      div.innerHTML = `
        <h4 style="margin: 0 0 5px 0; color: black;">Instructions</h4>
        <ol style="margin: 0; padding-left: 20px; font-size: 12px; color: black;">
          <li>Load DTM and DSM files</li>
          <li>Map will zoom to raster extent</li>
          <li>Click polygon tool (left side)</li>
          <li>Draw your Area of Interest</li>
          <li>Click first point to complete</li>
          <li>Click "Calculate Shadows"</li>
        </ol>
      `;
      return div;
    };
    info.addTo(map.current);

    return () => {
      map.current?.remove();
      map.current = null;
    };
  }, [onAOIDrawn]);

  // Handle raster bounds - zoom to extent when rasters are loaded
  useEffect(() => {
    if (!map.current || !rasterBounds) return;

    // Remove existing bounds rectangle if any
    if (rasterBoundsRect.current) {
      map.current.removeLayer(rasterBoundsRect.current);
    }

    // Create bounds
    const bounds = L.latLngBounds(
      [rasterBounds.min_lat, rasterBounds.min_lon],
      [rasterBounds.max_lat, rasterBounds.max_lon]
    );

    // Draw a rectangle showing raster extent
    rasterBoundsRect.current = L.rectangle(bounds, {
      color: '#ffcc00',
      weight: 2,
      fillOpacity: 0.05,
      dashArray: '5, 10',
      interactive: false
    }).addTo(map.current);

    // Add a popup to show it's the raster extent
    rasterBoundsRect.current.bindPopup('Raster Extent');

    // Zoom to bounds with some padding
    map.current.fitBounds(bounds, {
      padding: [50, 50],
      maxZoom: 16
    });

    console.log('Zoomed to raster bounds:', bounds);
  }, [rasterBounds]);

  // Handle shadow data visualization
  useEffect(() => {
    if (!map.current || !hasResults || !shadowData.length || !drawnItems.current) return;

    // Remove existing shadow layer
    if (shadowLayer.current) {
      map.current.removeLayer(shadowLayer.current);
      shadowLayer.current = null;
    }

    // Get bounds from drawn AOI
    const layers = drawnItems.current.getLayers();
    if (layers.length === 0) return;

    const bounds = drawnItems.current.getBounds();
    
    // Create canvas for shadow visualization
    const canvas = document.createElement('canvas');
    const height = shadowData.length;
    const width = shadowData[0]?.length || 0;
    
    if (width === 0 || height === 0) return;

    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;

    // Draw shadow data with gradient colors
    const imageData = ctx.createImageData(width, height);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const idx = (y * width + x) * 4;
        const shadowValue = shadowData[y][x];
        
        // Create gradient from blue (no shadow) to dark purple (full shadow)
        const intensity = 1 - shadowValue;
        
        // Color scheme: light areas = yellow/white, shadows = blue/purple
        if (shadowValue < 0.1) {
          // No shadow - bright
          imageData.data[idx] = 255;     // R
          imageData.data[idx + 1] = 250; // G
          imageData.data[idx + 2] = 200; // B
        } else if (shadowValue < 0.5) {
          // Partial shadow - blue
          imageData.data[idx] = 100 * intensity;     // R
          imageData.data[idx + 1] = 150 * intensity; // G
          imageData.data[idx + 2] = 200;             // B
        } else {
          // Full shadow - dark purple
          imageData.data[idx] = 50;      // R
          imageData.data[idx + 1] = 30;  // G
          imageData.data[idx + 2] = 100; // B
        }
        
        imageData.data[idx + 3] = shadowValue > 0.01 ? 200 : 100; // A
      }
    }
    ctx.putImageData(imageData, 0, 0);

    // Add as image overlay
    const imageUrl = canvas.toDataURL();
    shadowLayer.current = L.imageOverlay(imageUrl, bounds, {
      opacity: 0.7,
    }).addTo(map.current);
    
    // Bring drawn items to front
    drawnItems.current.bringToFront();
  }, [shadowData, hasResults]);

  return (
    <div 
      ref={mapContainer} 
      className="w-full h-full"
      style={{ background: '#f0f0f0' }}
    />
  );
};

export default LeafletMapView;