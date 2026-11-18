use std::f32::consts::{PI, TAU};

use rune::{Color, LineCap, Path, Shape};

use crate::theme::Palette;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MascotMood {
    Happy,
    Tense,
    Cool,
    Dead,
}

fn circle(cx: f32, cy: f32, r: f32, steps: usize) -> Vec<[f32; 2]> {
    (0..steps)
        .map(|i| {
            let a = (i as f32 / steps as f32) * TAU;
            [cx + a.cos() * r, cy + a.sin() * r]
        })
        .collect()
}

fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, steps: usize) -> Vec<[f32; 2]> {
    (0..steps)
        .map(|i| {
            let a = (i as f32 / steps as f32) * TAU;
            [cx + a.cos() * rx, cy + a.sin() * ry]
        })
        .collect()
}

fn starburst(cx: f32, cy: f32, r_outer: f32, r_inner: f32, spikes: usize) -> Vec<[f32; 2]> {
    let n = spikes * 2;
    (0..n)
        .map(|i| {
            let a = (i as f32 / n as f32) * TAU - PI * 0.5;
            let r = if i % 2 == 0 { r_outer } else { r_inner };
            [cx + a.cos() * r, cy + a.sin() * r]
        })
        .collect()
}

/// Classic sculpted naval bomb with 8 spikes, glossy highlight, and optional blast starburst.
pub fn bomb_shape(p: &Palette, size: f32, detonated: bool) -> Shape {
    let s = size / 24.0;
    let cx = 12.0 * s;
    let cy = 12.2 * s;
    let mut sh = Shape::new();

    if detonated {
        sh = sh.fill(starburst(cx, cy, 10.8 * s, 5.8 * s, 10), p.blast_star.alpha(0.92));
    }

    let body = if detonated {
        rune::oklch(0.15, 0.015, 25.0)
    } else {
        p.bomb_body
    };
    let highlight = if detonated {
        Color::WHITE.alpha(0.90)
    } else {
        p.bomb_highlight
    };

    // In dark mode, paint a subtle metallic rim under the spikes and body so the dark iron bomb
    // reads unmistakably as a black mine with crisp silhouette separation.
    if !detonated && p.kind.is_dark() {
        for i in 0..8 {
            let angle = (i as f32) * (PI / 4.0);
            let len = if i % 2 == 0 { 8.6 * s } else { 7.4 * s };
            let w = if i % 2 == 0 { 2.8 * s } else { 2.4 * s };
            let tip = [cx + angle.cos() * len, cy + angle.sin() * len];
            sh = sh.line([cx, cy], tip, w.max(1.8), LineCap::Round, p.bomb_rim.alpha(0.45));
        }
        sh = sh.fill(circle(cx, cy, 6.45 * s, 24), p.bomb_rim.alpha(0.55));
    }

    // Cardinal and diagonal prongs.
    for i in 0..8 {
        let angle = (i as f32) * (PI / 4.0);
        let len = if i % 2 == 0 { 8.3 * s } else { 7.1 * s };
        let w = if i % 2 == 0 { 2.0 * s } else { 1.65 * s };
        let tip = [cx + angle.cos() * len, cy + angle.sin() * len];
        sh = sh.line([cx, cy], tip, w.max(1.2), LineCap::Round, body);
    }

    // Core sphere.
    sh = sh.fill(circle(cx, cy, 5.75 * s, 24), body);

    // Subtle lower-right shading crescent for volume.
    if !detonated {
        sh = sh.fill(circle(cx + 0.7 * s, cy + 0.8 * s, 4.4 * s, 20), Color::BLACK.alpha(0.22));
    }

    // Glossy upper-left specular dot + tiny secondary glint.
    sh = sh
        .fill(circle(cx - 2.0 * s, cy - 2.0 * s, 1.55 * s, 14), highlight)
        .fill(circle(cx - 3.3 * s, cy - 0.5 * s, 0.65 * s, 10), highlight.alpha(0.68));

    // Tiny glowing spark when detonated.
    if detonated {
        sh = sh.fill(circle(cx + 4.4 * s, cy - 4.4 * s, 1.8 * s, 12), p.bomb_spark);
    }

    sh
}

