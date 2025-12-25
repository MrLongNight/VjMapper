//! Audio Meter Widget
//!
//! Provides two styles of audio level visualization:
//! - Retro: Analog VU meter with needle and arc scale
//! - Digital: Segmented LED bar

use crate::config::AudioMeterStyle;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Widget};

/// A widget that displays audio levels.
pub struct AudioMeter {
    style: AudioMeterStyle,
    level_db_l: f32,
    level_db_r: f32,
    size: Vec2,
}

impl AudioMeter {
    /// Create a new audio meter
    pub fn new(style: AudioMeterStyle, level_db_l: f32, level_db_r: f32) -> Self {
        Self {
            style,
            level_db_l,
            level_db_r,
            size: Vec2::new(180.0, 40.0), // Default size, can be overridden by layout
        }
    }

    /// Set preferred size
    pub fn desired_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Widget for AudioMeter {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            match self.style {
                AudioMeterStyle::Retro => {
                    draw_retro_stereo_meter(ui, rect, self.level_db_l, self.level_db_r)
                }
                AudioMeterStyle::Digital => {
                    draw_digital_stereo_meter(ui, rect, self.level_db_l, self.level_db_r)
                }
            }
        }

        response
    }
}

/// Helper to draw a Phillips screw head
fn draw_screw(painter: &egui::Painter, center: Pos2, radius: f32) {
    // Screw head
    painter.circle_filled(center, radius, Color32::from_rgb(60, 60, 60)); // Dark metal
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(1.0, Color32::from_rgb(30, 30, 30)),
    );

    // Cross slot (Phillips)
    let slot_w = radius * 0.3;
    let slot_l = radius * 0.7;
    let color = Color32::from_rgb(20, 20, 20); // Deep recess

    painter.rect_filled(
        Rect::from_center_size(center, Vec2::new(slot_w, slot_l * 2.0)),
        1.0,
        color,
    );
    painter.rect_filled(
        Rect::from_center_size(center, Vec2::new(slot_l * 2.0, slot_w)),
        1.0,
        color,
    );
}

/// Helper to draw the mounting frame with screws
fn draw_mounting_frame(ui: &mut egui::Ui, rect: Rect) -> Rect {
    let painter = ui.painter();

    // Outer Frame (Dark bezel)
    painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 35));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(10, 10, 10)));

    // Screws in corners
    let screw_inset = 6.0;
    let screw_radius = 3.5;

    draw_screw(
        painter,
        rect.min + Vec2::new(screw_inset, screw_inset),
        screw_radius,
    );
    draw_screw(
        painter,
        rect.max - Vec2::new(screw_inset, screw_inset),
        screw_radius,
    );
    draw_screw(
        painter,
        Pos2::new(rect.min.x + screw_inset, rect.max.y - screw_inset),
        screw_radius,
    );
    draw_screw(
        painter,
        Pos2::new(rect.max.x - screw_inset, rect.min.y + screw_inset),
        screw_radius,
    );

    // Return inner rect for content
    rect.shrink(10.0) // Bezel thickness
}

/// Draws an analog retro VU meter (Stereo)
fn draw_retro_stereo_meter(ui: &mut egui::Ui, rect: Rect, db_l: f32, db_r: f32) {
    let inner_rect = draw_mounting_frame(ui, rect);

    // Split inner rect for Left/Right meters
    let split_width = inner_rect.width() / 2.0;
    let rect_l = Rect::from_min_size(
        inner_rect.min,
        Vec2::new(split_width - 2.0, inner_rect.height()),
    );
    let rect_r = Rect::from_min_size(
        inner_rect.min + Vec2::new(split_width + 2.0, 0.0),
        Vec2::new(split_width - 2.0, inner_rect.height()),
    );

    draw_single_vu(ui, rect_l, db_l, "L");
    draw_single_vu(ui, rect_r, db_r, "R");
}

