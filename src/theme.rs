use rune::{Color, oklch, oklcha};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThemeKind {
    #[default]
    Obsidian,
    Alabaster,
    Classic95,
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 3] = [
        ThemeKind::Obsidian,
        ThemeKind::Alabaster,
        ThemeKind::Classic95,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeKind::Obsidian => "Obsidian",
            ThemeKind::Alabaster => "Alabaster",
            ThemeKind::Classic95 => "Classic '95",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            ThemeKind::Obsidian => "Dark",
            ThemeKind::Alabaster => "Light",
            ThemeKind::Classic95 => "95",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ThemeKind::Obsidian => ThemeKind::Alabaster,
            ThemeKind::Alabaster => ThemeKind::Classic95,
            ThemeKind::Classic95 => ThemeKind::Obsidian,
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, ThemeKind::Obsidian)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub kind: ThemeKind,
    // Window & outer console chassis
    pub bg_top: Color,
    pub bg_bottom: Color,
    pub chassis: Color,
    pub chassis_top: Color,
    pub chassis_bottom: Color,
    pub chassis_border_top: Color,
    pub chassis_border_bottom: Color,
    pub chassis_shadow: Color,
    pub chassis_shadow_soft: Color,

    // Board recessed tray & wells
    pub tray_bg: Color,
    pub tray_border: Color,
    pub tray_inset_shadow: Color,
    pub tray_inset_highlight: Color,

    // HUD Header strip & digital counter wells
    pub hud_bg: Color,
    pub hud_border: Color,
    pub led_well: Color,
    pub led_well_border: Color,
    pub led_bombs: Color,
    pub led_timer: Color,
    pub led_ghost: Color,
    pub show_led_ghost: bool,

    // Raised hidden tiles
    pub tile_top: Color,
    pub tile_bottom: Color,
    pub tile_border_top: Color,
    pub tile_border_bottom: Color,
    pub tile_bevel_light: Color,
    pub tile_bevel_dark: Color,
    pub tile_shadow: Color,
    pub tile_hover: Color,
    pub tile_press: Color,
    pub tile_depressed: Color,
    pub tile_flagged_tint: Color,

    // Revealed recessed tiles
    pub revealed_a: Color,
    pub revealed_b: Color,
    pub revealed_border: Color,
    pub revealed_hover: Color,

    // Special tile states
    pub blast_top: Color,
    pub blast_bottom: Color,
    pub blast_border: Color,
    pub bomb_revealed_bg: Color,
    pub misflag_bg: Color,
    pub victory_flag_bg: Color,

    // Numbers 1..=8 (index 0 = '1', index 7 = '8')
    pub numbers: [Color; 8],

    // Art colors
    pub bomb_body: Color,
    pub bomb_rim: Color,
    pub bomb_highlight: Color,
    pub bomb_spark: Color,
    pub blast_star: Color,
    pub flag_pole: Color,
    pub flag_base: Color,
    pub flag_cloth: Color,
    pub flag_cloth_bright: Color,
    pub question_ink: Color,
    pub misflag_cross: Color,

    // Mascot medallion
    pub mascot_ring_top: Color,
    pub mascot_ring_bottom: Color,
    pub mascot_face: Color,
    pub mascot_face_shadow: Color,
    pub mascot_ink: Color,
    pub mascot_glasses: Color,
    pub mascot_glint: Color,
    pub mascot_blush: Color,

    // Typography & controls
    pub text: Color,
    pub text_muted: Color,
    pub text_faint: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub control_track: Color,
    pub control_pill: Color,
    pub control_hover: Color,
    pub control_border: Color,
    pub focus_ring: Color,
    pub modal_scrim: Color,
    pub modal_bg: Color,

    // Geometry tokens (animate smoothly between themes!)
    pub chassis_radius: f32,
    pub tray_radius: f32,
    pub tile_radius: f32,
    pub classic_bevel: bool,
}