/// Tactile pennant flag with stepped pedestal base and dimensional fold highlight.
pub fn flag_shape(p: &Palette, size: f32) -> Shape {
    let s = size / 24.0;
    let pole_x = 12.6 * s;

    // Billowing pennant banner pointing left (classic Minesweeper orientation!).
    let pennant = Path::new()
        .move_to([pole_x, 4.2 * s])
        .quad_to([8.4 * s, 6.2 * s], [4.6 * s, 8.6 * s])
        .quad_to([8.6 * s, 10.8 * s], [pole_x, 12.8 * s])
        .close();

    // Lighter upper highlight wedge for 3D silk sheen.
    let highlight = Path::new()
        .move_to([pole_x, 4.4 * s])
        .quad_to([8.8 * s, 6.3 * s], [5.4 * s, 8.4 * s])
        .line_to([pole_x, 8.7 * s])
        .close();

    // Stepped classic pedestal base.
    let base_top = vec![
        [8.4 * s, 17.5 * s],
        [16.8 * s, 17.5 * s],
        [16.8 * s, 19.1 * s],
        [8.4 * s, 19.1 * s],
    ];
    let base_bottom = vec![
        [6.4 * s, 19.0 * s],
        [18.8 * s, 19.0 * s],
        [18.8 * s, 20.8 * s],
        [6.4 * s, 20.8 * s],
    ];

    Shape::new()
        .fill(base_bottom, p.flag_base)
        .fill(base_top, p.flag_base)
        .line(
            [pole_x, 4.4 * s],
            [pole_x, 18.0 * s],
            (1.9 * s).max(1.3),
            LineCap::Round,
            p.flag_pole,
        )
        .fill_path(&pennant, p.flag_cloth)
        .fill_path(&highlight, p.flag_cloth_bright.alpha(0.55))
        .fill(circle(pole_x, 4.2 * s, (1.15 * s).max(0.9), 12), p.flag_pole)
}

/// Wrongly flagged cell on Game Over: muted bomb with a bold crimson X slash.
pub fn misflag_shape(p: &Palette, size: f32) -> Shape {
    let s = size / 24.0;
    bomb_shape(p, size, false)
        .line(
            [5.0 * s, 5.0 * s],
            [19.0 * s, 19.0 * s],
            (2.4 * s).max(1.8),
            LineCap::Round,
            p.misflag_cross,
        )
        .line(
            [19.0 * s, 5.0 * s],
            [5.0 * s, 19.0 * s],
            (2.4 * s).max(1.8),
            LineCap::Round,
            p.misflag_cross,
        )
}

/// Crisp vector question mark for optional `?` tile marking.
pub fn question_shape(color: Color, size: f32) -> Shape {
    let s = size / 24.0;
    let hook = Path::new()
        .move_to([8.4 * s, 8.6 * s])
        .cubic_to([8.4 * s, 4.8 * s], [15.8 * s, 4.8 * s], [15.8 * s, 8.8 * s])
        .cubic_to([15.8 * s, 11.4 * s], [12.0 * s, 11.8 * s], [12.0 * s, 14.5 * s]);

    Shape::new()
        .stroke_path(&hook, (2.3 * s).max(1.6), LineCap::Round, color)
        .fill(circle(12.0 * s, 18.4 * s, (1.45 * s).max(1.1), 14), color)
}

