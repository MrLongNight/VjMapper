
app-title = MapMap - VJ Projection Mapping
menu-file = File
menu-file-load-video = Load Video
menu-file-save-project = Save Project
menu-file-load-project = Load Project
menu-file-open-recent = Open Recent
menu-file-exit = Exit
menu-view = View
menu-help = Help
menu-help-about = About

dashboard-layout-grid = Grid
dashboard-layout-freeform = Freeform
dashboard-columns = Columns:
dashboard-add-widget = ➕ Add Widget
dashboard-add-widget-tooltip = Add a new widget to the dashboard
dashboard-state = State:
dashboard-speed = Speed:
dashboard-loop = Loop
dashboard-audio-analysis = Audio Analysis
dashboard-device = Device
dashboard-no-device = No device selected
dashboard-volume = Volume
dashboard-spectrum = Frequency Spectrum
dashboard-no-audio-data = No audio analysis data available.
dashboard-remove-widget = Remove widget
dashboard-trigger = Trigger
dashboard-rms = RMS
dashboard-peak = Peak

media-browser-title = Media Browser
media-browser-back = Back
media-browser-forward = Forward
media-browser-up = Up
media-browser-refresh = Refresh
media-browser-path = Path:
media-browser-filter = Filter:
media-browser-all = All
media-browser-video = Video
media-browser-image = Image
media-browser-audio = Audio
media-browser-view-grid = Grid
media-browser-view-list = List
media-browser-sort = Sort
media-browser-sort-name = Name
media-browser-sort-type = Type
media-browser-sort-size = Size

panel-effect-chain = Effect Chain
effect-name-color-adjust = Color Adjust
effect-name-blur = Blur
effect-name-chromatic-aberration = Chromatic Aberration
effect-name-edge-detect = Edge Detect
effect-name-glow = Glow
effect-name-kaleidoscope = Kaleidoscope
effect-name-invert = Invert
effect-name-pixelate = Pixelate
effect-name-vignette = Vignette
effect-name-film-grain = Film Grain
effect-name-custom = Custom
effect-add = Add
effect-presets = Presets
effect-clear = Clear
effect-select-type = Select Effect Type
effect-no-effects = No effects in chain
effect-start-tip = Add an effect to get started
effect-intensity = Intensity
param-brightness = Brightness
param-contrast = Contrast
param-saturation = Saturation
param-radius = Radius
param-amount = Amount
param-threshold = Threshold
param-segments = Segments
param-rotation = Rotation
param-pixel-size = Pixel Size
param-softness = Softness
param-speed = Speed
no-parameters = No parameters
effect-save = Save
effect-presets-browser = Preset Browser
effect-search = Search...

# Node Editor
node-blur = Blur
node-glow = Glow
node-color-correction = Color Correction
node-sharpen = Sharpen
node-edge-detect = Edge Detect
node-chroma-key = Chroma Key
node-math-add = Add
node-math-subtract = Subtract
node-math-multiply = Multiply
node-math-divide = Divide
node-math-sin = Sin
node-math-cos = Cos
node-math-abs = Abs
node-math-clamp = Clamp
node-math-lerp = Lerp
node-math-smooth-step = Smooth Step
node-utility-switch = Switch
node-utility-merge = Merge
node-utility-split = Split
node-constant-value = Value
node-constant-vector3 = Vector3
node-constant-color = Color
node-io-input = Input
node-io-output = Output
node-category-effects = Effects
node-category-math = Math
node-category-utility = Utility
node-category-constants = Constants
node-category-io = I/O
node-category-custom = Custom
node-socket-input = Input
node-socket-radius = Radius
node-socket-intensity = Intensity
node-socket-threshold = Threshold
node-socket-hue = Hue
node-socket-saturation = Saturation
node-socket-brightness = Brightness
node-socket-value = Value
node-socket-min = Min
node-socket-max = Max
node-socket-condition = Condition
node-socket-true = True
node-socket-false = False
node-socket-mix = Mix
node-socket-result = Result
node-socket-color = Color
node-socket-vector = Vector
node-add = Add Node:

# Timeline
timeline-play = Play
timeline-pause = Pause
timeline-stop = Stop
timeline-time = Time
timeline-loop = Loop
timeline-snap = Snap
timeline-zoom = Zoom
timeline-curves = Curves
timeline-add-keyframe = Add Keyframe
timeline-no-clip = No animation clip loaded
timeline-window-title = Timeline
curve-editor-title = Curve Editor
curve-editor-editing = Editing
curve-editor-select-track = Select a track to edit curves

# Playback
playback-title = Playback Controls
playback-video = Video Playback
playback-play = Play
playback-pause = Pause
playback-stop = Stop
playback-speed = Speed
playback-mode = Mode:
playback-loop = Loop
playback-play-once = Play Once