fn draw_single_vu(ui: &mut egui::Ui, rect: Rect, db: f32, label_ch: &str) {
    let painter = ui.painter();

    // Background - Warm off-white / yellowish
    painter.rect_filled(rect, 2.0, Color32::from_rgb(235, 232, 215));

    // Inner shadow at top
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 4.0)),
        0.0,
        Color32::from_black_alpha(30),
    );

    // Calculate geometry (Pivot below)
    let offset_down = rect.height() * 0.9;
    let center = rect.center_bottom() + Vec2::new(0.0, offset_down);
    // Radius adjusted to fit
    let radius = (center.y - rect.min.y) - 8.0;

    // Helper to map angle to position
    let angle_to_pos = |angle_deg: f32, r: f32| -> Pos2 {
        let rad = (angle_deg - 90.0).to_radians();
        center + Vec2::new(rad.cos() * r, rad.sin() * r)
    };

    let start_angle = -35.0;
    let end_angle = 35.0;
    let zero_angle = 20.0;

    // Red zone background
    let red_zone_points: Vec<Pos2> = (0..=10)
        .map(|i| {
            let t = i as f32 / 10.0;
            let angle = zero_angle + t * (end_angle - zero_angle);
            angle_to_pos(angle, radius * 0.85) // Slightly shorter than ticks
        })
        .collect();

    if red_zone_points.len() >= 2 {
        painter.add(egui::Shape::line(
            red_zone_points,
            Stroke::new(8.0, Color32::from_rgba_premultiplied(200, 60, 60, 80)),
        ));
    }

    // Ticks & Labels
    let ticks = [
        (-20.0, start_angle, "-20"),
        (-10.0, -20.0, "-10"),
        (-5.0, 0.0, "-5"),
        (0.0, zero_angle, "0"),
        (3.0, end_angle, "+3"),
    ];

    for (_val, angle, label) in ticks.iter() {
        let p1 = angle_to_pos(*angle, radius * 0.7);
        let p2 = angle_to_pos(*angle, radius * 0.85);
        painter.line_segment([p1, p2], Stroke::new(1.5, Color32::from_rgb(20, 20, 20)));

        if rect.height() > 30.0 {
            let text_pos = angle_to_pos(*angle, radius * 0.55);
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(20, 20, 20),
            );
        }
    }

    // Channel Label (L/R)
    painter.text(
        rect.center_bottom() - Vec2::new(0.0, rect.height() * 0.15),
        egui::Align2::CENTER_BOTTOM,
        label_ch,
        egui::FontId::proportional(12.0),
        Color32::from_rgb(80, 80, 80),
    );

    // Needle
    let clamped_db = db.clamp(-40.0, 6.0);
    let needle_angle = if clamped_db < -20.0 {
        start_angle - 5.0
    } else if clamped_db < 0.0 {
        start_angle + (clamped_db - -20.0) / 20.0 * (zero_angle - start_angle)
    } else {
        zero_angle + (clamped_db - 0.0) / 3.0 * (end_angle - zero_angle)
    };

    let needle_len = radius * 0.95;
    let tip = angle_to_pos(needle_angle, needle_len);

    // Needle start (intersection with bottom)
    let needle_dir = Vec2::new(
        (needle_angle - 90.0).to_radians().cos(),
        (needle_angle - 90.0).to_radians().sin(),
    );
    let t_bottom = (rect.max.y - center.y) / needle_dir.y;
    let start_pos = center + needle_dir * t_bottom.max(0.0);

    // Needle shadow
    painter.line_segment(
        [start_pos + Vec2::new(1.5, 1.5), tip + Vec2::new(1.5, 1.5)],
        Stroke::new(1.5, Color32::from_black_alpha(40)),
    );
    // Needle body
    painter.line_segment(
        [start_pos, tip],
        Stroke::new(1.2, Color32::from_rgb(180, 30, 30)),
    );

    // Glass Reflection
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.35)),
        2.0,
        Color32::from_white_alpha(20),
    );
}

/// Draws a horizontal segmented digital LED meter (Stereo)
fn draw_digital_stereo_meter(ui: &mut egui::Ui, rect: Rect, db_l: f32, db_r: f32) {
    let inner_rect = draw_mounting_frame(ui, rect);
    let painter = ui.painter();

    // Background (Dark panel behind LEDs)
    painter.rect_filled(inner_rect, 0.0, Color32::from_rgb(10, 10, 12));

    // Layout: Top bar = L, Bottom bar = R, Middle = Scale
    let bar_height = (inner_rect.height() * 0.35).min(12.0);
    let gap = inner_rect.height() - (bar_height * 2.0);

    let rect_l = Rect::from_min_size(
        inner_rect.min + Vec2::new(0.0, inner_rect.height() * 0.1),
        Vec2::new(inner_rect.width(), bar_height),
    );
    let rect_r = Rect::from_min_size(
        inner_rect.max - Vec2::new(inner_rect.width(), bar_height + inner_rect.height() * 0.1),
        Vec2::new(inner_rect.width(), bar_height),
    );

    draw_led_bar(painter, rect_l, db_l);
    draw_led_bar(painter, rect_r, db_r);

    // Draw Center Scale
    if gap > 10.0 {
        let center_y = inner_rect.center().y;
        let font_id = egui::FontId::proportional(9.0);
        let text_color = Color32::from_rgb(150, 150, 150);

        let ticks = [
            (-40.0, "-40"),
            (-20.0, "-20"),
            (-10.0, "-10"),
            (-6.0, "-6"),
            (-3.0, "-3"),
            (0.0, "0"),
        ];

        // Map dB to X position
        // Range: -60 to +3
        let map_x = |db: f32| -> f32 {
            let t = (db - -60.0) / (3.0 - -60.0);
            inner_rect.min.x + t.clamp(0.0, 1.0) * inner_rect.width()
        };

        for (val, txt) in ticks {
            let x = map_x(val);
            painter.text(
                Pos2::new(x, center_y),
                egui::Align2::CENTER_CENTER,
                txt,
                font_id.clone(),
                text_color,
            );
        }
    }
}

fn draw_led_bar(painter: &egui::Painter, rect: Rect, db: f32) {
    let segment_count = 24; // More segments for wider meter
    let padding = 1.0;
    let total_width = rect.width() - (padding * 2.0);
    let segment_width = (total_width - (segment_count as f32 - 1.0)) / segment_count as f32;

    let min_db = -60.0;
    let max_db = 3.0;

    for i in 0..segment_count {
        let t = i as f32 / (segment_count as f32 - 1.0);
        let threshold_db = min_db + t * (max_db - min_db);

        let active = db >= threshold_db;

        // Color logic
        let base_color = if threshold_db >= 0.0 {
            Color32::from_rgb(255, 30, 30) // Red
        } else if threshold_db >= -6.0 {
            Color32::from_rgb(255, 200, 0) // Yellow
        } else {
            Color32::from_rgb(0, 230, 60) // Green
        };

        let color = if active {
            base_color
        } else {
            // Dimmed/Off state
            Color32::from_rgba_premultiplied(
                base_color.r() / 8,
                base_color.g() / 8,
                base_color.b() / 8,
                255,
            )
        };

        let x = rect.min.x + padding + (i as f32 * (segment_width + 1.0));
        let y = rect.min.y;

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(segment_width, rect.height())),
            1.0,
            color,
        );
    }
}
