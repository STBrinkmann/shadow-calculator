import React, { useEffect, useRef } from 'react';
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

interface RasterData {
  data: number[][];
  bounds: RasterBounds;
  transform: number[];
}

interface AllSummaryData {
  total_shadow_hours: number[][];
  avg_shadow_percentage: number[][];
  max_consecutive_shadow: number[][];
  morning_shadow_hours: number[][];
  afternoon_shadow_hours: number[][];
  bounds: RasterBounds;
  transform: number[];
}

interface MapViewProps {
  onAOIDrawn: (coordinates: number[][]) => void;
  rasterBounds?: RasterBounds | null;
  averageShadowRaster?: RasterData | null;
  allSummaryData?: AllSummaryData | null;
}

const LeafletMapView: React.FC<MapViewProps> = ({ 
  onAOIDrawn, 
  rasterBounds,
  averageShadowRaster,
  allSummaryData
}) => {
  const mapContainer = useRef<HTMLDivElement>(null);
  const map = useRef<L.Map | null>(null);
  const drawnItems = useRef<L.FeatureGroup | null>(null);
  const averageShadowLayer = useRef<L.ImageOverlay | null>(null);
  const legendControl = useRef<L.Control | null>(null);
  const rasterBoundsLayer = useRef<L.Rectangle | null>(null);

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
            color: '#dc2626',        // Red color
            fillColor: '#dc2626',    // Red fill color  
            fillOpacity: 0,          // No fill
            weight: 2,               // Line thickness
            interactive: false,      // Disable interaction to allow clicks through
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
      
      // Disable interaction on the polygon to allow clicks through
      layer.options.interactive = false;
      if (layer.setStyle) {
        layer.setStyle({ interactive: false });
      }
      
      // Clear previous drawings
      drawnItems.current?.clearLayers();
      
      // Add new polygon
      drawnItems.current?.addLayer(layer);
      
      // Disable events on the polygon layer to allow click-through
      layer.off();
      
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
    const info = new L.Control({ position: 'topright' });
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
      if (averageShadowLayer.current) {
        // Clean up tooltip if it exists
        const tooltip = (averageShadowLayer.current as any)._tooltip;
        if (tooltip && tooltip.parentNode) {
          tooltip.parentNode.removeChild(tooltip);
        }
        map.current?.removeLayer(averageShadowLayer.current);
      }
      if (legendControl.current) {
        map.current?.removeControl(legendControl.current);
      }
      if (rasterBoundsLayer.current) {
        map.current?.removeLayer(rasterBoundsLayer.current);
      }
      map.current?.remove();
      map.current = null;
    };
  }, [onAOIDrawn]);

  // Handle raster bounds - zoom to extent and show bounding box
  useEffect(() => {
    if (!map.current) return;

    // Remove existing bounding box if it exists
    if (rasterBoundsLayer.current) {
      map.current.removeLayer(rasterBoundsLayer.current);
      rasterBoundsLayer.current = null;
    }

    if (!rasterBounds) return;

    // Create bounds for zooming and visualization
    const bounds = L.latLngBounds(
      [rasterBounds.min_lat, rasterBounds.min_lon],
      [rasterBounds.max_lat, rasterBounds.max_lon]
    );

    // Add visible rectangular bounding box
    rasterBoundsLayer.current = L.rectangle(bounds, {
      color: '#3b82f6',      // Blue color
      fillColor: '#3b82f6',  // Blue fill
      fillOpacity: 0.1,      // Low fill opacity
      weight: 2,             // Border thickness
      opacity: 0.8,          // Border opacity
      dashArray: '5, 5',     // Dashed border
      interactive: false     // Allow clicks through
    }).addTo(map.current);

    // Add popup to the bounding box
    rasterBoundsLayer.current.bindPopup(`
      <div style="color: black; font-size: 12px;">
        <strong>Raster Data Extent</strong><br/>
        <strong>Longitude:</strong> ${rasterBounds.min_lon.toFixed(4)}° to ${rasterBounds.max_lon.toFixed(4)}°<br/>
        <strong>Latitude:</strong> ${rasterBounds.min_lat.toFixed(4)}° to ${rasterBounds.max_lat.toFixed(4)}°<br/>
        <strong>Size:</strong> ${((rasterBounds.max_lon - rasterBounds.min_lon) * 111320).toFixed(0)}m × ${((rasterBounds.max_lat - rasterBounds.min_lat) * 111320).toFixed(0)}m (approx)
      </div>
    `);

    // Zoom to bounds with some padding
    map.current.fitBounds(bounds, {
      padding: [50, 50],
      maxZoom: 16
    });

    console.log('Added raster bounding box and zoomed to bounds:', bounds);
  }, [rasterBounds]);

  // Handle shadow data visualization - REMOVED: Now using averageShadowRaster instead
  // This was creating the rectangular bounding box overlay

  // Handle average shadow raster visualization
  useEffect(() => {
    if (!map.current) return;
    
    if (!averageShadowRaster) {
      // Remove existing layers when no raster data
      if (averageShadowLayer.current) {
        // Clean up tooltip if it exists
        const tooltip = (averageShadowLayer.current as any)._tooltip;
        if (tooltip && tooltip.parentNode) {
          tooltip.parentNode.removeChild(tooltip);
        }
        map.current.removeLayer(averageShadowLayer.current);
        averageShadowLayer.current = null;
      }
      if (legendControl.current) {
        map.current.removeControl(legendControl.current);
        legendControl.current = null;
      }
      return;
    }

    // Remove existing average shadow layer
    if (averageShadowLayer.current) {
      // Clean up tooltip if it exists
      const tooltip = (averageShadowLayer.current as any)._tooltip;
      if (tooltip && tooltip.parentNode) {
        tooltip.parentNode.removeChild(tooltip);
      }
      map.current.removeLayer(averageShadowLayer.current);
      averageShadowLayer.current = null;
    }

    const { data, bounds } = averageShadowRaster;
    const height = data.length;
    const width = data[0]?.length || 0;
    
    if (width === 0 || height === 0) return;

    // Create canvas for average shadow visualization
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;

    // Create improved shadow color mapping with better low-shadow visualization
    const imageData = ctx.createImageData(width, height);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const idx = (y * width + x) * 4;
        const shadowValue = Math.max(0, Math.min(1, data[y][x])); // Clamp to 0-1
        
        // Improved shadow color scheme with sand/white for low shadows
        let r, g, b, a;
        
        if (shadowValue <= 0.5) {
          // Low shadow areas (0-50%): Sand/white tones with moderate transparency
          const t = shadowValue / 0.5;
          r = Math.round(255 - t * 35);   // 255 -> 220 (stay light/sandy)
          g = Math.round(250 - t * 30);   // 250 -> 220 (warm sand color)
          b = Math.round(235 - t * 65);   // 235 -> 170 (slight brown tint)
          a = Math.round(60 + t * 80);    // More visible: 60 -> 140 (24%-55% opacity)
        } else if (shadowValue < 0.7) {
          // Medium shadow: Transition from sand to orange
          const t = (shadowValue - 0.5) / 0.2;
          r = Math.round(220 + t * 35);   // 220 -> 255
          g = Math.round(220 - t * 70);   // 220 -> 150
          b = Math.round(170 - t * 170);  // 170 -> 0
          a = Math.round(140 + t * 60);   // 140 -> 200 (55%-78% opacity)
        } else if (shadowValue < 0.85) {
          // Heavy shadow: Orange to red
          const t = (shadowValue - 0.7) / 0.15;
          r = 255;                        // Stay full red
          g = Math.round(150 - t * 90);   // 150 -> 60
          b = Math.round(t * 30);         // 0 -> 30
          a = Math.round(200 + t * 35);   // 200 -> 235 (78%-92% opacity)
        } else {
          // Very heavy shadow: Red to dark purple
          const t = (shadowValue - 0.85) / 0.15;
          r = Math.round(255 - t * 155);  // 255 -> 100
          g = Math.round(60 - t * 40);    // 60 -> 20
          b = Math.round(30 + t * 120);   // 30 -> 150
          a = Math.round(235 + t * 20);   // 235 -> 255 (92%-100% opacity)
        }
        
        imageData.data[idx] = r;
        imageData.data[idx + 1] = g;
        imageData.data[idx + 2] = b;
        imageData.data[idx + 3] = a;
      }
    }
    ctx.putImageData(imageData, 0, 0);

    // Convert transform to Leaflet bounds
    const rasterBounds = L.latLngBounds(
      [bounds.min_lat, bounds.min_lon],
      [bounds.max_lat, bounds.max_lon]
    );

    // Add as image overlay
    const imageUrl = canvas.toDataURL();
    averageShadowLayer.current = L.imageOverlay(imageUrl, rasterBounds, {
      opacity: 0.75,
      interactive: true,
    }).addTo(map.current);

    // Add hover tooltip and click popup functionality
    if (allSummaryData) {
      // Create tooltip element
      const tooltip = document.createElement('div');
      tooltip.style.cssText = `
        position: absolute;
        background: rgba(0, 0, 0, 0.8);
        color: white;
        padding: 6px 10px;
        border-radius: 4px;
        font-size: 12px;
        pointer-events: none;
        z-index: 1000;
        display: none;
        white-space: nowrap;
      `;
      document.body.appendChild(tooltip);

      // Helper function to get raster value at lat/lng
      const getRasterValueAtLatLng = (latlng: L.LatLng, data: number[][]) => {
        const { bounds } = allSummaryData;
        const latRange = bounds.max_lat - bounds.min_lat;
        const lngRange = bounds.max_lon - bounds.min_lon;
        
        // Convert lat/lng to array indices
        const rowFloat = ((bounds.max_lat - latlng.lat) / latRange) * data.length;
        const colFloat = ((latlng.lng - bounds.min_lon) / lngRange) * data[0].length;
        
        const row = Math.floor(rowFloat);
        const col = Math.floor(colFloat);
        
        // Check bounds
        if (row >= 0 && row < data.length && col >= 0 && col < data[0].length) {
          return data[row][col];
        }
        return null;
      };

      // Mouse move handler for tooltip
      const onMouseMove = (e: L.LeafletMouseEvent) => {
        const shadowValue = getRasterValueAtLatLng(e.latlng, allSummaryData.avg_shadow_percentage);
        
        if (shadowValue !== null) {
          const shadowPercent = Math.round(shadowValue * 100);
          tooltip.innerHTML = `Shadow: ${shadowPercent}%`;
          tooltip.style.display = 'block';
          tooltip.style.left = (e.originalEvent.clientX + 10) + 'px';
          tooltip.style.top = (e.originalEvent.clientY - 30) + 'px';
        }
      };

      // Mouse out handler
      const onMouseOut = () => {
        tooltip.style.display = 'none';
      };

      // Click handler for popup
      const onClick = (e: L.LeafletMouseEvent) => {
        const shadowValue = getRasterValueAtLatLng(e.latlng, allSummaryData.avg_shadow_percentage);
        const totalHours = getRasterValueAtLatLng(e.latlng, allSummaryData.total_shadow_hours);
        const maxConsecutive = getRasterValueAtLatLng(e.latlng, allSummaryData.max_consecutive_shadow);
        const morningHours = getRasterValueAtLatLng(e.latlng, allSummaryData.morning_shadow_hours);
        const afternoonHours = getRasterValueAtLatLng(e.latlng, allSummaryData.afternoon_shadow_hours);

        if (shadowValue !== null) {
          const popupContent = `
            <div style="font-family: sans-serif; min-width: 200px;">
              <h4 style="margin: 0 0 12px 0; color: #333; font-size: 16px; border-bottom: 2px solid #4f46e5; padding-bottom: 4px;">
                📊 Shadow Statistics
              </h4>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 13px;">
                <div style="background: #f3f4f6; padding: 8px; border-radius: 4px;">
                  <div style="font-weight: bold; color: #6b7280; font-size: 11px;">AVERAGE SHADOW</div>
                  <div style="font-size: 16px; color: #1f2937; font-weight: bold;">${Math.round(shadowValue * 100)}%</div>
                </div>
                <div style="background: #fef3c7; padding: 8px; border-radius: 4px;">
                  <div style="font-weight: bold; color: #92400e; font-size: 11px;">TOTAL HOURS</div>
                  <div style="font-size: 16px; color: #92400e; font-weight: bold;">${totalHours?.toFixed(1) || 'N/A'}h</div>
                </div>
                <div style="background: #fee2e2; padding: 8px; border-radius: 4px;">
                  <div style="font-weight: bold; color: #991b1b; font-size: 11px;">MAX CONSECUTIVE</div>
                  <div style="font-size: 16px; color: #991b1b; font-weight: bold;">${maxConsecutive?.toFixed(1) || 'N/A'}h</div>
                </div>
                <div style="background: #ecfdf5; padding: 8px; border-radius: 4px;">
                  <div style="font-weight: bold; color: #065f46; font-size: 11px;">MORNING/AFTERNOON</div>
                  <div style="font-size: 14px; color: #065f46; font-weight: bold;">${morningHours?.toFixed(1) || 'N/A'}h / ${afternoonHours?.toFixed(1) || 'N/A'}h</div>
                </div>
              </div>
              <div style="margin-top: 8px; font-size: 11px; color: #6b7280; text-align: center;">
                📍 Lat: ${e.latlng.lat.toFixed(5)}, Lng: ${e.latlng.lng.toFixed(5)}
              </div>
            </div>
          `;

          L.popup({
            maxWidth: 300,
            className: 'shadow-stats-popup'
          })
          .setLatLng(e.latlng)
          .setContent(popupContent)
          .openOn(map.current!);
        }
      };

      // Add event listeners
      averageShadowLayer.current.on('mousemove', onMouseMove);
      averageShadowLayer.current.on('mouseout', onMouseOut);
      averageShadowLayer.current.on('click', onClick);

      // Store tooltip reference for cleanup
      (averageShadowLayer.current as any)._tooltip = tooltip;
    }
    
    // Add legend for shadow visualization
    if (!legendControl.current) {
      legendControl.current = new L.Control({ position: 'bottomright' });
      legendControl.current.onAdd = function () {
        const div = L.DomUtil.create('div', 'legend');
        div.style.backgroundColor = 'white';
        div.style.padding = '10px';
        div.style.borderRadius = '5px';
        div.style.boxShadow = '0 0 15px rgba(0,0,0,0.2)';
        div.style.fontSize = '12px';
        div.innerHTML = `
          <h4 style="margin: 0 0 8px 0; color: black;">Average Shadow %</h4>
          <div style="display: flex; align-items: center; margin-bottom: 4px;">
            <div style="width: 20px; height: 15px; background: linear-gradient(to right, #FFFF64, #FFC832); margin-right: 8px; border: 1px solid #ccc;"></div>
            <span style="color: black;">0-25%</span>
          </div>
          <div style="display: flex; align-items: center; margin-bottom: 4px;">
            <div style="width: 20px; height: 15px; background: linear-gradient(to right, #FFC832, #FF6400); margin-right: 8px; border: 1px solid #ccc;"></div>
            <span style="color: black;">25-50%</span>
          </div>
          <div style="display: flex; align-items: center; margin-bottom: 4px;">
            <div style="width: 20px; height: 15px; background: linear-gradient(to right, #FF6400, #641E1E); margin-right: 8px; border: 1px solid #ccc;"></div>
            <span style="color: black;">50-75%</span>
          </div>
          <div style="display: flex; align-items: center;">
            <div style="width: 20px; height: 15px; background: linear-gradient(to right, #641E1E, #320A96); margin-right: 8px; border: 1px solid #ccc;"></div>
            <span style="color: black;">75-100%</span>
          </div>
        `;
        return div;
      };
      if (legendControl.current && map.current) {
        legendControl.current.addTo(map.current);
      }
    }
    
    // Don't bring drawn items to front - let them stay behind for click-through
  }, [averageShadowRaster, allSummaryData]);

  return (
    <div 
      ref={mapContainer} 
      className="w-full h-full"
      style={{ background: '#f0f0f0' }}
    />
  );
};

export default LeafletMapView;