impl Palette {
    pub fn for_theme(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Obsidian => Self::obsidian(),
            ThemeKind::Alabaster => Self::alabaster(),
            ThemeKind::Classic95 => Self::classic95(),
        }
    }

    pub fn number_color(&self, n: u8) -> Color {
        if (1..=8).contains(&n) {
            self.numbers[(n - 1) as usize]
        } else {
            self.text
        }
    }

    fn obsidian() -> Self {
        Self {
            kind: ThemeKind::Obsidian,
            bg_top: oklch(0.145, 0.013, 260.0),
            bg_bottom: oklch(0.110, 0.014, 264.0),
            chassis: oklch(0.192, 0.013, 260.0),
            chassis_top: oklch(0.212, 0.014, 258.0),
            chassis_bottom: oklch(0.175, 0.013, 262.0),
            chassis_border_top: oklcha(1.0, 0.0, 0.0, 0.14),
            chassis_border_bottom: oklcha(1.0, 0.0, 0.0, 0.03),
            chassis_shadow: oklcha(0.0, 0.0, 0.0, 0.52),
            chassis_shadow_soft: oklcha(0.0, 0.0, 0.0, 0.34),

            tray_bg: oklch(0.122, 0.011, 260.0),
            tray_border: oklcha(1.0, 0.0, 0.0, 0.06),
            tray_inset_shadow: oklcha(0.0, 0.0, 0.0, 0.58),
            tray_inset_highlight: oklcha(1.0, 0.0, 0.0, 0.04),

            hud_bg: oklch(0.158, 0.012, 260.0),
            hud_border: oklcha(1.0, 0.0, 0.0, 0.065),
            led_well: oklch(0.095, 0.010, 260.0),
            led_well_border: oklcha(1.0, 0.0, 0.0, 0.08),
            led_bombs: oklch(0.71, 0.21, 24.0),
            led_timer: oklch(0.84, 0.125, 80.0),
            led_ghost: oklcha(0.71, 0.15, 24.0, 0.08),
            show_led_ghost: false,

            tile_top: oklch(0.328, 0.016, 258.0),
            tile_bottom: oklch(0.258, 0.015, 260.0),
            tile_border_top: oklcha(1.0, 0.0, 0.0, 0.18),
            tile_border_bottom: oklcha(0.0, 0.0, 0.0, 0.36),
            tile_bevel_light: oklcha(1.0, 0.0, 0.0, 0.12),
            tile_bevel_dark: oklcha(0.0, 0.0, 0.0, 0.34),
            tile_shadow: oklcha(0.0, 0.0, 0.0, 0.42),
            tile_hover: oklcha(0.95, 0.02, 250.0, 0.10),
            tile_press: oklcha(0.0, 0.0, 0.0, 0.18),
            tile_depressed: oklch(0.205, 0.014, 260.0),
            tile_flagged_tint: oklch(0.295, 0.032, 26.0),

            revealed_a: oklch(0.172, 0.011, 260.0),
            revealed_b: oklch(0.162, 0.011, 260.0),
            revealed_border: oklcha(1.0, 0.0, 0.0, 0.036),
            revealed_hover: oklcha(1.0, 0.0, 0.0, 0.03),

            blast_top: oklch(0.58, 0.22, 25.0),
            blast_bottom: oklch(0.45, 0.20, 22.0),
            blast_border: oklch(0.78, 0.18, 36.0),
            bomb_revealed_bg: oklch(0.285, 0.044, 24.0),
            misflag_bg: oklch(0.255, 0.058, 20.0),
            victory_flag_bg: oklch(0.26, 0.055, 152.0),

            numbers: [
                oklch(0.75, 0.155, 245.0), // 1: Cerulean
                oklch(0.77, 0.165, 152.0), // 2: Emerald
                oklch(0.70, 0.200, 24.0),  // 3: Coral Red
                oklch(0.72, 0.180, 290.0), // 4: Royal Violet
                oklch(0.76, 0.170, 52.0),  // 5: Warm Amber
                oklch(0.78, 0.135, 198.0), // 6: Cyan
                oklch(0.87, 0.105, 90.0),  // 7: Pale Gold
                oklch(0.76, 0.020, 255.0), // 8: Silver
            ],

            bomb_body: oklch(0.155, 0.014, 260.0),
            bomb_rim: oklch(0.72, 0.035, 32.0),
            bomb_highlight: oklch(0.96, 0.008, 255.0),
            bomb_spark: oklch(0.88, 0.18, 78.0),
            blast_star: oklch(0.90, 0.18, 76.0),
            flag_pole: oklch(0.86, 0.01, 255.0),
            flag_base: oklch(0.72, 0.015, 255.0),
            flag_cloth: oklch(0.65, 0.225, 26.0),
            flag_cloth_bright: oklch(0.77, 0.185, 36.0),
            question_ink: oklch(0.84, 0.155, 76.0),
            misflag_cross: oklch(0.66, 0.22, 24.0),

            mascot_ring_top: oklch(0.32, 0.018, 258.0),
            mascot_ring_bottom: oklch(0.24, 0.015, 260.0),
            mascot_face: oklch(0.84, 0.165, 86.0),
            mascot_face_shadow: oklch(0.72, 0.155, 72.0),
            mascot_ink: oklch(0.17, 0.02, 55.0),
            mascot_glasses: oklch(0.15, 0.015, 260.0),
            mascot_glint: oklcha(1.0, 0.0, 0.0, 0.62),
            mascot_blush: oklcha(0.66, 0.20, 25.0, 0.45),

            text: oklch(0.955, 0.006, 255.0),
            text_muted: oklch(0.73, 0.012, 258.0),
            text_faint: oklch(0.55, 0.012, 258.0),
            accent: oklch(0.72, 0.175, 248.0),
            accent_soft: oklcha(0.72, 0.175, 248.0, 0.16),
            success: oklch(0.74, 0.165, 152.0),
            danger: oklch(0.67, 0.21, 25.0),
            warning: oklch(0.82, 0.155, 78.0),
            control_track: oklcha(1.0, 0.0, 0.0, 0.055),
            control_pill: oklch(0.32, 0.016, 258.0),
            control_hover: oklcha(1.0, 0.0, 0.0, 0.075),
            control_border: oklcha(1.0, 0.0, 0.0, 0.09),
            focus_ring: oklch(0.76, 0.17, 78.0),
            modal_scrim: oklcha(0.04, 0.01, 260.0, 0.64),
            modal_bg: oklch(0.205, 0.014, 260.0),

            chassis_radius: 18.0,
            tray_radius: 12.0,
            tile_radius: 6.5,
            classic_bevel: false,
        }
    }

    fn alabaster() -> Self {
        Self {
            kind: ThemeKind::Alabaster,
            bg_top: oklch(0.945, 0.008, 82.0),
            bg_bottom: oklch(0.908, 0.010, 245.0),
            chassis: oklch(0.985, 0.004, 82.0),
            chassis_top: oklch(0.995, 0.003, 82.0),
            chassis_bottom: oklch(0.968, 0.006, 245.0),
            chassis_border_top: oklcha(1.0, 0.0, 0.0, 0.9),
            chassis_border_bottom: oklcha(0.2, 0.02, 255.0, 0.14),
            chassis_shadow: oklcha(0.2, 0.02, 255.0, 0.15),
            chassis_shadow_soft: oklcha(0.2, 0.02, 255.0, 0.08),

            tray_bg: oklch(0.862, 0.010, 250.0),
            tray_border: oklcha(0.2, 0.02, 255.0, 0.12),
            tray_inset_shadow: oklcha(0.2, 0.02, 255.0, 0.18),
            tray_inset_highlight: oklcha(1.0, 0.0, 0.0, 0.65),

            hud_bg: oklch(0.948, 0.007, 248.0),
            hud_border: oklcha(0.2, 0.02, 255.0, 0.09),
            led_well: oklch(0.165, 0.012, 260.0),
            led_well_border: oklcha(0.0, 0.0, 0.0, 0.25),
            led_bombs: oklch(0.71, 0.21, 24.0),
            led_timer: oklch(0.86, 0.125, 82.0),
            led_ghost: oklcha(0.71, 0.15, 24.0, 0.08),
            show_led_ghost: false,

            tile_top: oklch(0.995, 0.002, 85.0),
            tile_bottom: oklch(0.935, 0.008, 250.0),
            tile_border_top: oklcha(1.0, 0.0, 0.0, 0.95),
            tile_border_bottom: oklcha(0.2, 0.02, 255.0, 0.20),
            tile_bevel_light: oklcha(1.0, 0.0, 0.0, 0.85),
            tile_bevel_dark: oklcha(0.2, 0.02, 255.0, 0.18),
            tile_shadow: oklcha(0.2, 0.02, 255.0, 0.16),
            tile_hover: oklcha(0.55, 0.14, 248.0, 0.08),
            tile_press: oklcha(0.2, 0.02, 255.0, 0.10),
            tile_depressed: oklch(0.885, 0.010, 250.0),
            tile_flagged_tint: oklch(0.965, 0.026, 28.0),

            revealed_a: oklch(0.892, 0.008, 250.0),
            revealed_b: oklch(0.876, 0.009, 250.0),
            revealed_border: oklcha(0.2, 0.02, 255.0, 0.085),
            revealed_hover: oklcha(0.2, 0.02, 255.0, 0.035),

            blast_top: oklch(0.62, 0.22, 25.0),
            blast_bottom: oklch(0.51, 0.21, 22.0),
            blast_border: oklch(0.45, 0.20, 22.0),
            bomb_revealed_bg: oklch(0.875, 0.035, 24.0),
            misflag_bg: oklch(0.89, 0.055, 22.0),
            victory_flag_bg: oklch(0.91, 0.055, 152.0),

            numbers: [
                oklch(0.49, 0.205, 256.0), // 1: Cobalt
                oklch(0.48, 0.165, 148.0), // 2: Forest Emerald
                oklch(0.54, 0.225, 25.0),  // 3: Crimson
                oklch(0.39, 0.185, 284.0), // 4: Deep Indigo
                oklch(0.44, 0.165, 22.0),  // 5: Terracotta Maroon
                oklch(0.48, 0.125, 200.0), // 6: Deep Teal
                oklch(0.25, 0.015, 260.0), // 7: Ink
                oklch(0.47, 0.015, 255.0), // 8: Slate
            ],

            bomb_body: oklch(0.21, 0.016, 260.0),
            bomb_rim: oklch(0.96, 0.005, 250.0),
            bomb_highlight: oklch(0.96, 0.008, 250.0),
            bomb_spark: oklch(0.74, 0.19, 55.0),
            blast_star: oklch(0.94, 0.16, 82.0),
            flag_pole: oklch(0.26, 0.016, 260.0),
            flag_base: oklch(0.34, 0.016, 260.0),
            flag_cloth: oklch(0.58, 0.23, 26.0),
            flag_cloth_bright: oklch(0.69, 0.20, 34.0),
            question_ink: oklch(0.49, 0.16, 280.0),
            misflag_cross: oklch(0.55, 0.23, 24.0),

            mascot_ring_top: oklch(0.99, 0.003, 82.0),
            mascot_ring_bottom: oklch(0.91, 0.009, 250.0),
            mascot_face: oklch(0.86, 0.17, 88.0),
            mascot_face_shadow: oklch(0.75, 0.16, 74.0),
            mascot_ink: oklch(0.20, 0.02, 55.0),
            mascot_glasses: oklch(0.17, 0.015, 260.0),
            mascot_glint: oklcha(1.0, 0.0, 0.0, 0.65),
            mascot_blush: oklcha(0.64, 0.20, 24.0, 0.42),

            text: oklch(0.21, 0.014, 260.0),
            text_muted: oklch(0.46, 0.015, 258.0),
            text_faint: oklch(0.60, 0.012, 258.0),
            accent: oklch(0.54, 0.19, 254.0),
            accent_soft: oklcha(0.54, 0.19, 254.0, 0.12),
            success: oklch(0.50, 0.16, 150.0),
            danger: oklch(0.55, 0.22, 25.0),
            warning: oklch(0.62, 0.16, 65.0),
            control_track: oklcha(0.2, 0.02, 255.0, 0.085),
            control_pill: oklch(1.0, 0.0, 0.0),
            control_hover: oklcha(0.2, 0.02, 255.0, 0.06),
            control_border: oklcha(0.2, 0.02, 255.0, 0.12),
            focus_ring: oklch(0.54, 0.19, 254.0),
            modal_scrim: oklcha(0.15, 0.02, 260.0, 0.42),
            modal_bg: oklch(0.99, 0.003, 82.0),

            chassis_radius: 18.0,
            tray_radius: 12.0,
            tile_radius: 6.5,
            classic_bevel: false,
        }
    }

    fn classic95() -> Self {
        Self {
            kind: ThemeKind::Classic95,
            bg_top: oklch(0.46, 0.065, 202.0), // Classic Windows 95 Teal Desktop!
            bg_bottom: oklch(0.39, 0.060, 206.0),
            chassis: oklch(0.81, 0.004, 250.0),
            chassis_top: oklch(0.83, 0.004, 250.0),
            chassis_bottom: oklch(0.79, 0.004, 250.0),
            chassis_border_top: oklch(0.98, 0.0, 0.0),
            chassis_border_bottom: oklch(0.44, 0.006, 250.0),
            chassis_shadow: oklcha(0.0, 0.0, 0.0, 0.35),
            chassis_shadow_soft: oklcha(0.0, 0.0, 0.0, 0.20),

            tray_bg: oklch(0.60, 0.005, 250.0),
            tray_border: oklch(0.52, 0.005, 250.0),
            tray_inset_shadow: oklch(0.44, 0.005, 250.0),
            tray_inset_highlight: oklch(0.97, 0.0, 0.0),

            hud_bg: oklch(0.79, 0.004, 250.0),
            hud_border: oklch(0.56, 0.005, 250.0),
            led_well: oklch(0.08, 0.0, 0.0),
            led_well_border: oklch(0.45, 0.004, 250.0),
            led_bombs: oklch(0.63, 0.255, 27.0),
            led_timer: oklch(0.63, 0.255, 27.0),
            led_ghost: oklcha(0.45, 0.16, 27.0, 0.24),
            show_led_ghost: true,

            tile_top: oklch(0.845, 0.004, 250.0),
            tile_bottom: oklch(0.795, 0.004, 250.0),
            tile_border_top: oklch(0.98, 0.0, 0.0),
            tile_border_bottom: oklch(0.48, 0.005, 250.0),
            tile_bevel_light: oklch(0.99, 0.0, 0.0),
            tile_bevel_dark: oklch(0.47, 0.005, 250.0),
            tile_shadow: oklcha(0.0, 0.0, 0.0, 0.16),
            tile_hover: oklcha(1.0, 0.0, 0.0, 0.14),
            tile_press: oklcha(0.0, 0.0, 0.0, 0.08),
            tile_depressed: oklch(0.76, 0.004, 250.0),
            tile_flagged_tint: oklch(0.825, 0.008, 28.0),

            revealed_a: oklch(0.765, 0.004, 250.0),
            revealed_b: oklch(0.752, 0.004, 250.0),
            revealed_border: oklch(0.63, 0.005, 250.0),
            revealed_hover: oklcha(1.0, 0.0, 0.0, 0.06),

            blast_top: oklch(0.60, 0.245, 28.0),
            blast_bottom: oklch(0.54, 0.24, 27.0),
            blast_border: oklch(0.40, 0.18, 25.0),
            bomb_revealed_bg: oklch(0.76, 0.006, 250.0),
            misflag_bg: oklch(0.76, 0.03, 25.0),
            victory_flag_bg: oklch(0.81, 0.035, 145.0),

            numbers: [
                oklch(0.45, 0.26, 264.0),  // 1: Classic Blue
                oklch(0.47, 0.18, 142.0),  // 2: Classic Green
                oklch(0.56, 0.24, 27.0),   // 3: Classic Red
                oklch(0.30, 0.18, 268.0),  // 4: Classic Navy
                oklch(0.38, 0.16, 24.0),   // 5: Classic Maroon
                oklch(0.49, 0.12, 198.0),  // 6: Classic Teal
                oklch(0.16, 0.0, 0.0),     // 7: Classic Black
                oklch(0.46, 0.005, 250.0), // 8: Classic Grey
            ],

            bomb_body: oklch(0.16, 0.0, 0.0),
            bomb_rim: oklch(0.86, 0.0, 0.0),
            bomb_highlight: oklch(0.98, 0.0, 0.0),
            bomb_spark: oklch(0.72, 0.22, 42.0),
            blast_star: oklch(0.90, 0.19, 86.0),
            flag_pole: oklch(0.15, 0.0, 0.0),
            flag_base: oklch(0.15, 0.0, 0.0),
            flag_cloth: oklch(0.57, 0.24, 27.0),
            flag_cloth_bright: oklch(0.67, 0.21, 32.0),
            question_ink: oklch(0.16, 0.0, 0.0),
            misflag_cross: oklch(0.55, 0.24, 26.0),

            mascot_ring_top: oklch(0.85, 0.004, 250.0),
            mascot_ring_bottom: oklch(0.77, 0.004, 250.0),
            mascot_face: oklch(0.89, 0.18, 96.0),
            mascot_face_shadow: oklch(0.78, 0.16, 86.0),
            mascot_ink: oklch(0.15, 0.0, 0.0),
            mascot_glasses: oklch(0.14, 0.0, 0.0),
            mascot_glint: oklcha(1.0, 0.0, 0.0, 0.7),
            mascot_blush: oklcha(0.62, 0.22, 26.0, 0.45),

            text: oklch(0.16, 0.005, 250.0),
            text_muted: oklch(0.38, 0.006, 250.0),
            text_faint: oklch(0.48, 0.006, 250.0),
            accent: oklch(0.40, 0.19, 262.0),
            accent_soft: oklcha(0.40, 0.19, 262.0, 0.15),
            success: oklch(0.46, 0.16, 145.0),
            danger: oklch(0.55, 0.23, 27.0),
            warning: oklch(0.58, 0.16, 65.0),
            control_track: oklch(0.71, 0.005, 250.0),
            control_pill: oklch(0.92, 0.003, 250.0),
            control_hover: oklcha(1.0, 0.0, 0.0, 0.18),
            control_border: oklch(0.52, 0.005, 250.0),
            focus_ring: oklch(0.40, 0.22, 262.0),
            modal_scrim: oklcha(0.08, 0.02, 210.0, 0.50),
            modal_bg: oklch(0.83, 0.004, 250.0),

            chassis_radius: 6.0,
            tray_radius: 4.0,
            tile_radius: 2.5,
            classic_bevel: true,
        }
    }
}
