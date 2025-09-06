import React, { useEffect, useRef, useState } from 'react';
import mapboxgl from 'mapbox-gl';
import MapboxDraw from '@mapbox/mapbox-gl-draw';
import '@mapbox/mapbox-gl-draw/dist/mapbox-gl-draw.css';
import 'mapbox-gl/dist/mapbox-gl.css';

// Get token from environment variable
const MAPBOX_TOKEN = import.meta.env.VITE_MAPBOX_TOKEN || 'YOUR_MAPBOX_TOKEN_HERE';
mapboxgl.accessToken = MAPBOX_TOKEN;

interface MapViewProps {
  onAOIDrawn: (coordinates: number[][]) => void;
  shadowData: number[][];
  hasResults: boolean;
}

const MapView: React.FC<MapViewProps> = ({ onAOIDrawn, shadowData, hasResults }) => {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<mapboxgl.Map | null>(null);
  const draw = useRef<MapboxDraw | null>(null);
  const [mapLoaded, setMapLoaded] = useState(false);

  useEffect(() => {
    if (!mapContainer.current || map.current) return;

    // Check if token is set
    if (MAPBOX_TOKEN === 'YOUR_MAPBOX_TOKEN_HERE') {
      console.error('Please set your Mapbox token in .env.local file');
      // Create a placeholder
      const div = mapContainer.current;
      div.innerHTML = `
        <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #1a1a1a; color: white;">
          <div style="text-align: center; padding: 20px;">
            <h3 style="font-size: 24px; margin-bottom: 20px;">🗺️ Map View</h3>
            <p style="margin-bottom: 10px;">Please add your Mapbox token to .env.local file:</p>
            <code style="background: #333; padding: 10px; border-radius: 5px; display: inline-block; margin: 10px 0;">VITE_MAPBOX_TOKEN=your_token_here</code>
            <p style="margin-top: 20px;">Get a free token at: 
              <a href="https://www.mapbox.com/" target="_blank" style="color: #3b82f6; text-decoration: underline;">mapbox.com</a>
            </p>
            <p style="margin-top: 10px; color: #999; font-size: 14px;">After adding the token, restart the app with npm run tauri dev</p>
          </div>
        </div>
      `;
      return;
    }

    try {
      map.current = new mapboxgl.Map({
        container: mapContainer.current,
        style: 'mapbox://styles/mapbox/satellite-v9',
        center: [8.5417, 47.3769], // Default to Zurich
        zoom: 13,
      });

      draw.current = new MapboxDraw({
        displayControlsDefault: false,
        controls: {
          polygon: true,
          trash: true,
        },
        defaultMode: 'draw_polygon',
      });

      map.current.addControl(draw.current, 'top-left');
      map.current.addControl(new mapboxgl.NavigationControl(), 'top-right');
      map.current.addControl(new mapboxgl.ScaleControl(), 'bottom-right');

      map.current.on('load', () => {
        setMapLoaded(true);
        console.log('Map loaded successfully');
      });

      map.current.on('draw.create', (e) => {
        const feature = e.features[0];
        if (feature && feature.geometry.type === 'Polygon') {
          const coords = feature.geometry.coordinates[0];
          console.log('AOI drawn:', coords);
          onAOIDrawn(coords);
        }
      });

      map.current.on('draw.update', (e) => {
        const feature = e.features[0];
        if (feature && feature.geometry.type === 'Polygon') {
          const coords = feature.geometry.coordinates[0];
          onAOIDrawn(coords);
        }
      });

      map.current.on('draw.delete', () => {
        onAOIDrawn([]);
      });

    } catch (error) {
      console.error('Error initializing map:', error);
    }

    return () => {
      map.current?.remove();
    };
  }, [onAOIDrawn]);

  // Shadow visualization effect
  useEffect(() => {
    if (!map.current || !mapLoaded || !hasResults || !shadowData.length) return;

    // Remove existing shadow layer if present
    if (map.current.getLayer('shadow-layer')) {
      map.current.removeLayer('shadow-layer');
    }
    if (map.current.getSource('shadow-source')) {
      map.current.removeSource('shadow-source');
    }

    // Create canvas for shadow visualization
    const canvas = document.createElement('canvas');
    const height = shadowData.length;
    const width = shadowData[0]?.length || 0;
    
    if (width === 0 || height === 0) return;

    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;

    // Draw shadow data with gradient
    const imageData = ctx.createImageData(width, height);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const idx = (y * width + x) * 4;
        const shadowValue = shadowData[y][x];
        
        // Create gradient from blue (no shadow) to dark purple (full shadow)
        const intensity = 1 - shadowValue;
        imageData.data[idx] = 50 * (1 - intensity);     // R
        imageData.data[idx + 1] = 50 * (1 - intensity); // G
        imageData.data[idx + 2] = 100 * (1 - intensity) + 50; // B
        imageData.data[idx + 3] = shadowValue * 200 + 55; // A
      }
    }
    ctx.putImageData(imageData, 0, 0);

    // Add as map layer
    const dataUrl = canvas.toDataURL();
    
    // Get bounds from drawn AOI
    const features = draw.current?.getAll();
    if (features && features.features.length > 0) {
      const coords = features.features[0].geometry.coordinates[0];
      const bounds = coords.reduce(
        (bounds: any, coord: number[]) => {
          return [
            [Math.min(bounds[0][0], coord[0]), Math.min(bounds[0][1], coord[1])],
            [Math.max(bounds[1][0], coord[0]), Math.max(bounds[1][1], coord[1])],
          ];
        },
        [[Infinity, Infinity], [-Infinity, -Infinity]]
      );

      map.current.addSource('shadow-source', {
        type: 'image',
        url: dataUrl,
        coordinates: [
          [bounds[0][0], bounds[1][1]],
          [bounds[1][0], bounds[1][1]],
          [bounds[1][0], bounds[0][1]],
          [bounds[0][0], bounds[0][1]],
        ],
      });

      map.current.addLayer({
        id: 'shadow-layer',
        type: 'raster',
        source: 'shadow-source',
        paint: {
          'raster-opacity': 0.7,
          'raster-fade-duration': 0,
        },
      });
    }
  }, [shadowData, hasResults, mapLoaded]);

  return (
    <div className="relative w-full h-full">
      <div ref={mapContainer} className="w-full h-full" />
      {mapLoaded && (
        <div className="absolute top-4 left-1/2 transform -translate-x-1/2 bg-gray-800 bg-opacity-90 px-4 py-2 rounded-lg shadow-lg">
          <p className="text-sm text-gray-300">Draw a polygon to define your Area of Interest (AOI)</p>
        </div>
      )}
    </div>
  );
};

export default MapView;