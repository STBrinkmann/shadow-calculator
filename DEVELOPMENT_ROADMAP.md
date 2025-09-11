# Shadow Calculator Development Roadmap

## Current Architecture Overview

### Core Components
- **Rust Backend** (`src-tauri/src/`): High-performance shadow calculation engine using ray marching
- **React Frontend** (`src/`): Interactive map interface with Leaflet integration
- **Data Flow**: Tauri commands → GDAL raster processing → parallel computation → JSON results → React visualization

### Established Patterns
- **Backend Calculations**: Array3<f32> for 3D data, parallel processing with Rayon
- **Frontend Components**: TypeScript with Tailwind CSS, modular component structure
- **Data Interfaces**: Strongly typed interfaces for Rust↔TypeScript communication
- **UI Conventions**: Popup-based information display, grid layouts, consistent color schemes

### Key Technical Strengths
- Fast parallel shadow computation (40k cells × 5.8k timestamps in 2-10 minutes)
- Solar-aware timestamp generation (daylight hours only)
- Interactive map visualization with real-time raster overlay
- Comprehensive summary statistics (total, consecutive, time-period breakdowns)

## Implementation Priority Order

### TIER 1: High Impact, Lower Complexity
1. **Seasonal Analysis Dashboard** - Leverage existing temporal engine
2. **Solar Panel Placement Optimizer** - Clear commercial value
3. **Plant Sun/Shade Requirements Database** - Gardening market appeal

### TIER 2: Medium Complexity, High User Value  
4. **Garden Design Canvas** - Visual planning integration
5. **Advanced Solar Engineering** - Professional tooling
6. **Seasonal Planting Calendar** - Smart scheduling system

### TIER 3: Future Expansion
7. **Microclimate Modeling** - Advanced physics simulation
8. **Collaborative Planning** - Multi-user features
9. **Mobile Field App** - On-site analysis tools

## Shared Technical Patterns

### Backend Rust Patterns
```rust
// Statistical calculation pattern (reference: shadow_engine.rs:496-697)
fn calculate_[feature]_stats(&self, shadow_fraction: &Array3<f32>) -> [Feature]Stats {
    // Parallel processing using rayon
    let results: Vec<_> = cell_coords.par_iter()
        .map(|&(row, col)| { /* calculation */ })
        .collect();
}

// Data structure pattern (reference: types.rs:82-93)
#[derive(Debug, Clone)]
pub struct [Feature]Stats {
    pub [metric]: Array3<f32>,
    // ... other metrics
}
```

### Frontend React Patterns
```typescript
// Component structure pattern (reference: App.tsx:46-100)
interface [Feature]Props {
    data: [Feature]Data | null;
    onUpdate: (data: [Feature]Data) => void;
}

// Data fetching pattern (reference: App.tsx:172-200)
const handle[Feature]Calculate = useCallback(async () => {
    const result = await invoke<[Feature]Data>('[feature]_command', { config });
    set[Feature]Data(result);
}, [dependencies]);
```