/// Expressive sunlit Bombsweeper Mascot Face (Happy, Tense :O, Victory Cool 😎, Dead X_X).
pub fn mascot_face_shape(p: &Palette, mood: MascotMood, size: f32) -> Shape {
    let s = size / 32.0;
    let cx = 16.0 * s;
    let cy = 16.0 * s;
    let r = 11.6 * s;

    // Base face disc + subtle warm lower shading + specular upper gloss + dark outline.
    let face_outline = circle(cx, cy, r, 32);
    let mut sh = Shape::new()
        .fill(circle(cx, cy, r, 32), p.mascot_face_shadow)
        .fill(circle(cx - 0.5 * s, cy - 0.8 * s, r * 0.93, 32), p.mascot_face)
        .fill(ellipse(cx - 2.6 * s, cy - 5.4 * s, 4.6 * s, 2.2 * s, 18), Color::WHITE.alpha(0.32))
        .stroke_with(face_outline, true, (1.35 * s).max(1.0), LineCap::Round, p.mascot_ink.alpha(0.82));

    match mood {
        MascotMood::Happy => {
            // Rosy cheeks
            sh = sh
                .fill(ellipse(cx - 6.0 * s, cy + 1.4 * s, 2.1 * s, 1.3 * s, 14), p.mascot_blush)
                .fill(ellipse(cx + 6.0 * s, cy + 1.4 * s, 2.1 * s, 1.3 * s, 14), p.mascot_blush);
            // Eyes with tiny friendly highlights
            sh = sh
                .fill(circle(cx - 4.0 * s, cy - 2.2 * s, 1.55 * s, 14), p.mascot_ink)
                .fill(circle(cx + 4.0 * s, cy - 2.2 * s, 1.55 * s, 14), p.mascot_ink)
                .fill(circle(cx - 4.4 * s, cy - 2.7 * s, 0.5 * s, 10), Color::WHITE.alpha(0.85))
                .fill(circle(cx + 3.6 * s, cy - 2.7 * s, 0.5 * s, 10), Color::WHITE.alpha(0.85));
            // Cheerful smile
            let smile = Path::new()
                .move_to([cx - 4.8 * s, cy + 2.4 * s])
                .quad_to([cx, cy + 7.2 * s], [cx + 4.8 * s, cy + 2.4 * s]);
            sh = sh.stroke_path(&smile, (1.75 * s).max(1.3), LineCap::Round, p.mascot_ink);
        }
        MascotMood::Tense => {
            // Alert eyebrows
            let brow_l = Path::new()
                .move_to([cx - 6.2 * s, cy - 5.2 * s])
                .quad_to([cx - 4.0 * s, cy - 6.5 * s], [cx - 2.2 * s, cy - 5.5 * s]);
            let brow_r = Path::new()
                .move_to([cx + 2.2 * s, cy - 5.5 * s])
                .quad_to([cx + 4.0 * s, cy - 6.5 * s], [cx + 6.2 * s, cy - 5.2 * s]);
            // Wide eyes + surprised O mouth
            sh = sh
                .stroke_path(&brow_l, (1.25 * s).max(1.0), LineCap::Round, p.mascot_ink)
                .stroke_path(&brow_r, (1.25 * s).max(1.0), LineCap::Round, p.mascot_ink)
                .fill(circle(cx - 4.1 * s, cy - 2.1 * s, 1.85 * s, 14), p.mascot_ink)
                .fill(circle(cx + 4.1 * s, cy - 2.1 * s, 1.85 * s, 14), p.mascot_ink)
                .fill(circle(cx - 4.5 * s, cy - 2.6 * s, 0.6 * s, 10), Color::WHITE.alpha(0.9))
                .fill(circle(cx + 3.7 * s, cy - 2.6 * s, 0.6 * s, 10), Color::WHITE.alpha(0.9))
                .stroke_with(circle(cx, cy + 4.0 * s, 2.55 * s, 18), true, (1.65 * s).max(1.2), LineCap::Round, p.mascot_ink);
        }
        MascotMood::Cool => {
            // Classic sleek sunglasses (left & right wayfarer lenses + bridge + arms)
            let left_lens = Path::new()
                .move_to([cx - 8.8 * s, cy - 4.3 * s])
                .line_to([cx - 1.1 * s, cy - 4.3 * s])
                .cubic_to([cx - 0.9 * s, cy + 0.6 * s], [cx - 3.0 * s, cy + 1.4 * s], [cx - 5.2 * s, cy + 1.4 * s])
                .cubic_to([cx - 7.8 * s, cy + 1.4 * s], [cx - 9.0 * s, cy - 0.2 * s], [cx - 8.8 * s, cy - 4.3 * s])
                .close();
            let right_lens = Path::new()
                .move_to([cx + 1.1 * s, cy - 4.3 * s])
                .line_to([cx + 8.8 * s, cy - 4.3 * s])
                .cubic_to([cx + 9.0 * s, cy - 0.2 * s], [cx + 7.8 * s, cy + 1.4 * s], [cx + 5.2 * s, cy + 1.4 * s])
                .cubic_to([cx + 3.0 * s, cy + 1.4 * s], [cx + 0.9 * s, cy + 0.6 * s], [cx + 1.1 * s, cy - 4.3 * s])
                .close();
            let smile = Path::new()
                .move_to([cx - 5.2 * s, cy + 3.2 * s])
                .quad_to([cx, cy + 8.0 * s], [cx + 5.2 * s, cy + 3.2 * s]);

            sh = sh
                .line([cx - 10.6 * s, cy - 3.4 * s], [cx + 10.6 * s, cy - 3.4 * s], (1.7 * s).max(1.2), LineCap::Round, p.mascot_glasses)
                .fill_path(&left_lens, p.mascot_glasses)
                .fill_path(&right_lens, p.mascot_glasses)
                // Crisp diagonal glints on lenses
                .line([cx - 6.8 * s, cy - 2.9 * s], [cx - 4.8 * s, cy - 0.5 * s], (1.25 * s).max(0.9), LineCap::Round, p.mascot_glint)
                .line([cx + 3.0 * s, cy - 2.9 * s], [cx + 5.0 * s, cy - 0.5 * s], (1.25 * s).max(0.9), LineCap::Round, p.mascot_glint)
                .stroke_path(&smile, (1.85 * s).max(1.3), LineCap::Round, p.mascot_ink);
        }
        MascotMood::Dead => {
            // Crisp X_X eyes + dizzy frown
            let d = 1.9 * s;
            let (lx, rx, ey) = (cx - 4.2 * s, cx + 4.2 * s, cy - 2.2 * s);
            let frown = Path::new()
                .move_to([cx - 4.8 * s, cy + 5.2 * s])
                .quad_to([cx, cy + 1.0 * s], [cx + 4.8 * s, cy + 5.2 * s]);

            sh = sh
                .line([lx - d, ey - d], [lx + d, ey + d], (1.75 * s).max(1.2), LineCap::Round, p.mascot_ink)
                .line([lx + d, ey - d], [lx - d, ey + d], (1.75 * s).max(1.2), LineCap::Round, p.mascot_ink)
                .line([rx - d, ey - d], [rx + d, ey + d], (1.75 * s).max(1.2), LineCap::Round, p.mascot_ink)
                .line([rx + d, ey - d], [rx - d, ey + d], (1.75 * s).max(1.2), LineCap::Round, p.mascot_ink)
                .stroke_path(&frown, (1.75 * s).max(1.2), LineCap::Round, p.mascot_ink);
        }
    }

    sh
}