# Performance
perf-title = Performance
perf-fps = FPS: { $val }
perf-frametime = Frame Time: { $val } ms
perf-demo = MapMap Phase 0 Demo

# Layers
layers-title = Layers
layers-total = Total Layers: { $count }
layers-bypass = Bypass (B)
layers-solo = Solo (S)
layers-blend-mode = Blend Mode
layers-opacity = Opacity (V)
layers-duplicate = Duplicate
layers-remove = Remove
layers-add = Add Layer
layers-eject-all = Eject All (X)

# Paints
paints-title = Paints
paints-total = Total Paints: { $count }
paints-playing = Playing
paints-loop = Loop
paints-speed = Speed
paints-color = Color
paints-add = Add Paint

# Mappings
mappings-title = Mappings
mappings-total = Total Mappings: { $count }
mappings-solo = Solo
mappings-lock = Lock
mappings-opacity = Opacity
mappings-depth = Depth
mappings-mesh = Mesh: { $type } ({ $count } vertices)
mappings-remove-this = Remove This
mappings-add-quad = Add Quad Mapping

# Transform
transform-title = Transform Controls
transform-phase1 = Phase 1: Transform System
transform-editing = Editing: { $name }
transform-position = Position:
transform-scale = Scale:
transform-width = Width
transform-height = Height
transform-reset-scale = Reset Scale (1:1)
transform-rotation = Rotation (degrees):
transform-reset-rotation = Reset Rotation
transform-anchor = Anchor Point (0-1):
transform-center = Center Anchor (0.5, 0.5)
transform-presets = Resize Presets:
transform-fill = Fill (Cover)
transform-fit = Fit (Contain)
transform-stretch = Stretch (Distort)
transform-original = Original (1:1)
transform-no-selection = Selected layer not found.
transform-no-layer = No layer selected.
transform-select-tip = Click a layer name in the\nLayers panel to select it.

# Master
master-title = Master Controls
master-phase1 = Phase 1: Master Controls
master-composition = Composition:
master-opacity = Master Opacity (M)
master-speed = Master Speed (S)
master-size = Size: { $w }x{ $h }
master-framerate = Frame Rate: { $fps } fps
master-multipliers = Effective Multipliers:
master-help-opacity = All layer opacity × Master Opacity
master-help-speed = All playback speed × Master Speed

# Output
output-title = Outputs
output-config-title = Multi-Output Configuration
output-canvas = Canvas: { $w }x{ $h }
output-total = Outputs: { $count }
output-quick-2x2 = 2x2 Projector Array
output-add = Add Output
output-selected = Selected Output Settings
output-name = Name: { $name }
output-res = Resolution: { $w }x{ $h }
output-region = Canvas Region:
output-blend-status = Edge Blending:
output-calib-status = Color Calibration:
output-tip = Tip:
output-tip-text = Edge Blending and Color Calibration panels open automatically!
output-remove = Remove Output
output-multi-active = Multi-window rendering: ACTIVE
output-multi-help = Output windows are automatically created and synchronized

# Edge Blend
blend-title = Edge Blending
blend-output = Output: { $name }
blend-left = Left Edge
blend-right = Right Edge
blend-top = Top Edge
blend-bottom = Bottom Edge
blend-gamma = Blend Gamma
blend-reset = Reset to Defaults
blend-width = Width
blend-offset = Offset

# Color Calibration
calib-title = Color Calibration
calib-brightness = Brightness
calib-contrast = Contrast
calib-gamma = Gamma (Per Channel)
calib-red = Red Gamma
calib-green = Green Gamma
calib-blue = Blue Gamma
calib-temp = Color Temperature
calib-sat = Saturation
calib-reset = Reset to Defaults

# Oscillator
osc-title = Oscillator Distortion
osc-enable = Enable Effect
osc-presets = Quick Presets:
osc-subtle = Subtle
osc-dramatic = Dramatic
osc-rings = Rings
osc-reset = Reset
osc-dist-params = Distortion Parameters
osc-amount = Amount
osc-scale = Scale
osc-speed = Speed
osc-visual = Visual Overlay
osc-overlay-opacity = Overlay Opacity
osc-color-mode = Color Mode
osc-sim-params = Simulation Parameters
osc-res = Resolution
osc-radius = Kernel Radius
osc-noise = Noise Amount
osc-freq-min = Frequency Min (Hz)
osc-freq-max = Frequency Max (Hz)
osc-coord-mode = Coordinate Mode
osc-phase-init = Phase Init
osc-coupling = Coupling Rings (Advanced)
osc-ring = Ring { $id }
osc-ring-dist = Distance
osc-ring-width = Width
osc-ring-coup = Coupling
osc-ring-reset = Reset Ring
osc-ring-clear = Clear Ring

# Audio
audio-title = Audio Analysis
audio-input = Audio Input
audio-device = Device
audio-spectrum = Frequency Spectrum