### UI Consistency Rules
- **Colors**: Use existing shadow visualization palette (#fef3c7 shadow, #dcfce7 sun, #fee2e2 alerts)
- **Layout**: Follow App.tsx sidebar/main structure, LeafletMapView popup patterns
- **Typography**: Consistent font sizes (11px labels, 14px content, 16px highlights)
- **Spacing**: 8px grid system, 12px margins, 4px border radius

---

## Feature Specifications

## TIER 1 FEATURES

### 1. Seasonal Analysis Dashboard

**Context**: Users need to understand how shadow patterns change across seasons for solar planning and garden design. Our existing temporal calculation engine already generates timestamped data - we need to group and visualize by month/season.

**Prerequisites**:
- Current shadow calculation working ✓
- Summary statistics system established ✓ 
- UI popup and visualization patterns established ✓

**Implementation Steps**:

#### Step 1a: Backend - Add Seasonal Data Structures
**File**: `src-tauri/src/types.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyShadowStats {
    pub month: u32,
    pub year: i32,
    pub total_shadow_hours: Array3<f32>,
    pub avg_shadow_percentage: Array3<f32>,
    pub max_consecutive_shadow: Array3<f32>,
    pub solar_efficiency_percentage: Array3<f32>,
    pub days_in_analysis: u32,
}

#[derive(Debug, Clone)]
pub struct SeasonalAnalysis {
    pub monthly_stats: Vec<MonthlyShadowStats>,
    pub seasonal_summaries: [SeasonStats; 4], // Spring, Summer, Fall, Winter
}
```

#### Step 1b: Backend - Add Seasonal Calculation Logic
**File**: `src-tauri/src/shadow_engine.rs`
Add method after `calculate_summary_stats()`:
```rust
pub fn calculate_seasonal_analysis(
    &self,
    shadow_fraction: &Array3<f32>,
    timestamps: &[chrono::DateTime<chrono::Utc>]
) -> SeasonalAnalysis {
    // Group timestamps by month using chrono
    // Calculate monthly statistics using existing patterns
    // Aggregate into seasonal summaries (3-month groups)
}
```

#### Step 1c: Backend - Add Tauri Command
**File**: `src-tauri/src/main.rs`
```rust
#[tauri::command]
async fn get_seasonal_analysis(app_handle: AppHandle) -> Result<SeasonalAnalysis, String> {
    // Use existing pattern from get_all_summary_data
    // Return seasonal analysis instead of current summary
}
```

#### Step 2a: Frontend - Add Seasonal Analysis Interface
**File**: `src/types.ts`
```typescript
interface MonthlyShadowStats {
    month: number;
    year: number;
    total_shadow_hours: number[][];
    avg_shadow_percentage: number[][];
    max_consecutive_shadow: number[][];
    solar_efficiency_percentage: number[][];
    days_in_analysis: number;
}

interface SeasonalAnalysis {
    monthly_stats: MonthlyShadowStats[];
    seasonal_summaries: SeasonStats[];
}
```

#### Step 2b: Frontend - Create Seasonal Dashboard Component
**File**: `src/components/SeasonalDashboard.tsx`
```typescript
// Follow App.tsx structure for state management
// Use grid layout (4 cols desktop, 2 mobile)
// Create MonthCard sub-components showing key metrics
// Add timeline slider for animation
```

#### Step 2c: Frontend - Integration with Main App
**File**: `src/App.tsx`
- Add "Seasonal Analysis" tab next to existing controls
- Add state for seasonal data: `const [seasonalData, setSeasonalData] = useState<SeasonalAnalysis | null>(null);`
- Add handler: `const handleSeasonalAnalysis = useCallback(async () => { /* invoke seasonal command */ }, []);`

**Technical Constraints**:
- Reuse existing Array3<f32> → number[][] conversion patterns from main.rs:330-380
- Follow LeafletMapView popup styling for month cards
- Maintain performance: group existing data, don't recalculate shadows
- Use consistent color schemes from current visualization

**Success Criteria**:
- Display 12 months of shadow data simultaneously
- Smooth animation between months (< 300ms transitions)
- Performance under 3 seconds for seasonal view generation
- Mobile responsive month grid layout

**Testing Strategy**:
- Test with different date ranges (3 months vs full year)
- Verify monthly calculations sum to annual totals
- Check seasonal boundaries (Dec-Jan-Feb = Winter)
- Test edge cases (polar regions, single month data)

---

### 2. Solar Panel Placement Optimizer

**Context**: Solar installers need tools to determine optimal panel placement based on shading analysis. This builds on our shadow data to provide actionable solar energy recommendations with clear commercial value.

**Prerequisites**:
- Seasonal analysis complete (for year-round optimization)
- Shadow statistics working ✓
- Interactive map functionality ✓

**Implementation Steps**:

#### Step 1a: Backend - Solar Panel Data Structures
**File**: `src-tauri/src/types.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarPanelConfig {
    pub panel_width: f64,      // meters
    pub panel_height: f64,     // meters  
    pub panel_efficiency: f64, // 0.0-1.0 (e.g. 0.22 for 22%)
    pub tilt_angle: f64,       // degrees from horizontal
    pub azimuth_angle: f64,    // degrees from south (0=south, 90=west)
    pub system_losses: f64,    // 0.0-1.0 (inverter, wiring, soiling losses)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarAnalysisResult {
    pub daily_kwh_generation: Array3<f32>,    // kWh per day per cell
    pub annual_kwh_total: Array3<f32>,        // Total annual kWh
    pub capacity_factor: Array3<f32>,         // Actual/theoretical output ratio
    pub optimal_locations: Vec<OptimalSpot>,   // Top N locations ranked
    pub financial_analysis: FinancialMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalSpot {
    pub row: usize,
    pub col: usize,
    pub annual_kwh: f32,
    pub capacity_factor: f32,
    pub shading_loss_percent: f32,
}
```

#### Step 1b: Backend - Solar Calculation Engine
**File**: `src-tauri/src/solar_optimizer.rs` (new module)
```rust
use crate::types::*;
use crate::sun_position::SunCalculator;

pub struct SolarOptimizer {
    shadow_data: SummaryStats,
    panel_config: SolarPanelConfig,
    sun_calculator: SunCalculator,
}

impl SolarOptimizer {
    pub fn calculate_solar_potential(&self) -> SolarAnalysisResult {
        // For each cell, calculate:
        // 1. Available solar irradiance (using solar_efficiency_percentage)
        // 2. Panel energy output (efficiency × irradiance × area)
        // 3. System losses (inverter, wiring, soiling)
        // 4. Financial metrics (cost/benefit analysis)
    }
    
    fn calculate_irradiance_with_shading(&self, row: usize, col: usize) -> f32 {
        // Use existing solar_efficiency_percentage from shadow analysis
        // Apply solar irradiance models (simplified: 1000W/m² peak × efficiency)
    }
}
```

#### Step 2a: Frontend - Solar Optimizer Interface
**File**: `src/components/SolarOptimizer.tsx`
```typescript
interface SolarOptimizerProps {
    shadowData: AllSummaryData | null;
    onOptimizationComplete: (results: SolarAnalysisResult) => void;
}

// Component structure:
// 1. Panel configuration form (size, efficiency, tilt, azimuth)
// 2. Financial parameters (electricity cost, system cost)
// 3. "Optimize Placement" button
// 4. Results visualization (heatmap overlay + ranked spots)
```

#### Step 2b: Frontend - Solar Results Overlay
**File**: `src/components/LeafletMapView.tsx` (enhance existing)
```typescript
// Add new overlay mode for solar potential
// Color coding: Green (high generation) → Red (low generation)
// Click handler: Show detailed solar metrics in popup
// Integration with existing raster overlay system
```

#### Step 2c: Frontend - Solar Configuration Panel
**File**: `src/components/SolarConfigPanel.tsx`
```typescript
// Form inputs for panel specifications
// Presets for common panel types (residential, commercial)
// Real-time preview of affected area on map
// Cost calculator integration
```

**Technical Constraints**:
- Use existing parallel processing patterns from shadow_engine.rs
- Integrate with current map overlay system in LeafletMapView.tsx
- Follow existing form patterns from TimeControls.tsx
- Maintain performance: leverage pre-calculated shadow data

**Success Criteria**:
- Generate solar potential map in < 5 seconds
- Identify top 10 optimal placement locations
- Calculate annual kWh estimates within 15% accuracy
- Provide actionable recommendations (tilt, azimuth, placement)

**Testing Strategy**:
- Compare results with established solar calculators (PVWatts)
- Test different panel configurations and orientations
- Validate with known installations (if available)
- Check edge cases (heavily shaded areas, optimal areas)

---

### 3. Plant Sun/Shade Requirements Database & Collaborative Platform

**Context**: Gardeners need to know which plants will thrive in specific light conditions. This feature transforms the application into a collaborative platform where users contribute to a shared plant knowledge base, accelerating database growth while maintaining quality through AI assistance and community validation.

**Prerequisites**:
- Shadow analysis working ✓
- Interactive map with popup system ✓
- Web architecture established for hosted deployment

**Implementation Steps**:

#### Step 1a: Backend - Enhanced Plant Database Schema
**File**: `src-tauri/src/types.rs` (for desktop) + database schema for webapp
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantRequirements {
    pub plant_id: u32,
    pub common_name: String,
    pub scientific_name: String,
    pub sun_requirements: SunRequirement,
    pub min_daily_hours: f32,        // Minimum sun hours needed
    pub max_daily_hours: f32,        // Maximum sun hours tolerated
    pub preferred_hours: f32,         // Optimal sun hours
    pub shade_tolerance: ShadeTolerance,
    pub plant_type: PlantType,
    pub growing_season: Vec<Season>,  // Multiple seasons possible
    pub mature_size: PlantSize,
    pub mature_dimensions: PlantDimensions,
    pub soil_requirements: SoilRequirements,
    pub water_needs: WaterRequirement,
    pub temperature_range: TemperatureRange,
    pub companion_plants: Vec<u32>,   // IDs of beneficial companion plants
    pub avoid_plants: Vec<u32>,       // IDs of plants to avoid nearby
    pub care_instructions: Vec<CareInstruction>,
    pub harvest_info: Option<HarvestInfo>,
    
    // Community & validation fields
    pub contributor_id: Option<String>,
    pub validation_status: ValidationStatus,
    pub community_rating: f32,        // 1.0-5.0 based on user feedback
    pub verified_by_experts: bool,
    pub ai_assisted: bool,            // True if AI helped populate fields
    pub created_date: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub update_history: Vec<PlantUpdateRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDimensions {
    pub height_cm: (u32, u32),        // (min, max) height
    pub spread_cm: (u32, u32),        // (min, max) spread
    pub root_depth_cm: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoilRequirements {
    pub ph_range: (f32, f32),         // (min, max) pH
    pub drainage: DrainageRequirement,
    pub soil_types: Vec<SoilType>,    // Clay, Sandy, Loam, etc.
    pub organic_matter: bool,         // Requires rich organic soil
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,           // Awaiting review
    CommunityVerified, // Verified by multiple users
    ExpertVerified,    // Verified by gardening expert
    Disputed,          // Conflicting information reported
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAssistanceConfig {
    pub provider: AIProvider,
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Google,
    Custom { endpoint: String },
}
```

#### Step 1b: Backend - Plant Database Management System
**File**: `src-tauri/src/plant_database_manager.rs` (new module)
```rust
pub struct PlantDatabaseManager {
    local_db: SqliteConnection,      // Local cache
    remote_api: Option<RemoteAPI>,   // Connection to hosted database
    ai_config: Option<AIAssistanceConfig>,
}

impl PlantDatabaseManager {
    pub async fn add_plant_with_ai_assistance(
        &self, 
        plant_name: String,
        scientific_name: Option<String>
    ) -> Result<PlantRequirements, DatabaseError> {
        // 1. Query AI for plant characteristics
        if let Some(ai_config) = &self.ai_config {
            let ai_data = self.query_ai_for_plant_data(plant_name, scientific_name, ai_config).await?;
            
            // 2. Present AI suggestions to user for validation
            let validated_data = self.present_for_user_validation(ai_data)?;
            
            // 3. Save to both local and remote databases
            self.save_plant_entry(validated_data).await
        } else {
            // Fallback to manual entry form
            Err(DatabaseError::AINotConfigured)
        }
    }
    
    async fn query_ai_for_plant_data(
        &self,
        common_name: String,
        scientific_name: Option<String>,
        config: &AIAssistanceConfig
    ) -> Result<AIPlantSuggestion, AIError> {
        let prompt = self.build_plant_query_prompt(&common_name, &scientific_name);
        
        match config.provider {
            AIProvider::OpenAI => self.query_openai(&prompt, config).await,
            AIProvider::Anthropic => self.query_claude(&prompt, config).await,
            AIProvider::Google => self.query_gemini(&prompt, config).await,
            AIProvider::Custom { endpoint } => self.query_custom_endpoint(&prompt, config, endpoint).await,
        }
    }
    
    fn build_plant_query_prompt(&self, common_name: &str, scientific_name: &Option<String>) -> String {
        format!(
            "Please provide detailed growing requirements for the plant '{}'{} in JSON format with the following fields:
            - sun_requirements (FullSun/PartialSun/PartialShade/FullShade)
            - min_daily_hours, max_daily_hours, preferred_hours (as numbers)
            - plant_type (Vegetable/Fruit/Herb/Tree/Shrub/Flower/Grass)
            - growing_season (array of Spring/Summer/Fall/Winter)
            - mature_dimensions (height and spread in cm)
            - soil_ph_range, drainage_requirements
            - water_needs, temperature_range
            - companion_plants and plants_to_avoid (common names)
            - care_instructions (array of seasonal care tips)
            - harvest_info (if applicable: harvest_season, days_to_maturity, yield_info)
            
            Provide accurate, research-based information suitable for gardeners.",
            common_name,
            scientific_name.as_ref().map(|s| format!(" ({})", s)).unwrap_or_default()
        )
    }
}
```

#### Step 1c: Backend - AI Integration Services
**File**: `src-tauri/src/ai_services.rs` (new module)
```rust
use reqwest::Client;
use serde_json::{json, Value};

impl PlantDatabaseManager {
    async fn query_openai(&self, prompt: &str, config: &AIAssistanceConfig) -> Result<AIPlantSuggestion, AIError> {
        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&json!({
                "model": config.model,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.1
            }))
            .send()
            .await?;
        
        let response_data: Value = response.json().await?;
        self.parse_ai_response(response_data)
    }
    
    async fn query_claude(&self, prompt: &str, config: &AIAssistanceConfig) -> Result<AIPlantSuggestion, AIError> {
        let client = Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": config.model,
                "max_tokens": 2000,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;
        
        let response_data: Value = response.json().await?;
        self.parse_ai_response(response_data)
    }
    
    // Similar implementations for Google Gemini and custom endpoints
}
```

#### Step 2a: Frontend - Plant Entry Interface with AI Assistance
**File**: `src/components/PlantEntryForm.tsx`
```typescript
interface PlantEntryFormProps {
    onPlantAdded: (plant: PlantRequirements) => void;
    aiConfig: AIAssistanceConfig | null;
}

export const PlantEntryForm: React.FC<PlantEntryFormProps> = ({ onPlantAdded, aiConfig }) => {
    const [plantName, setPlantName] = useState('');
    const [scientificName, setScientificName] = useState('');
    const [aiSuggestions, setAISuggestions] = useState<AIPlantSuggestion | null>(null);
    const [isLoadingAI, setIsLoadingAI] = useState(false);
    const [manualMode, setManualMode] = useState(false);

    const handleAIAssist = async () => {
        if (!aiConfig || !plantName.trim()) return;
        
        setIsLoadingAI(true);
        try {
            const suggestions = await invoke<AIPlantSuggestion>('get_ai_plant_suggestions', {
                commonName: plantName,
                scientificName: scientificName || null,
                aiConfig
            });
            setAISuggestions(suggestions);
        } catch (error) {
            console.error('AI assistance failed:', error);
            // Fall back to manual entry
            setManualMode(true);
        } finally {
            setIsLoadingAI(false);
        }
    };

    return (
        <div className="plant-entry-form p-6 bg-white rounded-lg shadow-lg">
            <h3 className="text-xl font-bold mb-4">Add New Plant to Database</h3>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                <div>
                    <label className="block text-sm font-medium mb-2">Common Name *</label>
                    <input
                        type="text"
                        value={plantName}
                        onChange={(e) => setPlantName(e.target.value)}
                        className="w-full p-2 border rounded-md"
                        placeholder="e.g., Tomato"
                    />
                </div>
                <div>
                    <label className="block text-sm font-medium mb-2">Scientific Name</label>
                    <input
                        type="text"
                        value={scientificName}
                        onChange={(e) => setScientificName(e.target.value)}
                        className="w-full p-2 border rounded-md"
                        placeholder="e.g., Solanum lycopersicum"
                    />
                </div>
            </div>

            <div className="flex gap-4 mb-6">
                {aiConfig && (
                    <button
                        onClick={handleAIAssist}
                        disabled={!plantName.trim() || isLoadingAI}
                        className="bg-blue-500 text-white px-6 py-2 rounded-md hover:bg-blue-600 disabled:opacity-50"
                    >
                        {isLoadingAI ? (
                            <>
                                <span className="animate-spin mr-2">⟳</span>
                                Ask {aiConfig.provider} AI...
                            </>
                        ) : (
                            `🤖 Ask ${aiConfig.provider} AI`
                        )}
                    </button>
                )}
                <button
                    onClick={() => setManualMode(true)}
                    className="bg-gray-500 text-white px-6 py-2 rounded-md hover:bg-gray-600"
                >
                    📝 Manual Entry
                </button>
            </div>

            {aiSuggestions && (
                <AIValidationPanel
                    suggestions={aiSuggestions}
                    onValidate={(validatedData) => onPlantAdded(validatedData)}
                    onReject={() => setManualMode(true)}
                />
            )}

            {manualMode && (
                <ManualPlantEntryForm
                    plantName={plantName}
                    scientificName={scientificName}
                    onSubmit={(plantData) => onPlantAdded(plantData)}
                />
            )}
        </div>
    );
};
```

#### Step 2b: Frontend - AI Validation Panel
**File**: `src/components/AIValidationPanel.tsx`
```typescript
interface AIValidationPanelProps {
    suggestions: AIPlantSuggestion;
    onValidate: (plant: PlantRequirements) => void;
    onReject: () => void;
}

export const AIValidationPanel: React.FC<AIValidationPanelProps> = ({ suggestions, onValidate, onReject }) => {
    const [editedData, setEditedData] = useState(suggestions);
    const [validationNotes, setValidationNotes] = useState<string>('');

    return (
        <div className="ai-validation-panel border-2 border-blue-200 rounded-lg p-4 mb-6">
            <div className="flex items-center mb-4">
                <span className="text-lg">🤖</span>
                <h4 className="text-lg font-semibold ml-2">AI Suggestions - Please Review & Validate</h4>
            </div>
            
            <div className="bg-yellow-50 border border-yellow-200 rounded p-3 mb-4">
                <p className="text-sm text-yellow-800">
                    ⚠️ AI-generated information should be verified. Please review each field and make corrections as needed.
                </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <label className="block text-sm font-medium mb-1">Sun Requirements</label>
                    <select
                        value={editedData.sun_requirements}
                        onChange={(e) => setEditedData({...editedData, sun_requirements: e.target.value})}
                        className="w-full p-2 border rounded-md"
                    >
                        <option value="FullSun">Full Sun (6+ hours)</option>
                        <option value="PartialSun">Partial Sun (4-6 hours)</option>
                        <option value="PartialShade">Partial Shade (2-4 hours)</option>
                        <option value="FullShade">Full Shade (&lt;2 hours)</option>
                    </select>
                </div>
                
                <div>
                    <label className="block text-sm font-medium mb-1">Preferred Daily Hours</label>
                    <input
                        type="number"
                        min="0"
                        max="12"
                        step="0.5"
                        value={editedData.preferred_hours}
                        onChange={(e) => setEditedData({...editedData, preferred_hours: parseFloat(e.target.value)})}
                        className="w-full p-2 border rounded-md"
                    />
                </div>
                
                {/* Additional form fields for all plant characteristics */}
            </div>

            <div className="mt-4">
                <label className="block text-sm font-medium mb-1">Validation Notes (Optional)</label>
                <textarea
                    value={validationNotes}
                    onChange={(e) => setValidationNotes(e.target.value)}
                    className="w-full p-2 border rounded-md"
                    rows={3}
                    placeholder="Add any corrections you made or additional notes..."
                />
            </div>

            <div className="flex gap-4 mt-6">
                <button
                    onClick={() => onValidate({...editedData, validation_notes: validationNotes})}
                    className="bg-green-500 text-white px-6 py-2 rounded-md hover:bg-green-600"
                >
                    ✅ Validate & Add to Database
                </button>
                <button
                    onClick={onReject}
                    className="bg-red-500 text-white px-6 py-2 rounded-md hover:bg-red-600"
                >
                    ❌ Reject & Enter Manually
                </button>
            </div>
        </div>
    );
};
```

#### Step 2c: Frontend - AI Configuration Settings
**File**: `src/components/AISettings.tsx`
```typescript
interface AISettingsProps {
    config: AIAssistanceConfig | null;
    onConfigUpdate: (config: AIAssistanceConfig) => void;
}

export const AISettings: React.FC<AISettingsProps> = ({ config, onConfigUpdate }) => {
    const [provider, setProvider] = useState<AIProvider>(config?.provider || 'OpenAI');
    const [apiKey, setApiKey] = useState(config?.api_key || '');
    const [model, setModel] = useState(config?.model || '');
    const [enabled, setEnabled] = useState(config?.enabled || false);

    const providerModels = {
        OpenAI: ['gpt-4', 'gpt-3.5-turbo'],
        Anthropic: ['claude-3-sonnet-20240229', 'claude-3-haiku-20240307'],
        Google: ['gemini-pro', 'gemini-pro-vision'],
    };

    return (
        <div className="ai-settings p-6 bg-gray-50 rounded-lg">
            <h3 className="text-lg font-bold mb-4">🤖 AI Assistant Settings</h3>
            
            <div className="space-y-4">
                <div>
                    <label className="flex items-center mb-2">
                        <input
                            type="checkbox"
                            checked={enabled}
                            onChange={(e) => setEnabled(e.target.checked)}
                            className="mr-2"
                        />
                        Enable AI assistance for plant data entry
                    </label>
                </div>

                {enabled && (
                    <>
                        <div>
                            <label className="block text-sm font-medium mb-1">AI Provider</label>
                            <select
                                value={provider}
                                onChange={(e) => {
                                    const newProvider = e.target.value as AIProvider;
                                    setProvider(newProvider);
                                    setModel(providerModels[newProvider]?.[0] || '');
                                }}
                                className="w-full p-2 border rounded-md"
                            >
                                <option value="OpenAI">OpenAI (ChatGPT)</option>
                                <option value="Anthropic">Anthropic (Claude)</option>
                                <option value="Google">Google (Gemini)</option>
                            </select>
                        </div>

                        <div>
                            <label className="block text-sm font-medium mb-1">Model</label>
                            <select
                                value={model}
                                onChange={(e) => setModel(e.target.value)}
                                className="w-full p-2 border rounded-md"
                            >
                                {providerModels[provider]?.map(modelName => (
                                    <option key={modelName} value={modelName}>{modelName}</option>
                                ))}
                            </select>
                        </div>

                        <div>
                            <label className="block text-sm font-medium mb-1">API Key</label>
                            <input
                                type="password"
                                value={apiKey}
                                onChange={(e) => setApiKey(e.target.value)}
                                className="w-full p-2 border rounded-md"
                                placeholder="Enter your API key..."
                            />
                            <p className="text-xs text-gray-600 mt-1">
                                Your API key is stored locally and never shared.
                            </p>
                        </div>
                    </>
                )}
            </div>

            <button
                onClick={() => onConfigUpdate({ provider, api_key: apiKey, model, enabled })}
                className="mt-4 bg-blue-500 text-white px-6 py-2 rounded-md hover:bg-blue-600"
            >
                Save AI Settings
            </button>
        </div>
    );
};
```

#### Step 3a: Web Architecture - Database Service
**File**: Backend API server (separate from Tauri app)
```rust
// Plant database service for web deployment
// PostgreSQL/MySQL backend with REST API
// User authentication and contribution tracking
// Community validation system
// Data synchronization with desktop clients
```

#### Step 3b: Community Features
**File**: `src/components/CommunityDatabase.tsx`
```typescript
// Browse community-contributed plants
// Rate and review plant entries
// Report inaccurate information
// Follow favorite contributors
// Export plant collections
// Community leaderboard for contributors
```

**Technical Constraints**:
- Maintain backwards compatibility with existing shadow analysis
- Use existing component patterns from App.tsx for consistency
- Secure API key storage (encrypted local storage)
- Rate limiting for AI API calls to prevent abuse
- Offline fallback when AI services unavailable
- Data validation to prevent malicious entries

**Success Criteria**:
- AI-assisted entry reduces data entry time by 80%
- Database grows to 1000+ plants through community contributions
- 95% of AI suggestions validated as accurate by users
- Web application supports 10,000+ concurrent users
- Synchronization between desktop and web versions < 5 seconds

**Testing Strategy**:
- Validate AI responses against botanical references
- Load testing with multiple AI providers
- Security testing for API key handling
- User acceptance testing with gardening community
- Performance testing with large plant databases

---

## TIER 2 FEATURES

### 4. Garden Design Canvas & Dynamic Shadow Modeling

**Context**: This is the flagship feature that transforms static shadow analysis into dynamic garden simulation. Users design complete garden layouts with planters, trellises, and plants that grow over time, automatically updating the DSM and shadow calculations. This creates a living digital twin of their garden evolution.

**Prerequisites**:
- Plant recommendation system with growth data complete
- Interactive map functionality established ✓
- Shadow analysis engine optimized for real-time recalculation
- DSM modification capabilities implemented

**Implementation Steps**:

#### Step 1a: Backend - Advanced Garden Infrastructure Data Structures
**File**: `src-tauri/src/types.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenDesign {
    pub design_id: String,
    pub name: String,
    pub created_date: chrono::DateTime<chrono::Utc>,
    pub garden_elements: Vec<GardenElement>,
    pub planter_templates: Vec<PlanterTemplate>,
    pub bounds: RasterBounds,
    pub base_dsm: Array2<f32>,           // Original terrain DSM
    pub modified_dsm: Array2<f32>,       // DSM with garden infrastructure
    pub growth_timeline: Vec<GrowthSnapshot>,
    pub simulation_settings: SimulationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenElement {
    pub element_id: String,
    pub element_type: GardenElementType,
    pub geometry: ElementGeometry,
    pub height_profile: HeightProfile,   // How element affects DSM
    pub plant_instances: Vec<PlantInstance>,
    pub created_date: chrono::NaiveDate,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GardenElementType {
    // Planting Infrastructure
    ElevatedBed { 
        height_cm: u32,
        material: PlanterMaterial,
        drainage: bool,
        irrigation: bool,
    },
    GroundBed { 
        depth_cm: u32,           // Below grade depth
        soil_amendment: SoilType,
        mulch_depth_cm: u32,
    },
    Container { 
        pot_type: ContainerType,
        volume_liters: u32,
        mobility: ContainerMobility,
    },
    
    // Support Structures  
    Trellis {
        height_cm: u32,
        width_cm: u32,
        material: TrellisMaterial,
        orientation: f64,        // Degrees from north
        mesh_density: MeshDensity,
    },
    Arbor {
        height_cm: u32,
        arch_width_cm: u32,
        arch_depth_cm: u32,
        covered_percentage: f32, // 0.0-1.0 for vine coverage
    },
    Stakes {
        height_cm: u32,
        diameter_cm: u32,
        quantity: u32,
    },
    
    // Permanent Structures
    Greenhouse {
        dimensions: (u32, u32, u32), // length, width, height in cm
        material: GreenhouseMaterial,
        ventilation: VentilationType,
    },
    Shed {
        dimensions: (u32, u32, u32),
        material: String,
    },
    
    // Landscape Features
    Path { 
        width_cm: u32,
        material: PathMaterial,
        elevation_change_cm: i32, // Can be negative for sunken paths
    },
    WaterFeature {
        feature_type: WaterFeatureType,
        dimensions: (u32, u32, u32),
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantInstance {
    pub plant_id: u32,
    pub position: (f64, f64),            // Relative to element
    pub planting_date: chrono::NaiveDate,
    pub current_growth_stage: GrowthStage,
    pub health_status: PlantHealth,
    pub custom_growth_rate: Option<f32>, // Multiplier for standard growth
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthSnapshot {
    pub date: chrono::NaiveDate,
    pub plant_heights: std::collections::HashMap<String, f32>, // element_id -> height
    pub canopy_spreads: std::collections::HashMap<String, f32>,
    pub dsm_modifications: Array2<f32>,
    pub shadow_impact_summary: ShadowImpactSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanterTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub thumbnail: Option<String>,       // Base64 encoded image
    pub dimensions: (u32, u32, u32),     // length, width, height
    pub element_type: GardenElementType,
    pub recommended_plants: Vec<PlantRecommendation>,
    pub material_cost_estimate: Option<f32>,
    pub difficulty_level: DifficultyLevel,
    pub tags: Vec<String>,               // "raised-bed", "vertical", "compact", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    TerracottaPot,
    PlasticPot,
    FiberglassPot,
    WoodenBox,
    MetalRaisedBed,
    FabricPot,
    Custom { material: String, shape: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrellisMaterial {
    Wood,
    Metal,
    Bamboo,
    Plastic,
    Wire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshDensity {
    Fine,    // 2-5cm spacing - for small vines, peas
    Medium,  // 5-15cm spacing - for beans, cucumbers  
    Coarse,  // 15-30cm spacing - for large vines, squash
}
```

#### Step 1b: Backend - Dynamic DSM Modification Engine
**File**: `src-tauri/src/dsm_modifier.rs` (new module)
```rust
use crate::types::*;
use ndarray::{Array2, Axis};

pub struct DSMModifier {
    base_dsm: Array2<f32>,
    current_dsm: Array2<f32>,
    resolution: f64,
    modification_history: Vec<DSMModification>,
}

impl DSMModifier {
    pub fn new(base_dsm: Array2<f32>, resolution: f64) -> Self {
        let current_dsm = base_dsm.clone();
        Self {
            base_dsm,
            current_dsm,
            resolution,
            modification_history: Vec::new(),
        }
    }
    
    pub fn apply_garden_design(&mut self, design: &GardenDesign) -> Result<Array2<f32>, DSMError> {
        // Reset to base DSM
        self.current_dsm = self.base_dsm.clone();
        
        // Apply each garden element
        for element in &design.garden_elements {
            self.apply_element_to_dsm(element)?;
        }
        
        Ok(self.current_dsm.clone())
    }
    
    fn apply_element_to_dsm(&mut self, element: &GardenElement) -> Result<(), DSMError> {
        match &element.element_type {
            GardenElementType::ElevatedBed { height_cm, .. } => {
                self.add_elevated_structure(&element.geometry, *height_cm as f32 / 100.0)
            },
            GardenElementType::Trellis { height_cm, width_cm, orientation, mesh_density, .. } => {
                self.add_trellis_structure(&element.geometry, *height_cm as f32 / 100.0, *width_cm as f32 / 100.0, *orientation, mesh_density)
            },
            GardenElementType::Greenhouse { dimensions, .. } => {
                let (length, width, height) = *dimensions;
                self.add_greenhouse_structure(&element.geometry, height as f32 / 100.0)
            },
            // Add plant growth effects
            _ => self.add_plant_canopy_effects(element),
        }
    }
    
    fn add_elevated_structure(&mut self, geometry: &ElementGeometry, height_m: f32) -> Result<(), DSMError> {
        let affected_cells = self.geometry_to_cells(geometry)?;
        
        for (row, col) in affected_cells {
            if row < self.current_dsm.nrows() && col < self.current_dsm.ncols() {
                self.current_dsm[[row, col]] += height_m;
            }
        }
        
        self.modification_history.push(DSMModification {
            modification_type: ModificationType::ElevatedStructure,
            height_change: height_m,
            affected_area: geometry.clone(),
            timestamp: chrono::Utc::now(),
        });
        
        Ok(())
    }
    
    fn add_trellis_structure(
        &mut self, 
        geometry: &ElementGeometry, 
        height_m: f32,
        width_m: f32,
        orientation: f64,
        mesh_density: &MeshDensity
    ) -> Result<(), DSMError> {
        let shadow_reduction_factor = match mesh_density {
            MeshDensity::Fine => 0.8,    // 80% shadow reduction
            MeshDensity::Medium => 0.6,  // 60% shadow reduction  
            MeshDensity::Coarse => 0.4,  // 40% shadow reduction
        };
        
        // Create partial height effect - trellises don't block light completely
        let effective_height = height_m * shadow_reduction_factor;
        
        let affected_cells = self.geometry_to_cells(geometry)?;
        for (row, col) in affected_cells {
            if row < self.current_dsm.nrows() && col < self.current_dsm.ncols() {
                self.current_dsm[[row, col]] += effective_height;
            }
        }
        
        Ok(())
    }
    
    fn add_plant_canopy_effects(&mut self, element: &GardenElement) -> Result<(), DSMError> {
        for plant_instance in &element.plant_instances {
            // Get plant characteristics from database
            let plant_data = self.get_plant_data(plant_instance.plant_id)?;
            
            // Calculate current plant dimensions based on growth stage and time
            let current_height = self.calculate_current_plant_height(plant_instance, &plant_data);
            let current_spread = self.calculate_current_plant_spread(plant_instance, &plant_data);
            
            // Apply plant canopy to DSM
            self.add_plant_canopy_to_dsm(
                &plant_instance.position, 
                current_height, 
                current_spread,
                &plant_data.canopy_density
            )?;
        }
        Ok(())
    }
    
    fn calculate_current_plant_height(&self, instance: &PlantInstance, plant_data: &PlantRequirements) -> f32 {
        let days_since_planting = (chrono::Utc::now().date_naive() - instance.planting_date).num_days() as f32;
        let growth_rate = instance.custom_growth_rate.unwrap_or(1.0);
        
        // Simple linear growth model (can be enhanced with S-curves, seasonal effects)
        let mature_height_cm = plant_data.mature_dimensions.height_cm.1 as f32; // Use max height
        let days_to_maturity = plant_data.harvest_info
            .as_ref()
            .map(|h| h.days_to_maturity as f32)
            .unwrap_or(120.0); // Default 4 months
        
        let growth_fraction = (days_since_planting / days_to_maturity * growth_rate).min(1.0);
        (mature_height_cm * growth_fraction) / 100.0 // Convert to meters
    }
    
    pub fn simulate_growth_over_time(
        &mut self,
        design: &GardenDesign,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        interval_days: u32,
    ) -> Vec<GrowthSnapshot> {
        let mut snapshots = Vec::new();
        let mut current_date = start_date;
        
        while current_date <= end_date {
            // Calculate plant growth for this date
            let mut snapshot = GrowthSnapshot {
                date: current_date,
                plant_heights: std::collections::HashMap::new(),
                canopy_spreads: std::collections::HashMap::new(),
                dsm_modifications: Array2::zeros((0, 0)),
                shadow_impact_summary: ShadowImpactSummary::default(),
            };
            
            // Update DSM for this growth stage
            self.apply_garden_design_at_date(design, current_date);
            snapshot.dsm_modifications = self.current_dsm.clone();
            
            snapshots.push(snapshot);
            current_date += chrono::Duration::days(interval_days as i64);
        }
        
        snapshots
    }
}
```

#### Step 1c: Backend - Planter Template Library
**File**: `src-tauri/src/planter_library.rs` (new module)
```rust
pub struct PlanterLibrary {
    templates: Vec<PlanterTemplate>,
    user_templates: Vec<PlanterTemplate>,
}

impl PlanterLibrary {
    pub fn get_default_templates() -> Vec<PlanterTemplate> {
        vec![
            // Standard Raised Beds
            PlanterTemplate {
                template_id: "raised_4x8".to_string(),
                name: "Standard 4x8 Raised Bed".to_string(),
                description: "Classic rectangular raised bed for vegetables".to_string(),
                thumbnail: None,
                dimensions: (240, 120, 30), // 8ft x 4ft x 12in
                element_type: GardenElementType::ElevatedBed {
                    height_cm: 30,
                    material: PlanterMaterial::Cedar,
                    drainage: true,
                    irrigation: false,
                },
                recommended_plants: vec![
                    // Tomatoes, peppers, lettuce, herbs
                ],
                material_cost_estimate: Some(150.0),
                difficulty_level: DifficultyLevel::Beginner,
                tags: vec!["raised-bed".to_string(), "vegetable".to_string(), "beginner".to_string()],
            },
            
            // Vertical Growing Systems
            PlanterTemplate {
                template_id: "trellis_6ft".to_string(),
                name: "6ft Bean Trellis".to_string(), 
                description: "Vertical trellis for climbing beans and peas".to_string(),
                thumbnail: None,
                dimensions: (180, 30, 180), // 6ft wide, 1ft deep, 6ft high
                element_type: GardenElementType::Trellis {
                    height_cm: 180,
                    width_cm: 180,
                    material: TrellisMaterial::Wood,
                    orientation: 0.0,
                    mesh_density: MeshDensity::Medium,
                },
                recommended_plants: vec![
                    // Beans, peas, cucumbers
                ],
                material_cost_estimate: Some(75.0),
                difficulty_level: DifficultyLevel::Intermediate,
                tags: vec!["vertical".to_string(), "trellis".to_string(), "climbing".to_string()],
            },
            
            // Container Gardens
            PlanterTemplate {
                template_id: "large_pot_cluster".to_string(),
                name: "Large Pot Cluster".to_string(),
                description: "Collection of large pots for patio gardening".to_string(),
                thumbnail: None,
                dimensions: (120, 120, 40), // 4ft x 4ft arrangement, 16in pots
                element_type: GardenElementType::Container {
                    pot_type: ContainerType::TerracottaPot,
                    volume_liters: 75,
                    mobility: ContainerMobility::Stationary,
                },
                recommended_plants: vec![
                    // Compact vegetables, herbs, flowers
                ],
                material_cost_estimate: Some(200.0),
                difficulty_level: DifficultyLevel::Beginner,
                tags: vec!["container".to_string(), "patio".to_string(), "mobile".to_string()],
            },
        ]
    }
}
```

#### Step 2a: Frontend - Advanced Planter Design Tool
**File**: `src/components/PlanterDesigner.tsx`
```typescript
interface PlanterDesignerProps {
    onTemplateCreated: (template: PlanterTemplate) => void;
    onTemplateSelected: (template: PlanterTemplate) => void;
    existingTemplates: PlanterTemplate[];
}

export const PlanterDesigner: React.FC<PlanterDesignerProps> = ({ 
    onTemplateCreated, 
    onTemplateSelected, 
    existingTemplates 
}) => {
    const [designMode, setDesignMode] = useState<'browse' | 'create' | 'customize'>('browse');
    const [currentTemplate, setCurrentTemplate] = useState<PlanterTemplate | null>(null);
    const [dimensions, setDimensions] = useState({ length: 240, width: 120, height: 30 });
    const [material, setMaterial] = useState<PlanterMaterial>('Cedar');
    const [plantingLayout, setPlantingLayout] = useState<PlantLayoutGrid>({ rows: 2, cols: 4, spacing: 30 });

    return (
        <div className="planter-designer p-6 bg-white rounded-lg shadow-lg">
            <div className="designer-header mb-6">
                <h3 className="text-xl font-bold mb-4">🏗️ Planter Design Studio</h3>
                
                <div className="mode-selector flex gap-2 mb-4">
                    <button
                        onClick={() => setDesignMode('browse')}
                        className={`px-4 py-2 rounded-md ${designMode === 'browse' ? 'bg-blue-500 text-white' : 'bg-gray-200'}`}
                    >
                        📚 Browse Templates
                    </button>
                    <button
                        onClick={() => setDesignMode('create')}
                        className={`px-4 py-2 rounded-md ${designMode === 'create' ? 'bg-blue-500 text-white' : 'bg-gray-200'}`}
                    >
                        🎨 Create Custom
                    </button>
                    <button
                        onClick={() => setDesignMode('customize')}
                        className={`px-4 py-2 rounded-md ${designMode === 'customize' ? 'bg-blue-500 text-white' : 'bg-gray-200'}`}
                        disabled={!currentTemplate}
                    >
                        ⚙️ Customize Selected
                    </button>
                </div>
            </div>

            {designMode === 'browse' && (
                <TemplateGallery
                    templates={existingTemplates}
                    onSelect={setCurrentTemplate}
                    selectedTemplate={currentTemplate}
                />
            )}

            {designMode === 'create' && (
                <CustomPlanterBuilder
                    dimensions={dimensions}
                    onDimensionsChange={setDimensions}
                    material={material}
                    onMaterialChange={setMaterial}
                    plantingLayout={plantingLayout}
                    onLayoutChange={setPlantingLayout}
                    onSaveTemplate={(template) => {
                        onTemplateCreated(template);
                        setDesignMode('browse');
                    }}
                />
            )}

            {designMode === 'customize' && currentTemplate && (
                <TemplateCustomizer
                    baseTemplate={currentTemplate}
                    onCustomizationComplete={(customized) => {
                        onTemplateCreated(customized);
                        setDesignMode('browse');
                    }}
                />
            )}
        </div>
    );
};
```

#### Step 2b: Frontend - Drag & Drop Garden Canvas
**File**: `src/components/GardenCanvas.tsx`
```typescript
interface GardenCanvasProps {
    shadowData: AllSummaryData | null;
    selectedTemplate: PlanterTemplate | null;
    onDesignUpdate: (design: GardenDesign) => void;
    onShadowRecalculationRequested: () => void;
}

export const GardenCanvas: React.FC<GardenCanvasProps> = ({
    shadowData,
    selectedTemplate,
    onDesignUpdate,
    onShadowRecalculationRequested
}) => {
    const [gardenElements, setGardenElements] = useState<GardenElement[]>([]);
    const [isDragging, setIsDragging] = useState(false);
    const [draggedElement, setDraggedElement] = useState<GardenElement | null>(null);
    const [showGrowthSimulation, setShowGrowthSimulation] = useState(false);
    const [simulationDate, setSimulationDate] = useState<Date>(new Date());

    const handleElementDrop = (position: L.LatLng, template: PlanterTemplate) => {
        if (!selectedTemplate) return;

        const newElement: GardenElement = {
            element_id: `element_${Date.now()}`,
            element_type: template.element_type,
            geometry: {
                Rectangle: {
                    bounds: {
                        min_lat: position.lat - 0.0001,
                        max_lat: position.lat + 0.0001,
                        min_lon: position.lng - 0.0001,
                        max_lon: position.lng + 0.0001,
                    }
                }
            },
            height_profile: calculateHeightProfile(template),
            plant_instances: [],
            created_date: new Date().toISOString().split('T')[0],
            notes: '',
        };

        setGardenElements(prev => [...prev, newElement]);
        
        // Trigger DSM recalculation
        onShadowRecalculationRequested();
    };

    return (
        <div className="garden-canvas relative">
            {/* Enhanced Leaflet Map with Drop Zones */}
            <div className="map-container relative h-96 rounded-lg overflow-hidden">
                <LeafletMapWithDropZones
                    shadowData={shadowData}
                    gardenElements={gardenElements}
                    onElementDrop={handleElementDrop}
                    onElementSelect={setSelectedElement}
                    simulationDate={showGrowthSimulation ? simulationDate : null}
                />
                
                {/* Growth Simulation Controls */}
                <div className="simulation-controls absolute top-4 right-4 bg-white p-4 rounded-lg shadow-lg">
                    <label className="flex items-center mb-2">
                        <input
                            type="checkbox"
                            checked={showGrowthSimulation}
                            onChange={(e) => setShowGrowthSimulation(e.target.checked)}
                            className="mr-2"
                        />
                        🌱 Growth Simulation
                    </label>
                    
                    {showGrowthSimulation && (
                        <div>
                            <label className="block text-sm font-medium mb-1">Simulation Date</label>
                            <input
                                type="date"
                                value={simulationDate.toISOString().split('T')[0]}
                                onChange={(e) => setSimulationDate(new Date(e.target.value))}
                                className="w-full p-2 border rounded-md text-sm"
                            />
                            <button
                                onClick={onShadowRecalculationRequested}
                                className="mt-2 w-full bg-green-500 text-white px-3 py-1 rounded-md text-sm hover:bg-green-600"
                            >
                                🔄 Update Shadows
                            </button>
                        </div>
                    )}
                </div>
            </div>

            {/* Element Properties Panel */}
            {selectedElement && (
                <ElementPropertiesPanel
                    element={selectedElement}
                    plantDatabase={plantDatabase}
                    onElementUpdate={(updated) => {
                        setGardenElements(prev => 
                            prev.map(el => el.element_id === updated.element_id ? updated : el)
                        );
                        onShadowRecalculationRequested();
                    }}
                />
            )}
        </div>
    );
};
```

#### Step 2c: Frontend - Growth Timeline Visualization
**File**: `src/components/GrowthTimeline.tsx`
```typescript
interface GrowthTimelineProps {
    gardenDesign: GardenDesign;
    onDateSelected: (date: Date) => void;
    currentDate: Date;
}

export const GrowthTimeline: React.FC<GrowthTimelineProps> = ({
    gardenDesign,
    onDateSelected,
    currentDate
}) => {
    const [timelineData, setTimelineData] = useState<GrowthSnapshot[]>([]);
    const [playbackSpeed, setPlaybackSpeed] = useState(1);
    const [isPlaying, setIsPlaying] = useState(false);

    useEffect(() => {
        // Generate growth timeline data
        const generateTimeline = async () => {
            const timeline = await invoke<GrowthSnapshot[]>('simulate_garden_growth', {
                design: gardenDesign,
                startDate: new Date().toISOString().split('T')[0],
                endDate: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
                intervalDays: 7, // Weekly snapshots
            });
            setTimelineData(timeline);
        };
        
        if (gardenDesign.garden_elements.length > 0) {
            generateTimeline();
        }
    }, [gardenDesign]);

    return (
        <div className="growth-timeline p-4 bg-gray-50 rounded-lg">
            <div className="timeline-header flex items-center justify-between mb-4">
                <h4 className="text-lg font-semibold">🌱 Garden Growth Timeline</h4>
                
                <div className="playback-controls flex items-center gap-2">
                    <button
                        onClick={() => setIsPlaying(!isPlaying)}
                        className="bg-green-500 text-white px-4 py-2 rounded-md hover:bg-green-600"
                    >
                        {isPlaying ? '⏸️ Pause' : '▶️ Play'}
                    </button>
                    
                    <select
                        value={playbackSpeed}
                        onChange={(e) => setPlaybackSpeed(Number(e.target.value))}
                        className="p-2 border rounded-md"
                    >
                        <option value={0.5}>0.5x Speed</option>
                        <option value={1}>1x Speed</option>
                        <option value={2}>2x Speed</option>
                        <option value={4}>4x Speed</option>
                    </select>
                </div>
            </div>

            {/* Timeline Slider */}
            <div className="timeline-slider mb-4">
                <input
                    type="range"
                    min={0}
                    max={timelineData.length - 1}
                    value={timelineData.findIndex(snapshot => 
                        snapshot.date === currentDate.toISOString().split('T')[0]
                    )}
                    onChange={(e) => {
                        const snapshot = timelineData[Number(e.target.value)];
                        if (snapshot) {
                            onDateSelected(new Date(snapshot.date));
                        }
                    }}
                    className="w-full"
                />
                
                <div className="flex justify-between text-sm text-gray-600 mt-1">
                    <span>🌱 Planting</span>
                    <span>🌿 Growing</span>
                    <span>🌾 Harvest</span>
                    <span>❄️ Winter</span>
                </div>
            </div>

            {/* Growth Statistics */}
            <div className="growth-stats grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="stat-card bg-white p-3 rounded-md">
                    <div className="text-sm text-gray-600">Total Height</div>
                    <div className="text-lg font-bold text-green-600">
                        {getCurrentTotalHeight(timelineData, currentDate).toFixed(1)}m
                    </div>
                </div>
                
                <div className="stat-card bg-white p-3 rounded-md">
                    <div className="text-sm text-gray-600">Shadow Impact</div>
                    <div className="text-lg font-bold text-orange-600">
                        {getCurrentShadowImpact(timelineData, currentDate).toFixed(1)}%
                    </div>
                </div>
                
                <div className="stat-card bg-white p-3 rounded-md">
                    <div className="text-sm text-gray-600">Harvest Ready</div>
                    <div className="text-lg font-bold text-purple-600">
                        {getHarvestReadyCount(timelineData, currentDate)} plants
                    </div>
                </div>
                
                <div className="stat-card bg-white p-3 rounded-md">
                    <div className="text-sm text-gray-600">Garden Health</div>
                    <div className="text-lg font-bold text-blue-600">
                        {getGardenHealthScore(timelineData, currentDate)}%
                    </div>
                </div>
            </div>
        </div>
    );
};
```

#### Step 3a: Integration - Real-time Shadow Recalculation
**File**: Enhanced `src-tauri/src/shadow_engine.rs`
```rust
impl ShadowEngine {
    pub fn calculate_shadows_with_modified_dsm(
        &mut self,
        modified_dsm: Array2<f32>,
        garden_design: &GardenDesign,
        simulation_date: Option<chrono::NaiveDate>
    ) -> Result<ShadowResult, ShadowError> {
        // Update DSM with garden design
        self.dsm = modified_dsm;
        
        // If simulation date provided, calculate plant growth effects
        if let Some(date) = simulation_date {
            self.dsm = self.apply_plant_growth_to_dsm(&garden_design, date);
        }
        
        // Run standard shadow calculation with modified DSM
        self.calculate_shadows()
    }
    
    fn apply_plant_growth_to_dsm(
        &self,
        garden_design: &GardenDesign,
        target_date: chrono::NaiveDate
    ) -> Array2<f32> {
        let mut growth_modified_dsm = self.dsm.clone();
        
        for element in &garden_design.garden_elements {
            for plant_instance in &element.plant_instances {
                // Calculate plant size at target date
                let plant_data = self.get_plant_data(plant_instance.plant_id).unwrap();
                let current_height = self.calculate_plant_height_at_date(
                    plant_instance, 
                    &plant_data, 
                    target_date
                );
                let current_spread = self.calculate_plant_spread_at_date(
                    plant_instance, 
                    &plant_data, 
                    target_date
                );
                
                // Apply plant canopy to DSM
                self.add_plant_to_dsm(
                    &mut growth_modified_dsm,
                    plant_instance,
                    current_height,
                    current_spread
                );
            }
        }
        
        growth_modified_dsm
    }
}
```

**Technical Constraints**:
- Maintain real-time performance for DSM modifications (< 2 seconds for recalculation)
- Use existing shadow calculation engine with minimal modifications
- Follow established Leaflet patterns for drag-and-drop functionality
- Integrate with plant database for growth characteristics
- Preserve undo/redo functionality for design changes

**Success Criteria**:
- Template library with 20+ pre-designed planters
- Drag-and-drop placement with snap-to-grid precision
- Real-time shadow updates when elements are added/moved
- Growth simulation showing plant development over 12 months
- Export functionality for construction/planting plans
- Performance: Handle 50+ garden elements without lag

**Testing Strategy**:
- Validate DSM modifications produce accurate shadow changes
- Test growth models against real plant development timelines
- User testing with landscape designers and master gardeners
- Performance testing with complex garden designs
- Cross-validation with physical measurements where possible

---

### 5. Advanced Solar Engineering

**Context**: Professional solar installers need sophisticated tools for complex installations, including string optimization, inverter matching, and detailed shading analysis. This builds on the basic solar optimizer with engineering-grade features.

**Prerequisites**:
- Basic solar panel optimizer complete
- Seasonal analysis available
- Performance optimization patterns established

**Implementation Steps**:

#### Step 1a: Backend - Advanced Solar Models
**File**: `src-tauri/src/advanced_solar.rs` (new module)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarSystemDesign {
    pub system_id: String,
    pub panel_arrays: Vec<PanelArray>,
    pub inverters: Vec<InverterConfig>,
    pub electrical_design: ElectricalDesign,
    pub shading_analysis: DetailedShadingAnalysis,
    pub performance_model: PerformanceModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelArray {
    pub array_id: String,
    pub panels: Vec<PanelLocation>,
    pub tilt: f64,
    pub azimuth: f64,
    pub string_configuration: StringConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringConfiguration {
    pub panels_per_string: u32,
    pub strings_per_inverter: u32,
    pub bypass_diodes: bool,
    pub power_optimizers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedShadingAnalysis {
    pub hourly_shading_factors: Array3<f32>,  // [time, row, col]
    pub string_mismatch_losses: Vec<f32>,
    pub inverter_clipping_losses: Vec<f32>,
    pub temperature_derating: Array3<f32>,
}
```

#### Step 1b: Backend - Professional Calculation Engine
**File**: `src-tauri/src/advanced_solar.rs`
```rust
impl AdvancedSolarEngine {
    pub fn optimize_string_layout(&self, available_area: &Polygon<f64>) -> StringOptimization {
        // Optimize panel grouping to minimize shading mismatch losses
        // Calculate optimal inverter sizing
        // Analyze bypass diode effectiveness
        // Consider power optimizer placement
    }
    
    pub fn calculate_financial_metrics(&self, system: &SolarSystemDesign) -> FinancialAnalysis {
        // Detailed cost analysis including:
        // - Equipment costs (panels, inverters, mounting, electrical)
        // - Installation labor costs
        // - Permitting and inspection fees
        // - Utility interconnection costs
        // - Net present value and IRR calculations
        // - Payback period with degradation factors
    }
    
    pub fn generate_professional_report(&self, system: &SolarSystemDesign) -> Report {
        // Generate detailed engineering report
        // Include shading analysis charts
        // Performance predictions with confidence intervals
        // Compliance with local electrical codes
    }
}
```

#### Step 2a: Frontend - Professional Design Interface
**File**: `src/components/ProfessionalSolarDesigner.tsx`
```typescript
// Advanced UI for solar professionals
// Multiple panel array support with individual optimization
// String layout visualization with electrical schematic overlay
// Inverter selection wizard with compatibility checking
// Code compliance verification (NEC, local amendments)

interface ProfessionalDesignerProps {
    shadowData: AllSummaryData | null;
    systemRequirements: SystemRequirements;
    onDesignComplete: (design: SolarSystemDesign) => void;
}
```

#### Step 2b: Frontend - Engineering Analysis Dashboard
**File**: `src/components/EngineeringDashboard.tsx`
```typescript
// Detailed performance charts and graphs
// Shading loss analysis with time-series visualization
// String performance comparison
// Financial analysis with sensitivity analysis
// Professional report generation and export
```

#### Step 2c: Frontend - Code Compliance Checker
**File**: `src/components/CodeComplianceChecker.tsx`
```typescript
// Automated checking against electrical codes
// Setback requirements verification
// Fire safety pathway validation  
// Structural load calculations
// Utility interconnection requirements
```

**Technical Constraints**:
- Use existing high-performance calculation patterns from shadow_engine.rs
- Integrate with professional solar design standards (NEC, IEC)
- Maintain calculation accuracy for engineering applications
- Export to industry-standard formats (AutoCAD, SketchUp)

**Success Criteria**:
- Engineering-grade accuracy for shading loss calculations
- String optimization reduces mismatch losses by 10-15%
- Professional report generation meets industry standards
- Integration with common solar design software workflows

---

### 6. Seasonal Planting Calendar

**Context**: Gardeners need intelligent scheduling for planting, caring for, and harvesting plants based on local conditions and shadow patterns. This creates a dynamic calendar that adapts to specific site conditions.

**Prerequisites**:
- Plant database with requirements complete
- Seasonal shadow analysis available  
- Garden design tools established

**Implementation Steps**:

#### Step 1a: Backend - Planting Schedule Engine
**File**: `src-tauri/src/planting_scheduler.rs` (new module)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantingSchedule {
    pub schedule_id: String,
    pub garden_design: GardenDesign,
    pub planting_events: Vec<PlantingEvent>,
    pub care_reminders: Vec<CareReminder>,
    pub harvest_predictions: Vec<HarvestEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantingEvent {
    pub event_id: String,
    pub plant_id: u32,
    pub location: ElementGeometry,
    pub optimal_date: chrono::NaiveDate,
    pub date_range: (chrono::NaiveDate, chrono::NaiveDate), // earliest-latest
    pub succession_interval: Option<u32>, // days between successive plantings
    pub companion_plants: Vec<u32>,
    pub soil_preparation_tasks: Vec<String>,
}

pub struct PlantingScheduler {
    climate_data: ClimateData,
    shadow_analysis: SeasonalAnalysis,
    plant_database: Vec<PlantRequirements>,
}

impl PlantingScheduler {
    pub fn generate_optimal_schedule(&self, garden_design: &GardenDesign) -> PlantingSchedule {
        // For each plant in the design:
        // 1. Analyze light conditions at specific location
        // 2. Determine optimal planting windows based on:
        //    - Frost dates
        //    - Soil temperature requirements  
        //    - Day length requirements
        //    - Heat tolerance
        // 3. Schedule succession plantings for continuous harvest
        // 4. Plan companion planting arrangements
        // 5. Schedule care tasks (fertilizing, pruning, etc.)
    }
}
```

#### Step 1b: Backend - Climate Integration
**File**: `src-tauri/src/climate_data.rs` (new module)
```rust
#[derive(Debug, Clone)]
pub struct ClimateData {
    pub location: (f64, f64), // lat, lon
    pub hardiness_zone: String,
    pub average_first_frost: chrono::NaiveDate,
    pub average_last_frost: chrono::NaiveDate,
    pub growing_season_length: u32, // days
    pub monthly_temperatures: [f32; 12], // average temps
}

impl ClimateData {
    pub fn from_coordinates(lat: f64, lon: f64) -> Result<Self, ClimateError> {
        // Use USDA hardiness zone data
        // Integrate with weather services for local frost dates
        // Calculate growing season based on temperature thresholds
    }
}
```

#### Step 2a: Frontend - Interactive Planting Calendar
**File**: `src/components/PlantingCalendar.tsx`
```typescript
// Calendar view with planting events overlaid
// Color coding by plant type and task type
// Drag-and-drop rescheduling with validation
// Integration with garden design for location-specific scheduling

interface CalendarEvent {
    id: string;
    title: string;
    date: Date;
    type: 'planting' | 'care' | 'harvest';
    plant_id?: number;
    location?: string;
    description: string;
    priority: 'low' | 'medium' | 'high';
}
```

#### Step 2b: Frontend - Task Management System
**File**: `src/components/GardenTaskManager.tsx`
```typescript
// Todo list for garden tasks with scheduling
// Weather-based task postponement
// Photo documentation for progress tracking
// Integration with mobile reminders/notifications

interface GardenTask {
    task_id: string;
    title: string;
    description: string;
    due_date: Date;
    plant_id?: number;
    location: string;
    estimated_duration: number; // minutes
    required_tools: string[];
    weather_dependent: boolean;
    completed: boolean;
    completion_notes?: string;
    completion_photos?: string[];
}
```

#### Step 2c: Frontend - Harvest Tracking & Planning
**File**: `src/components/HarvestPlanner.tsx`
```typescript
// Harvest predictions based on planting dates and variety
// Yield tracking and analysis
// Preservation/storage recommendations
// Next season planning based on current year performance

interface HarvestRecord {
    harvest_id: string;
    plant_id: number;
    harvest_date: Date;
    quantity: number;
    quality_rating: number; // 1-5 scale
    notes: string;
    weather_conditions: string;
    storage_method: string;
}
```

**Technical Constraints**:
- Integrate with existing garden design and plant database systems
- Use established date/time handling patterns from shadow calculation
- Follow existing UI patterns for calendar components
- Consider offline functionality for field use

**Success Criteria**:
- Generate location-specific planting schedules automatically
- Accurate frost date predictions and growing season calculation
- Succession planting optimization for continuous harvests
- Integration between planning, task management, and harvest tracking

---

## TIER 2.5 FEATURES (ADVANCED CAPABILITIES)

### 4.5. Advanced Shadow Mode with Light Transmittance & Remote Sensing

**Context**: Professional applications require sophisticated shadow modeling that accounts for partial light transmission through different materials. This advanced mode integrates remote sensing data to automatically classify surface materials and their light transmittance properties, enabling research-grade shadow accuracy for forestry, agriculture, and urban planning applications.

**Prerequisites**:
- Garden Design Canvas implemented
- High-performance shadow calculation engine established
- GDAL integration for additional raster formats
- Scientific validation framework

**Implementation Steps**:

#### Step 1a: Backend - Advanced Light Physics Engine
**File**: `src-tauri/src/advanced_shadow_physics.rs` (new module)
```rust
use crate::types::*;
use ndarray::Array2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTransmittance {
    pub material_id: String,
    pub visible_transmittance: f32,     // 0.0-1.0, fraction of visible light transmitted
    pub nir_transmittance: f32,         // Near-infrared transmittance
    pub seasonal_variation: SeasonalTransmittance,
    pub angular_dependence: AngularTransmittance,
    pub spectral_properties: SpectralProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalTransmittance {
    pub spring: f32,      // Leaf-out period
    pub summer: f32,      // Full canopy
    pub fall: f32,        // Senescence
    pub winter: f32,      // Bare branches
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngularTransmittance {
    pub overhead: f32,       // Sun directly above (90° elevation)
    pub oblique: f32,        // Sun at angle (45° elevation)  
    pub low_angle: f32,      // Sun near horizon (15° elevation)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialType {
    // Vegetation
    DeciduousTree { species: TreeSpecies, health: VegetationHealth },
    ConiferousTree { species: TreeSpecies, density: CanopyDensity },
    Shrub { height_class: HeightClass, density: CanopyDensity },
    Grass { type_: GrassType, height_cm: f32 },
    Crop { crop_type: CropType, growth_stage: CropStage },
    
    // Structures  
    Building { material: BuildingMaterial, condition: StructureCondition },
    Fence { material: FenceMaterial, transparency: f32 },
    ShadeCloth { density_percent: u8 }, // 30%, 50%, 70% shade cloth
    Greenhouse { panel_type: GreenhousePanelType },
    
    // Natural Features
    Water { depth_class: WaterDepth, turbidity: WaterClarity },
    Rock { rock_type: RockType },
    Soil { moisture: SoilMoisture, cover: f32 }, // fraction covered by vegetation
}

pub struct AdvancedShadowEngine {
    base_shadow_engine: ShadowEngine,
    material_map: Array2<MaterialType>,
    transmittance_lookup: std::collections::HashMap<String, MaterialTransmittance>,
    remote_sensing_data: RemoteSensingData,
}

impl AdvancedShadowEngine {
    pub fn calculate_advanced_shadows(
        &self,
        sun_position: (f64, f64), // azimuth, elevation
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<AdvancedShadowResult, ShadowError> {
        let (n_rows, n_cols) = self.material_map.dim();
        let mut shadow_fraction = Array2::<f32>::zeros((n_rows, n_cols));
        let mut transmitted_light = Array2::<f32>::ones((n_rows, n_cols)); // Start with full light
        
        // Enhanced ray marching with light attenuation
        for row in 0..n_rows {
            for col in 0..n_cols {
                let light_level = self.calculate_transmitted_light_at_cell(
                    row, col, sun_position, timestamp
                )?;
                
                shadow_fraction[[row, col]] = 1.0 - light_level;
                transmitted_light[[row, col]] = light_level;
            }
        }
        
        Ok(AdvancedShadowResult {
            shadow_fraction,
            transmitted_light,
            spectral_analysis: self.calculate_spectral_distribution(&transmitted_light),
            material_contributions: self.analyze_material_contributions(),
        })
    }
    
    fn calculate_transmitted_light_at_cell(
        &self,
        row: usize,
        col: usize,
        sun_position: (f64, f64),
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<f32, ShadowError> {
        let cell_height = self.base_shadow_engine.dsm[[row, col]];
        let (azimuth, elevation) = sun_position;
        let sun_dir = self.sun_direction(azimuth, elevation);
        
        // Enhanced ray marching with light attenuation
        let mut current_x = col as f64;
        let mut current_y = row as f64;
        let mut current_z = cell_height as f64;
        let mut accumulated_light_loss = 0.0f32;
        
        let step_size = 0.5;
        let max_distance = self.base_shadow_engine.config.buffer_meters / self.base_shadow_engine.resolution;
        let mut distance = 0.0;
        
        while distance < max_distance {
            current_x += sun_dir.0 * step_size;
            current_y -= sun_dir.1 * step_size;
            current_z += sun_dir.2 * step_size;
            distance += step_size;
            
            // Check bounds
            if current_x < 0.0 || current_y < 0.0 
                || current_x >= n_cols as f64 - 1.0 
                || current_y >= n_rows as f64 - 1.0 {
                break;
            }
            
            let terrain_height = self.base_shadow_engine.interpolate_height(current_y, current_x);
            
            if terrain_height > current_z as f32 {
                // Ray intersects with surface - calculate light attenuation
                let material = self.get_material_at_position(current_y as usize, current_x as usize);
                let transmittance = self.get_material_transmittance(&material, elevation, timestamp);
                
                // Accumulate light loss through this material
                accumulated_light_loss += 1.0 - transmittance;
                
                // If completely blocked, stop ray marching
                if accumulated_light_loss >= 1.0 {
                    return Ok(0.0);
                }
            }
        }
        
        // Return remaining light after all attenuations
        Ok((1.0 - accumulated_light_loss).max(0.0))
    }
    
    fn get_material_transmittance(
        &self,
        material: &MaterialType,
        sun_elevation: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> f32 {
        let base_transmittance = match material {
            MaterialType::DeciduousTree { species, health } => {
                self.calculate_tree_transmittance(species, health, timestamp)
            },
            MaterialType::ConiferousTree { species, density } => {
                self.calculate_conifer_transmittance(species, density)
            },
            MaterialType::Shrub { density, .. } => {
                match density {
                    CanopyDensity::Sparse => 0.7,
                    CanopyDensity::Medium => 0.4,
                    CanopyDensity::Dense => 0.2,
                }
            },
            MaterialType::ShadeCloth { density_percent } => {
                1.0 - (*density_percent as f32 / 100.0)
            },
            MaterialType::Greenhouse { panel_type } => {
                match panel_type {
                    GreenhousePanelType::Glass => 0.9,
                    GreenhousePanelType::Polycarbonate => 0.85,
                    GreenhousePanelType::Plastic => 0.8,
                }
            },
            MaterialType::Building { .. } => 0.0, // Opaque
            _ => 0.0, // Default to opaque for unknown materials
        };
        
        // Apply angular correction for sun elevation
        self.apply_angular_correction(base_transmittance, sun_elevation)
    }
    
    fn calculate_tree_transmittance(
        &self,
        species: &TreeSpecies,
        health: &VegetationHealth,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> f32 {
        let base_transmittance = match species {
            TreeSpecies::Oak => 0.15,        // Dense canopy
            TreeSpecies::Maple => 0.20,      // Medium density
            TreeSpecies::Birch => 0.35,      // Light canopy
            TreeSpecies::Willow => 0.25,     // Drooping branches
            _ => 0.25, // Default
        };
        
        // Seasonal adjustment for deciduous trees
        let seasonal_factor = self.get_seasonal_transmittance_factor(timestamp);
        
        // Health adjustment
        let health_factor = match health {
            VegetationHealth::Excellent => 1.0,
            VegetationHealth::Good => 1.2,
            VegetationHealth::Fair => 1.5,
            VegetationHealth::Poor => 2.0,
            VegetationHealth::Stressed => 3.0,
        };
        
        (base_transmittance * seasonal_factor * health_factor).min(1.0)
    }
    
    fn get_seasonal_transmittance_factor(&self, timestamp: chrono::DateTime<chrono::Utc>) -> f32 {
        let day_of_year = timestamp.ordinal() as f32;
        
        // Simplified seasonal model (Northern Hemisphere)
        match day_of_year {
            d if d < 80.0 => 0.8,   // Winter (Jan-Mar): bare branches
            d if d < 120.0 => 0.6,  // Spring (Mar-Apr): leaf-out
            d if d < 260.0 => 1.0,  // Summer (May-Sep): full canopy  
            d if d < 320.0 => 0.7,  // Fall (Sep-Nov): senescence
            _ => 0.8,               // Late fall/winter
        }
    }
}
```

#### Step 1b: Backend - Remote Sensing Data Integration
**File**: `src-tauri/src/remote_sensing.rs` (new module)  
```rust
use gdal::Dataset;
use ndarray::Array2;

#[derive(Debug, Clone)]
pub struct RemoteSensingData {
    pub nir_orthophoto: Option<Array2<f32>>,
    pub red_band: Option<Array2<f32>>,
    pub lulc_raster: Option<Array2<u8>>,
    pub lidar_chm: Option<Array2<f32>>,        // Canopy Height Model
    pub multispectral: Option<MultispectralData>,
}

#[derive(Debug, Clone)]
pub struct MultispectralData {
    pub bands: std::collections::HashMap<String, Array2<f32>>,
    pub wavelengths: Vec<f32>,
    pub acquisition_date: chrono::NaiveDate,
}

pub struct MaterialClassifier {
    classification_rules: ClassificationRuleset,
    training_data: Option<TrainingDataset>,
}

impl MaterialClassifier {
    pub fn classify_materials_from_remote_sensing(
        &self,
        remote_sensing_data: &RemoteSensingData,
    ) -> Result<Array2<MaterialType>, ClassificationError> {
        let (n_rows, n_cols) = self.get_dimensions(remote_sensing_data)?;
        let mut material_map = Array2::from_elem((n_rows, n_cols), MaterialType::Soil { 
            moisture: SoilMoisture::Medium, 
            cover: 0.0 
        });
        
        for row in 0..n_rows {
            for col in 0..n_cols {
                let material = self.classify_pixel(remote_sensing_data, row, col)?;
                material_map[[row, col]] = material;
            }
        }
        
        Ok(material_map)
    }
    
    fn classify_pixel(
        &self,
        data: &RemoteSensingData,
        row: usize,
        col: usize,
    ) -> Result<MaterialType, ClassificationError> {
        let mut classification_scores = std::collections::HashMap::new();
        
        // NDVI-based vegetation classification
        if let (Some(nir), Some(red)) = (&data.nir_orthophoto, &data.red_band) {
            let ndvi = self.calculate_ndvi(nir[[row, col]], red[[row, col]]);
            
            match ndvi {
                n if n > 0.8 => classification_scores.insert("dense_vegetation", 0.9),
                n if n > 0.6 => classification_scores.insert("moderate_vegetation", 0.8),
                n if n > 0.3 => classification_scores.insert("sparse_vegetation", 0.7),
                _ => classification_scores.insert("bare_ground", 0.8),
            };
        }
        
        // LULC classification
        if let Some(lulc) = &data.lulc_raster {
            let lulc_class = lulc[[row, col]];
            let material_type = self.lulc_to_material_type(lulc_class);
            classification_scores.insert(&format!("lulc_{}", lulc_class), 0.85);
        }
        
        // LiDAR height analysis
        if let Some(chm) = &data.lidar_chm {
            let height = chm[[row, col]];
            match height {
                h if h > 15.0 => classification_scores.insert("tall_tree", 0.9),
                h if h > 5.0 => classification_scores.insert("medium_tree", 0.8),
                h if h > 1.0 => classification_scores.insert("shrub", 0.7),
                _ => classification_scores.insert("ground_cover", 0.6),
            };
        }
        
        // Select best classification
        self.resolve_classification_conflicts(&classification_scores)
    }
    
    fn calculate_ndvi(&self, nir: f32, red: f32) -> f32 {
        if nir + red == 0.0 {
            0.0
        } else {
            (nir - red) / (nir + red)
        }
    }
}
```

#### Step 2a: Frontend - Remote Sensing Data Upload Interface
**File**: `src/components/RemoteSensingUpload.tsx`
```typescript
interface RemoteSensingUploadProps {
    onDataUploaded: (data: RemoteSensingDataset) => void;
    currentMode: 'basic' | 'advanced';
    onModeChange: (mode: 'basic' | 'advanced') => void;
}

export const RemoteSensingUpload: React.FC<RemoteSensingUploadProps> = ({
    onDataUploaded,
    currentMode,
    onModeChange
}) => {
    const [uploadedFiles, setUploadedFiles] = useState<{
        nirOrthophoto?: File;
        redBand?: File;
        lulcRaster?: File;
        lidarChm?: File;
        multispectral?: File[];
    }>({});

    return (
        <div className="remote-sensing-upload p-6 bg-gradient-to-br from-blue-50 to-green-50 rounded-lg">
            <div className="header mb-6">
                <div className="flex items-center justify-between mb-4">
                    <h3 className="text-xl font-bold text-gray-800">🛰️ Advanced Shadow Mode</h3>
                    
                    <div className="mode-toggle">
                        <label className="inline-flex items-center">
                            <input
                                type="checkbox"
                                checked={currentMode === 'advanced'}
                                onChange={(e) => onModeChange(e.target.checked ? 'advanced' : 'basic')}
                                className="sr-only"
                            />
                            <div className={`relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full ${
                                currentMode === 'advanced' ? 'bg-green-500' : 'bg-gray-300'
                            }`}>
                                <div className={`absolute left-1 top-1 w-4 h-4 transition duration-200 ease-in-out transform bg-white rounded-full ${
                                    currentMode === 'advanced' ? 'translate-x-6' : ''
                                }`} />
                            </div>
                            <span className="ml-3 text-sm font-medium">
                                {currentMode === 'advanced' ? 'Advanced Mode' : 'Basic Mode'}
                            </span>
                        </label>
                    </div>
                </div>
                
                <p className="text-sm text-gray-600">
                    {currentMode === 'basic' 
                        ? "Standard shadow analysis treating all surfaces as opaque barriers"
                        : "Research-grade analysis with material transmittance and remote sensing integration"
                    }
                </p>
            </div>

            {currentMode === 'advanced' && (
                <div className="advanced-options">
                    <div className="data-requirements mb-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
                        <h4 className="font-semibold text-blue-800 mb-2">📊 Optional Remote Sensing Data</h4>
                        <p className="text-sm text-blue-700 mb-3">
                            Upload additional data for enhanced material classification and light transmittance modeling:
                        </p>
                        
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div className="requirement-item">
                                <div className="flex items-center mb-1">
                                    <span className="text-green-600 mr-2">🌿</span>
                                    <span className="font-medium text-sm">NIR Orthophoto</span>
                                </div>
                                <p className="text-xs text-gray-600">
                                    Near-infrared imagery for vegetation health and density analysis (NDVI calculation)
                                </p>
                            </div>
                            
                            <div className="requirement-item">
                                <div className="flex items-center mb-1">
                                    <span className="text-brown-600 mr-2">🏞️</span>
                                    <span className="font-medium text-sm">LULC Raster</span>
                                </div>
                                <p className="text-xs text-gray-600">
                                    Land use/land cover classification for automatic material property assignment
                                </p>
                            </div>
                            
                            <div className="requirement-item">
                                <div className="flex items-center mb-1">
                                    <span className="text-purple-600 mr-2">⚡</span>
                                    <span className="font-medium text-sm">LiDAR CHM</span>
                                </div>
                                <p className="text-xs text-gray-600">
                                    Canopy Height Model for 3D vegetation structure and density estimation
                                </p>
                            </div>
                            
                            <div className="requirement-item">
                                <div className="flex items-center mb-1">
                                    <span className="text-rainbow mr-2">🌈</span>
                                    <span className="font-medium text-sm">Multispectral</span>
                                </div>
                                <p className="text-xs text-gray-600">
                                    Multiple band imagery for precise material identification and condition assessment
                                </p>
                            </div>
                        </div>
                    </div>

                    <div className="file-upload-grid grid grid-cols-1 md:grid-cols-2 gap-4">
                        <FileUploadCard
                            title="NIR Orthophoto"
                            description="Near-infrared band (.tif)"
                            acceptedFormats=".tif,.tiff"
                            icon="🌿"
                            onFileSelect={(file) => setUploadedFiles(prev => ({...prev, nirOrthophoto: file}))}
                            selectedFile={uploadedFiles.nirOrthophoto}
                        />
                        
                        <FileUploadCard
                            title="Red Band"
                            description="Visible red spectrum (.tif)"
                            acceptedFormats=".tif,.tiff"
                            icon="🔴"
                            onFileSelect={(file) => setUploadedFiles(prev => ({...prev, redBand: file}))}
                            selectedFile={uploadedFiles.redBand}
                        />
                        
                        <FileUploadCard
                            title="LULC Classification"
                            description="Land cover raster (.tif)"
                            acceptedFormats=".tif,.tiff"
                            icon="🏞️"
                            onFileSelect={(file) => setUploadedFiles(prev => ({...prev, lulcRaster: file}))}
                            selectedFile={uploadedFiles.lulcRaster}
                        />
                        
                        <FileUploadCard
                            title="LiDAR CHM"
                            description="Canopy height model (.tif)"
                            acceptedFormats=".tif,.tiff,.las,.laz"
                            icon="⚡"
                            onFileSelect={(file) => setUploadedFiles(prev => ({...prev, lidarChm: file}))}
                            selectedFile={uploadedFiles.lidarChm}
                        />
                    </div>

                    <div className="processing-options mt-6 p-4 bg-gray-50 rounded-lg">
                        <h4 className="font-semibold mb-3">🔬 Processing Options</h4>
                        
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium mb-1">Material Classification Method</label>
                                <select className="w-full p-2 border rounded-md text-sm">
                                    <option value="rule_based">Rule-based Classification</option>
                                    <option value="machine_learning">Machine Learning (requires training data)</option>
                                    <option value="hybrid">Hybrid Approach</option>
                                </select>
                            </div>
                            
                            <div>
                                <label className="block text-sm font-medium mb-1">Transmittance Model</label>
                                <select className="w-full p-2 border rounded-md text-sm">
                                    <option value="simplified">Simplified (Fast)</option>
                                    <option value="seasonal">Seasonal Variation</option>
                                    <option value="spectral">Full Spectral (Slow)</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
```

#### Step 2b: Frontend - Advanced Shadow Analysis Dashboard  
**File**: `src/components/AdvancedShadowDashboard.tsx`
```typescript
interface AdvancedShadowResult {
    shadow_fraction: number[][];
    transmitted_light: number[][];
    spectral_analysis: SpectralAnalysis;
    material_contributions: MaterialContribution[];
}

export const AdvancedShadowDashboard: React.FC<{result: AdvancedShadowResult}> = ({ result }) => {
    return (
        <div className="advanced-dashboard space-y-6">
            {/* Light Transmittance Visualization */}
            <div className="transmittance-map bg-white p-6 rounded-lg shadow-lg">
                <h4 className="text-lg font-semibold mb-4">💡 Light Transmittance Analysis</h4>
                
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div>
                        <h5 className="font-medium mb-2">Traditional Shadow Map</h5>
                        <div className="shadow-map-container relative">
                            {/* Binary shadow visualization */}
                            <ShadowMapVisualization data={result.shadow_fraction} mode="binary" />
                        </div>
                    </div>
                    
                    <div>
                        <h5 className="font-medium mb-2">Advanced Transmittance Map</h5>
                        <div className="transmittance-map-container relative">
                            {/* Graduated light levels */}
                            <ShadowMapVisualization data={result.transmitted_light} mode="transmittance" />
                        </div>
                    </div>
                </div>
                
                <div className="legend mt-4 flex justify-center">
                    <div className="flex items-center space-x-4 text-sm">
                        <div className="flex items-center">
                            <div className="w-4 h-4 bg-black mr-2"></div>
                            <span>0% Light (Complete Shadow)</span>
                        </div>
                        <div className="flex items-center">
                            <div className="w-4 h-4 bg-gray-600 mr-2"></div>
                            <span>25% Light (Dense Canopy)</span>
                        </div>
                        <div className="flex items-center">
                            <div className="w-4 h-4 bg-gray-400 mr-2"></div>
                            <span>50% Light (Moderate Shade)</span>
                        </div>
                        <div className="flex items-center">
                            <div className="w-4 h-4 bg-gray-200 mr-2"></div>
                            <span>75% Light (Light Shade)</span>
                        </div>
                        <div className="flex items-center">
                            <div className="w-4 h-4 bg-white border mr-2"></div>
                            <span>100% Light (Full Sun)</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Material Contribution Analysis */}
            <div className="material-analysis bg-white p-6 rounded-lg shadow-lg">
                <h4 className="text-lg font-semibold mb-4">🌳 Material Shadow Contributions</h4>
                
                <div className="contribution-chart">
                    {result.material_contributions.map((contribution, idx) => (
                        <div key={idx} className="contribution-item flex items-center justify-between py-2 border-b">
                            <div className="flex items-center">
                                <div className={`w-4 h-4 rounded mr-3`} style={{backgroundColor: contribution.color}}></div>
                                <span className="font-medium">{contribution.material_name}</span>
                            </div>
                            <div className="text-right">
                                <div className="font-semibold">{contribution.shadow_percentage.toFixed(1)}%</div>
                                <div className="text-sm text-gray-600">
                                    Avg Transmittance: {(contribution.avg_transmittance * 100).toFixed(0)}%
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Spectral Analysis */}
            <div className="spectral-analysis bg-white p-6 rounded-lg shadow-lg">
                <h4 className="text-lg font-semibold mb-4">🌈 Spectral Light Analysis</h4>
                
                <div className="spectral-charts grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h5 className="font-medium mb-2">Light Quality Distribution</h5>
                        <SpectralChart data={result.spectral_analysis.wavelength_distribution} />
                    </div>
                    
                    <div>
                        <h5 className="font-medium mb-2">Photosynthetically Active Radiation (PAR)</h5>
                        <PARChart data={result.spectral_analysis.par_levels} />
                    </div>
                </div>
            </div>
        </div>
    );
};
```

#### Step 3a: Integration - Data Processing Pipeline
**File**: `src-tauri/src/advanced_processing_pipeline.rs` (new module)
```rust
pub struct AdvancedProcessingPipeline {
    material_classifier: MaterialClassifier,
    transmittance_engine: AdvancedShadowEngine,
    data_validator: RemoteSensingValidator,
}

impl AdvancedProcessingPipeline {
    pub async fn process_advanced_shadow_analysis(
        &self,
        dtm: Array2<f32>,
        dsm: Array2<f32>,
        remote_sensing_data: RemoteSensingData,
        config: Config,
    ) -> Result<AdvancedShadowResult, ProcessingError> {
        // Step 1: Validate remote sensing data compatibility
        self.data_validator.validate_data_compatibility(&dtm, &dsm, &remote_sensing_data)?;
        
        // Step 2: Classify materials from remote sensing data
        let material_map = self.material_classifier
            .classify_materials_from_remote_sensing(&remote_sensing_data)?;
        
        // Step 3: Generate transmittance lookup table
        let transmittance_lookup = self.generate_transmittance_lookup(&material_map)?;
        
        // Step 4: Run advanced shadow analysis
        let mut advanced_engine = AdvancedShadowEngine::new(
            dtm, dsm, material_map, transmittance_lookup, remote_sensing_data
        );
        
        advanced_engine.calculate_advanced_shadows_time_series(&config).await
    }
}
```

**Technical Constraints**:
- Maintain backward compatibility with basic shadow mode
- Process remote sensing data using GDAL for format compatibility
- Optimize performance for real-time analysis with large datasets
- Implement scientific validation against field measurements
- Support common remote sensing formats (GeoTIFF, HDF5, NetCDF)

**Success Criteria**:
- 15-25% improvement in shadow accuracy over basic mode for vegetated areas
- Support for 10+ remote sensing data formats
- Processing performance: <30 seconds for 1km² area with full dataset
- Scientific validation: R² > 0.85 correlation with field measurements
- Professional adoption by forestry/agriculture consultants

**Testing Strategy**:
- Validate against research datasets with ground truth measurements
- Cross-validation with established vegetation indices (NDVI, LAI, fCover)
- Performance testing with large-scale remote sensing datasets
- User acceptance testing with GIS/remote sensing professionals
- Integration testing with common GIS software workflows

## TIER 3 FEATURES

### 7. Microclimate Modeling

**Context**: Advanced users (researchers, landscape architects) need detailed microclimate predictions based on shadow patterns, topography, and vegetation. This extends beyond simple shadow calculation to comprehensive environmental modeling.

**Prerequisites**:
- All shadow analysis features complete
- High-performance calculation engine established
- Advanced visualization capabilities

**Implementation Steps**:

#### Step 1a: Backend - Microclimate Physics Engine
**File**: `src-tauri/src/microclimate.rs` (new module)
```rust
#[derive(Debug, Clone)]
pub struct MicroclimateModel {
    pub temperature_gradients: Array3<f32>, // [time, row, col]
    pub humidity_patterns: Array3<f32>,
    pub wind_flow_vectors: Array4<f32>,     // [time, row, col, (u,v)]
    pub evapotranspiration_rates: Array3<f32>,
    pub soil_moisture_patterns: Array3<f32>,
    pub heat_island_effects: Array3<f32>,
}

pub struct MicroclimateEngine {
    dtm: Array2<f32>,
    dsm: Array2<f32>,
    shadow_data: ShadowResult,
    vegetation_data: VegetationMap,
    weather_forcing: WeatherData,
}

impl MicroclimateEngine {
    pub fn calculate_temperature_patterns(&self) -> Array3<f32> {
        // Model temperature variations based on:
        // - Solar irradiance (from shadow analysis)
        // - Thermal mass of surfaces (concrete vs grass vs water)
        // - Elevation effects (lapse rate)
        // - Vegetation cooling (evapotranspiration)
        // - Urban heat island effects
    }
    
    pub fn model_wind_patterns(&self) -> Array4<f32> {
        // Simplified CFD modeling:
        // - Topographic wind effects
        // - Building wake effects  
        // - Vegetation wind breaks
        // - Thermal circulation patterns
    }
}
```

#### Step 1b: Backend - Advanced Data Integration
**File**: `src-tauri/src/environmental_data.rs` (new module)
```rust
// Integration with meteorological data sources
// Soil type databases
// Vegetation classification from satellite imagery
// Urban surface material mapping
```

#### Step 2a: Frontend - Advanced Visualization
**File**: `src/components/MicroclimateVisualization.tsx`
```typescript
// 3D visualization of temperature/humidity gradients
// Animated wind flow vectors
// Time-lapse microclimate evolution
// Comparative analysis tools (before/after development scenarios)
```

---

### 8. Collaborative Planning

**Context**: Multiple users (family members, community gardens, professional teams) need to collaborate on garden/solar designs with shared access, version control, and communication tools.

**Prerequisites**:
- Garden design tools complete
- User authentication system
- Cloud storage integration

**Implementation Steps**:
- Multi-user design sharing
- Real-time collaborative editing
- Version control for designs  
- Comment/annotation system
- Permission management (view/edit/admin)
- Integration with external sharing platforms

---

### 9. Mobile Field App

**Context**: On-site analysis capabilities for professionals and advanced users, with GPS integration, camera functionality, and real-time shadow prediction.

**Prerequisites**:
- Core calculation engine proven
- Mobile development framework selected
- Cloud sync capabilities

**Implementation Steps**:
- Mobile app architecture (React Native/Flutter/Native)
- GPS integration for automatic location detection
- Camera overlay for real-time shadow prediction
- Offline calculation capabilities
- Field data collection tools
- Integration with desktop version

---

## Cross-Feature Technical Considerations

### Performance Targets
- **Shadow Calculations**: < 10 seconds for 40k cells × 6k timestamps
- **UI Responsiveness**: < 300ms for all interactions
- **Data Loading**: < 3 seconds for complex datasets
- **Memory Usage**: < 2GB RAM for typical analyses

### Data Persistence Strategy
- **Local Storage**: SQLite for user designs and plant database
- **Cloud Sync**: Optional cloud storage for design sharing
- **Export Formats**: JSON, CSV, GeoTIFF, PDF, PNG
- **Import Capabilities**: Standard GIS formats, existing design tools

### Cross-Platform Consistency  
- **Desktop**: Primary platform (Windows, macOS, Linux via Tauri)
- **Web**: Browser compatibility for viewing/basic editing
- **Mobile**: Native apps for field use
- **API**: RESTful API for third-party integrations

---

## Development Workflow Guidelines

### Code Quality Standards
- **Rust**: Follow established patterns from shadow_engine.rs
- **TypeScript**: Strict typing, consistent component patterns
- **Testing**: Unit tests for calculations, integration tests for UI
- **Documentation**: Inline code docs, user guides, API documentation

### Feature Development Process
1. **Planning**: Detailed technical specification (this document)
2. **Backend First**: Core calculations and data structures
3. **Frontend Integration**: UI components and user interactions  
4. **Testing**: Automated tests and user acceptance testing
5. **Documentation**: User guides and technical documentation
6. **Deployment**: Staged rollout with feature flags

This roadmap provides a comprehensive foundation for systematic development of the Shadow Calculator into a full-featured ecosystem of shadow analysis applications.