/// Compact bomb icon for HUD counter and title mark.
pub fn mini_bomb_icon(color: Color, spark: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let cx = 8.4 * s;
    let cy = 10.2 * s;
    let fuse = Path::new()
        .move_to([cx + 2.6 * s, cy - 3.6 * s])
        .quad_to([cx + 4.8 * s, cy - 7.0 * s], [cx + 6.4 * s, cy - 6.0 * s]);
    Shape::new()
        .fill(circle(cx, cy, 4.9 * s, 20), color)
        .fill(circle(cx - 1.6 * s, cy - 1.6 * s, 1.15 * s, 12), Color::WHITE.alpha(0.55))
        .stroke_path(&fuse, (1.5 * s).max(1.1), LineCap::Round, color)
        .fill(starburst(cx + 6.5 * s, cy - 6.0 * s, 2.5 * s, 1.0 * s, 4), spark)
}

/// Crisp stopwatch/clock icon for the HUD timer.
pub fn clock_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let cx = 9.0 * s;
    let cy = 10.0 * s;
    let r = 5.6 * s;
    Shape::new()
        .line([cx - 2.0 * s, 2.4 * s], [cx + 2.0 * s, 2.4 * s], (1.6 * s).max(1.1), LineCap::Round, color)
        .line([cx, 2.6 * s], [cx, 4.4 * s], (1.5 * s).max(1.1), LineCap::Round, color)
        .stroke_with(circle(cx, cy, r, 24), true, (1.55 * s).max(1.1), LineCap::Round, color)
        .line([cx, cy], [cx, cy - 3.2 * s], (1.45 * s).max(1.1), LineCap::Round, color)
        .line([cx, cy], [cx + 2.5 * s, cy + 1.3 * s], (1.45 * s).max(1.1), LineCap::Round, color)
}

/// 4-point sparkle icon for Smart Hint.
pub fn sparkle_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    Shape::new()
        .fill(starburst(8.6 * s, 9.4 * s, 6.2 * s, 1.8 * s, 4), color)
        .fill(starburst(14.3 * s, 4.2 * s, 2.5 * s, 0.8 * s, 4), color.alpha(0.8))
}

/// Trophy chalice icon for Best Times & Statistics.
pub fn trophy_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let cup = Path::new()
        .move_to([5.2 * s, 3.6 * s])
        .line_to([12.8 * s, 3.6 * s])
        .line_to([12.2 * s, 8.6 * s])
        .quad_to([9.0 * s, 11.4 * s], [5.8 * s, 8.6 * s])
        .close();
    let handle_l = Path::new()
        .move_to([5.2 * s, 4.6 * s])
        .cubic_to([2.6 * s, 4.6 * s], [2.6 * s, 8.2 * s], [5.7 * s, 8.0 * s]);
    let handle_r = Path::new()
        .move_to([12.8 * s, 4.6 * s])
        .cubic_to([15.4 * s, 4.6 * s], [15.4 * s, 8.2 * s], [12.3 * s, 8.0 * s]);

    Shape::new()
        .fill_path(&cup, color)
        .stroke_path(&handle_l, (1.3 * s).max(1.0), LineCap::Round, color)
        .stroke_path(&handle_r, (1.3 * s).max(1.0), LineCap::Round, color)
        .line([9.0 * s, 10.0 * s], [9.0 * s, 13.6 * s], (1.8 * s).max(1.2), LineCap::Round, color)
        .line([6.0 * s, 14.4 * s], [12.0 * s, 14.4 * s], (2.0 * s).max(1.3), LineCap::Round, color)
}

/// Compact flag icon for the toolbar Dig/Flag mode switch.
pub fn mini_flag_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let banner = vec![
        [5.4 * s, 3.4 * s],
        [13.8 * s, 6.4 * s],
        [5.4 * s, 9.8 * s],
    ];
    Shape::new()
        .line([5.4 * s, 3.2 * s], [5.4 * s, 15.0 * s], (1.7 * s).max(1.2), LineCap::Round, color)
        .fill(banner, color)
        .line([3.6 * s, 15.2 * s], [7.6 * s, 15.2 * s], (1.6 * s).max(1.1), LineCap::Round, color)
}

/// Crisp target/cursor icon for Dig mode.
pub fn dig_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let diamond = vec![
        [9.0 * s, 3.0 * s],
        [14.8 * s, 9.0 * s],
        [9.0 * s, 15.0 * s],
        [3.2 * s, 9.0 * s],
    ];
    Shape::new()
        .stroke_with(diamond, true, (1.6 * s).max(1.1), LineCap::Round, color)
        .fill(circle(9.0 * s, 9.0 * s, 2.0 * s, 12), color)
}

/// Sliders icon for Custom board settings.
pub fn sliders_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let w = (1.5 * s).max(1.1);
    Shape::new()
        .line([3.0 * s, 5.2 * s], [15.0 * s, 5.2 * s], w, LineCap::Round, color)
        .line([3.0 * s, 9.0 * s], [15.0 * s, 9.0 * s], w, LineCap::Round, color)
        .line([3.0 * s, 12.8 * s], [15.0 * s, 12.8 * s], w, LineCap::Round, color)
        .fill(circle(6.8 * s, 5.2 * s, 2.0 * s, 12), color)
        .fill(circle(11.6 * s, 9.0 * s, 2.0 * s, 12), color)
        .fill(circle(7.8 * s, 12.8 * s, 2.0 * s, 12), color)
}

/// Golden crown icon for Personal Best badge.
pub fn crown_icon(color: Color, size: f32) -> Shape {
    let s = size / 18.0;
    let body = vec![
        [3.0 * s, 13.4 * s],
        [3.0 * s, 6.2 * s],
        [6.6 * s, 9.4 * s],
        [9.0 * s, 4.4 * s],
        [11.4 * s, 9.4 * s],
        [15.0 * s, 6.2 * s],
        [15.0 * s, 13.4 * s],
    ];
    Shape::new().fill(body, color)
}

/// Crisp chamfered 3-digit 7-segment LED display for the Classic '95 theme.
/// Segments bitmask: bit 0=A(top), 1=B(top-right), 2=C(bot-right), 3=D(bottom),
/// 4=E(bot-left), 5=F(top-left), 6=G(middle).
pub fn seven_seg_display(text: &str, lit: Color, unlit: Color, width: f32, height: f32) -> Shape {
    let mut sh = Shape::new();
    let chars: Vec<char> = text.chars().take(3).collect();
    let count = chars.len().max(1) as f32;
    let gap = 5.0;
    let dw = (width - gap * (count - 1.0)) / count;
    let dh = height;
    let t = (dw * 0.21).clamp(2.0, 3.0);
    let m = dh * 0.5;
    let pad = 0.7;

    let h_seg = |x: f32, y: f32| -> Vec<[f32; 2]> {
        let half = t * 0.5;
        vec![
            [x + t * 0.6 + pad, y - half],
            [x + dw - t * 0.6 - pad, y - half],
            [x + dw - pad, y],
            [x + dw - t * 0.6 - pad, y + half],
            [x + t * 0.6 + pad, y + half],
            [x + pad, y],
        ]
    };

    let v_seg = |x: f32, y0: f32, y1: f32| -> Vec<[f32; 2]> {
        let half = t * 0.5;
        vec![
            [x, y0 + pad],
            [x + half, y0 + t * 0.6 + pad],
            [x + half, y1 - t * 0.6 - pad],
            [x, y1 - pad],
            [x - half, y1 - t * 0.6 - pad],
            [x - half, y0 + t * 0.6 + pad],
        ]
    };

    for (idx, ch) in chars.into_iter().enumerate() {
        let x0 = idx as f32 * (dw + gap);
        let mask: u8 = match ch {
            '0' => 0b011_1111,
            '1' => 0b000_0110,
            '2' => 0b101_1011,
            '3' => 0b100_1111,
            '4' => 0b110_0110,
            '5' => 0b110_1101,
            '6' => 0b111_1101,
            '7' => 0b000_0111,
            '8' => 0b111_1111,
            '9' => 0b110_1111,
            '-' => 0b100_0000,
            _ => 0,
        };
        let half = t * 0.5;
        let segs = [
            h_seg(x0, half),
            v_seg(x0 + dw - half, half, m),
            v_seg(x0 + dw - half, m, dh - half),
            h_seg(x0, dh - half),
            v_seg(x0 + half, m, dh - half),
            v_seg(x0 + half, half, m),
            h_seg(x0, m),
        ];
        for (bit, poly) in segs.into_iter().enumerate() {
            let color = if (mask & (1 << bit)) != 0 { lit } else { unlit };
            sh = sh.fill(poly, color);
        }
    }
    sh
}
