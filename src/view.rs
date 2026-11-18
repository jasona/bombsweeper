use rune::tok::motion;
use rune::{
    Align, Beat, Color, Cursor, Cx, Drag, El, MotionIntent, Phase, Sequence, Shadow, Spring,
    SpringPair, TextStyle, Transition, Tx, canvas, col, el, row, spacer, text,
};
use rune_kit::{Kit, RollingText};

use crate::art;
use crate::game::{CellState, Difficulty, GamePhase, HintKind};
use crate::state::{AppState, Modal, PressTarget, format_ms};
use crate::theme::{Palette, ThemeKind};

const TILE_SPRING: Spring = Spring::new(0.20, 0.0);
const POP_SPRING: Spring = Spring::new(0.24, 0.32);
const EXIT_SPRING: Spring = Spring::new(0.14, 0.0);
const ODOMETER_SPRING: Spring = Spring::new(0.28, 0.08);
const PROGRESS_SPRING: Spring = Spring::new(0.30, 0.0);

const BLAST_PULSE: &[Beat] = &[
    Beat::new(0.0).scale(0.84),
    Beat::new(0.14).scale(1.07),
    Beat::new(0.32).scale(1.0),
];

const HINT_PULSE: &[Beat] = &[
    Beat::new(0.0).opacity(0.45).scale(0.92),
    Beat::new(0.45).opacity(1.0).scale(1.02),
    Beat::new(0.90).opacity(0.65).scale(0.98),
];

pub fn view(s: &AppState, cx: &mut Cx<AppState>) -> El {
    let p = Palette::for_theme(s.theme);
    let kit = Kit::neutral(s.theme.is_dark());
    let hair = (1.0 / cx.env().scale).min(1.0);

    let (cols, _, _) = s.board.difficulty.dims();
    let tile = s.board.difficulty.tile_size();
    let gap = s.board.difficulty.tile_gap();
    let board_w = cols as f32 * tile + cols.saturating_sub(1) as f32 * gap;
    let content_w = (board_w + 20.0).max(428.0);

    let mut chassis = col()
        .key("chassis")
        .w(content_w + 32.0)
        .padding(16.0)
        .gap(11.0)
        .items_center()
        .radius(p.chassis_radius)
        .fill(p.chassis)
        .gradient(p.chassis_top, p.chassis_bottom)
        .border_gradient(if p.classic_bevel { 1.5 } else { 1.0 }, p.chassis_border_top, p.chassis_border_bottom)
        .shadow(2.0, 8.0, p.chassis_shadow)
        .shadow(16.0, 42.0, p.chassis_shadow_soft)
        .motion(motion::SNAPPY)
        .child(top_bar(s, &p, content_w))
        .child(difficulty_bar(s, &p, content_w))
        .child(hud_deck(s, &p, &kit, content_w, hair))
        .child(progress_rail(s, &p, content_w))
        .child(board_tray(s, &p, hair))
        .child(footer_bar(s, &p, content_w));

    if !p.classic_bevel {
        chassis = chassis.continuous_corners();
    } else {
        chassis = chassis
            .box_shadow(Shadow {
                x: 2.0,
                y: 2.0,
                blur: 0.5,
                spread: 0.0,
                color: p.tile_bevel_light,
                inset: true,
            })
            .box_shadow(Shadow {
                x: -2.0,
                y: -2.0,
                blur: 0.5,
                spread: 0.0,
                color: p.tile_bevel_dark,
                inset: true,
            });
    }

    let mut root = el()
        .key("window")
        .role("window")
        .w_full()
        .h_full()
        .center()
        .fill(p.bg_bottom)
        .gradient(p.bg_top, p.bg_bottom)
        .motion(motion::DEFAULT)
        .child(chassis);

    match s.modal {
        Modal::None => {}
        Modal::Stats => {
            root = root.child(stats_modal(s, &p, hair));
        }
        Modal::Custom => {
            root = root.child(custom_modal(s, &p, hair));
        }
    }

    root
}

fn top_bar(s: &AppState, p: &Palette, width: f32) -> El {
    let brand = row()
        .key("brand")
        .center_y()
        .gap(8.0)
        .child(
            el().key("brand-mark")
                .size(26.0, 26.0)
                .center()
                .radius(if p.classic_bevel { 3.0 } else { 7.5 })
                .fill(p.hud_bg)
                .border(1.0, p.hud_border)
                .child(canvas(art::mini_bomb_icon(p.text, p.bomb_spark, 16.0)).size(16.0, 16.0)),
        )
        .child(
            col()
                .gap(0.0)
                .child(
                    text("BOMBSWEEPER")
                        .key("title")
                        .font(TextStyle::new(13.0, 600.0).tracking(1.1))
                        .color(p.text),
                ),
        );

    let theme_pill = theme_switcher(s, p);

    let hint_active = s.hint.is_some();
    let hint_btn = icon_chip(
        "hint-btn",
        "Smart Hint (H)",
        hint_active,
        p,
        canvas(art::sparkle_icon(if hint_active { p.warning } else { p.text_muted }, 15.0))
            .size(15.0, 15.0),
    )
    .on_press(|s: &mut AppState| s.request_hint());

    let stats_btn = icon_chip(
        "stats-btn",
        "Best Times & Stats",
        s.modal == Modal::Stats,
        p,
        canvas(art::trophy_icon(
            if s.modal == Modal::Stats { p.warning } else { p.text_muted },
            15.0,
        ))
        .size(15.0, 15.0),
    )
    .on_press(|s: &mut AppState| {
        s.modal = if s.modal == Modal::Stats { Modal::None } else { Modal::Stats };
    });

    let custom_btn = icon_chip(
        "custom-btn",
        "Custom Board",
        s.modal == Modal::Custom || matches!(s.board.difficulty, Difficulty::Custom { .. }),
        p,
        canvas(art::sliders_icon(
            if s.modal == Modal::Custom || matches!(s.board.difficulty, Difficulty::Custom { .. }) {
                p.accent
            } else {
                p.text_muted
            },
            15.0,
        ))
        .size(15.0, 15.0),
    )
    .on_press(|s: &mut AppState| {
        s.modal = if s.modal == Modal::Custom { Modal::None } else { Modal::Custom };
    });

    let utilities = row()
        .key("utilities")
        .center_y()
        .gap(6.0)
        .child(theme_pill)
        .child(hint_btn)
        .child(stats_btn)
        .child(custom_btn);

    row()
        .key("top-bar")
        .w(width)
        .h(30.0)
        .center_y()
        .justify_between()
        .child(brand)
        .child(utilities)
}

fn theme_switcher(s: &AppState, p: &Palette) -> El {
    let pad = 3.0;
    let seg = 38.0;
    let h = 26.0;
    let idx = ThemeKind::ALL.iter().position(|&t| t == s.theme).unwrap_or(0);
    let r_outer = if p.classic_bevel { 4.0 } else { 8.0 };
    let r_inner = if p.classic_bevel { 2.5 } else { 5.5 };

    let mut segs = row().key("theme-segments");
    for &kind in &ThemeKind::ALL {
        let selected = kind == s.theme;
        segs = segs.child(
            el().key(format!("theme-{}", kind.short_label()))
                .role("radio")
                .label(format!("{} theme", kind.label()))
                .checked(selected)
                .focusable(true)
                .cursor(Cursor::Pointer)
                .size(seg, h - 2.0 * pad)
                .center()
                .radius(r_inner)
                .hover(if selected { Color::TRANSPARENT } else { p.control_hover })
                .on_press_tx(move |s: &mut AppState, tx| s.set_theme(kind, tx))
                .child(
                    text(kind.short_label())
                        .font(TextStyle::new(10.5, if selected { 600.0 } else { 500.0 }))
                        .color(if selected { p.text } else { p.text_muted })
                        .a11y_hidden(),
                ),
        );
    }

    el().key("theme-switcher")
        .role("radiogroup")
        .label("Theme")
        .size(seg * 3.0 + pad * 2.0, h)
        .padding(pad)
        .radius(r_outer)
        .fill(p.control_track)
        .border(1.0, p.control_border)
        .motion(motion::SNAPPY)
        .child(
            el().key("theme-indicator")
                .size(seg, h - 2.0 * pad)
                .absolute(pad + idx as f32 * seg, pad)
                .radius(r_inner)
                .fill(p.control_pill)
                .shadow(1.0, 2.5, p.tile_shadow)
                .lead(motion::LEAD, motion::DEFAULT)
                .a11y_hidden(),
        )
        .child(segs)
}

fn icon_chip(key: &str, help_text: &str, active: bool, p: &Palette, icon_el: El) -> El {
    let r = if p.classic_bevel { 3.5 } else { 7.5 };
    el().key(key)
        .role("button")
        .label(help_text)
        .help(help_text)
        .focusable(true)
        .cursor(Cursor::Pointer)
        .size(28.0, 26.0)
        .center()
        .radius(r)
        .fill(if active { p.control_pill } else { p.control_track })
        .border(1.0, if active { p.accent.alpha(0.5) } else { p.control_border })
        .hover(p.control_hover)
        .press_scale(0.92)
        .child(icon_el)
}

fn difficulty_bar(s: &AppState, p: &Palette, width: f32) -> El {
    let pad = 3.0;
    let h = 30.0;
    let seg_w = if width >= 540.0 { 96.0 } else { 82.0 };
    let active_idx = s.board.difficulty.preset_index();
    let r_outer = if p.classic_bevel { 4.0 } else { 9.0 };
    let r_inner = if p.classic_bevel { 2.5 } else { 6.5 };

    let mut segs = row().key("diff-segments");
    for &diff in &Difficulty::PRESETS {
        let selected = s.board.difficulty == diff;
        segs = segs.child(
            el().key(format!("diff-{}", diff.label().to_lowercase()))
                .role("segment")
                .label(format!("{} ({})", diff.label(), diff.short_label()))
                .checked(selected)
                .focusable(true)
                .cursor(Cursor::Pointer)
                .size(seg_w, h - 2.0 * pad)
                .center()
                .radius(r_inner)
                .hover(if selected { Color::TRANSPARENT } else { p.control_hover })
                .on_press_tx(move |s: &mut AppState, tx| s.start_new_game(diff, tx))
                .child(
                    text(diff.label())
                        .font(TextStyle::new(11.5, if selected { 600.0 } else { 500.0 }))
                        .color(if selected { p.text } else { p.text_muted })
                        .a11y_hidden(),
                ),
        );
    }

    let mut diff_control = el()
        .key("difficulty-segmented")
        .role("segmented")
        .label("Difficulty")
        .size(seg_w * 3.0 + pad * 2.0, h)
        .padding(pad)
        .radius(r_outer)
        .fill(p.control_track)
        .border(1.0, p.control_border)
        .motion(motion::SNAPPY);

    if let Some(idx) = active_idx {
        diff_control = diff_control.child(
            el().key("diff-indicator")
                .size(seg_w, h - 2.0 * pad)
                .absolute(pad + idx as f32 * seg_w, pad)
                .radius(r_inner)
                .fill(p.control_pill)
                .shadow(1.0, 3.0, p.tile_shadow)
                .lead(motion::LEAD, motion::DEFAULT)
                .a11y_hidden(),
        );
    }
    diff_control = diff_control.child(segs);

    // Custom active badge if using a custom board
    let custom_tag = if let Difficulty::Custom { cols, rows, bombs } = s.board.difficulty {
        Some(
            el().key("custom-active-badge")
                .h(26.0)
                .px(8.0)
                .center()
                .radius(r_inner)
                .fill(p.accent_soft)
                .border(1.0, p.accent.alpha(0.45))
                .child(
                    text(format!("{cols}×{rows} · {bombs}"))
                        .font(TextStyle::new(11.0, 600.0).features("'tnum' 1"))
                        .color(p.accent),
                ),
        )
    } else {
        None
    };

    // Compact Dig / Flag mode pill + `?` toggle on right
    let mode_btn = el()
        .key("mode-toggle")
        .role("switch")
        .label(if s.flag_mode { "Flag mode active" } else { "Dig mode active" })
        .checked(s.flag_mode)
        .help("Switch primary click between Dig and Flag (M)")
        .focusable(true)
        .cursor(Cursor::Pointer)
        .h(30.0)
        .px(9.0)
        .radius(r_outer)
        .fill(if s.flag_mode { p.flag_cloth.alpha(0.18) } else { p.control_track })
        .border(
            1.0,
            if s.flag_mode { p.flag_cloth.alpha(0.55) } else { p.control_border },
        )
        .hover(p.control_hover)
        .press_scale(0.95)
        .center()
        .on_press(|s: &mut AppState| s.flag_mode = !s.flag_mode)
        .child(
            row()
                .center_y()
                .gap(5.0)
                .child(
                    canvas(if s.flag_mode {
                        art::mini_flag_icon(p.flag_cloth, 14.0)
                    } else {
                        art::dig_icon(p.accent, 14.0)
                    })
                    .size(14.0, 14.0),
                )
                .child(
                    text(if s.flag_mode { "Flag" } else { "Dig" })
                        .font(TextStyle::new(11.5, 600.0))
                        .color(if s.flag_mode { p.flag_cloth } else { p.text })
                        .a11y_hidden(),
                ),
        );

    let q_btn = el()
        .key("question-toggle")
        .role("switch")
        .label("Question marks")
        .checked(s.allow_question)
        .help("Enable ? marks on right-click")
        .focusable(true)
        .cursor(Cursor::Pointer)
        .size(28.0, 30.0)
        .center()
        .radius(r_outer)
        .fill(if s.allow_question { p.control_pill } else { p.control_track })
        .border(
            1.0,
            if s.allow_question { p.warning.alpha(0.55) } else { p.control_border },
        )
        .hover(p.control_hover)
        .press_scale(0.93)
        .on_press(|s: &mut AppState| s.toggle_question_marks())
        .child(
            text("?")
                .font(TextStyle::new(12.5, 600.0))
                .color(if s.allow_question { p.warning } else { p.text_muted })
                .a11y_hidden(),
        );

    let mut left_group = row().center_y().gap(6.0).child(diff_control);
    if let Some(tag) = custom_tag {
        left_group = left_group.child(tag);
    }

    row()
        .key("difficulty-bar")
        .w(width)
        .h(30.0)
        .center_y()
        .justify_between()
        .child(left_group)
        .child(row().center_y().gap(6.0).child(mode_btn).child(q_btn))
}

fn hud_deck(s: &AppState, p: &Palette, kit: &Kit, width: f32, hair: f32) -> El {
    let bombs_left = s.board.bombs_remaining();
    let bombs_text = format_counter(bombs_left);
    let timer_text = format!("{:03}", s.elapsed_secs.min(999));

    let bombs_well = led_counter_well(
        "bombs-counter",
        "Bombs remaining",
        &bombs_text,
        bombs_left as f64,
        p.led_bombs,
        canvas(art::mini_bomb_icon(p.led_bombs, p.bomb_spark, 16.0)).size(16.0, 16.0),
        p,
        kit,
    );

    let timer_well = led_counter_well(
        "timer-counter",
        "Elapsed seconds",
        &timer_text,
        s.elapsed_secs as f64,
        p.led_timer,
        canvas(art::clock_icon(p.led_timer, 16.0)).size(16.0, 16.0),
        p,
        kit,
    );

    let mood = s.mascot_mood();
    let mascot_r = if p.classic_bevel { 4.0 } else { 13.0 };
    let mut mascot_btn = el()
        .key("mascot-reset")
        .role("button")
        .label("New game")
        .help("New game (R)")
        .focusable(true)
        .cursor(Cursor::Pointer)
        .size(46.0, 46.0)
        .center()
        .radius(mascot_r)
        .fill(p.mascot_ring_bottom)
        .gradient(p.mascot_ring_top, p.mascot_ring_bottom)
        .border_gradient(1.0, p.tile_border_top, p.tile_border_bottom)
        .shadow(2.0, 6.0, p.tile_shadow)
        .hover(p.tile_hover)
        .press(p.tile_press)
        .press_scale(0.91)
        .motion_intent(MotionIntent::BOUNCE)
        .on_press_tx(|s: &mut AppState, tx| s.restart(tx))
        .child(canvas(art::mascot_face_shape(p, mood, 34.0)).size(34.0, 34.0));

    if p.classic_bevel {
        mascot_btn = mascot_btn
            .box_shadow(Shadow {
                x: 2.0,
                y: 2.0,
                blur: 0.4,
                spread: 0.0,
                color: p.tile_bevel_light,
                inset: true,
            })
            .box_shadow(Shadow {
                x: -2.0,
                y: -2.0,
                blur: 0.4,
                spread: 0.0,
                color: p.tile_bevel_dark,
                inset: true,
            });
    } else if s.board.phase == GamePhase::Won {
        mascot_btn = mascot_btn.shadow(0.0, 14.0, p.warning.alpha(0.45));
    } else if s.board.phase == GamePhase::Lost {
        mascot_btn = mascot_btn.shadow(0.0, 12.0, p.danger.alpha(0.38));
    }

    let mut deck = row()
        .key("hud-deck")
        .w(width)
        .h(62.0)
        .px(12.0)
        .center_y()
        .justify_between()
        .radius(if p.classic_bevel { 4.0 } else { 12.0 })
        .fill(p.hud_bg)
        .border(hair.max(1.0), p.hud_border)
        .child(bombs_well)
        .child(mascot_btn)
        .child(timer_well);

    if p.classic_bevel {
        deck = deck
            .box_shadow(Shadow {
                x: 1.5,
                y: 1.5,
                blur: 0.5,
                spread: 0.0,
                color: p.tray_inset_shadow,
                inset: true,
            })
            .box_shadow(Shadow {
                x: -1.5,
                y: -1.5,
                blur: 0.5,
                spread: 0.0,
                color: p.tray_inset_highlight,
                inset: true,
            });
    } else {
        deck = deck.box_shadow(Shadow {
            x: 0.0,
            y: 1.0,
            blur: 4.0,
            spread: 0.0,
            color: p.tray_inset_shadow.alpha(0.35),
            inset: true,
        });
    }

    deck
}

#[allow(clippy::too_many_arguments)]
fn led_counter_well(
    key: &str,
    a11y_label: &str,
    display: &str,
    numeric: f64,
    ink: Color,
    icon_el: El,
    p: &Palette,
    kit: &Kit,
) -> El {
    let digit_box = if p.show_led_ghost {
        el().w(56.0)
            .h(26.0)
            .center()
            .a11y_hidden()
            .child(
                canvas(art::seven_seg_display(display, ink, p.led_ghost, 52.0, 24.0))
                    .size(52.0, 24.0),
            )
    } else {
        let digit_font = TextStyle::new(22.0, 600.0)
            .features("'tnum' 1")
            .tracking(1.6)
            .lh(26.0);

        let rolling = RollingText::new(format!("{key}-digits"), display)
            .value(numeric)
            .font(digit_font)
            .color(ink)
            .spring(ODOMETER_SPRING)
            .build(kit);

        el().w(56.0).h(28.0).center().child(rolling)
    };

    row()
        .key(key)
        .role("status")
        .label(format!("{a11y_label}: {display}"))
        .w(104.0)
        .h(42.0)
        .px(11.0)
        .center_y()
        .justify_between()
        .radius(if p.classic_bevel { 3.0 } else { 9.0 })
        .fill(p.led_well)
        .border(1.0, p.led_well_border)
        .box_shadow(Shadow {
            x: 0.0,
            y: 2.0,
            blur: 6.0,
            spread: 0.0,
            color: Color::BLACK.alpha(0.55),
            inset: true,
        })
        .child(icon_el)
        .child(digit_box)
}

fn format_counter(v: i32) -> String {
    let clamped = v.clamp(-99, 999);
    if clamped < 0 {
        format!("-{:02}", (-clamped).min(99))
    } else {
        format!("{clamped:03}")
    }
}

fn progress_rail(s: &AppState, p: &Palette, width: f32) -> El {
    let pct = s.board.progress();
    let fill_color = match s.board.phase {
        GamePhase::Won => p.success,
        GamePhase::Lost => p.danger,
        GamePhase::Ready | GamePhase::Playing => p.accent,
    };
    let bar_w = if pct <= 0.0 { 0.0 } else { (width * pct).max(6.0) };

    el().key("progress-track")
        .w(width)
        .h(4.0)
        .radius(2.0)
        .fill(p.control_track)
        .clip_rounded(2.0)
        .a11y_hidden()
        .child(
            el().key("progress-fill")
                .h(4.0)
                .w(bar_w)
                .radius(2.0)
                .fill(fill_color)
                .motion(PROGRESS_SPRING),
        )
}

fn board_tray(s: &AppState, p: &Palette, hair: f32) -> El {
    let base_tile = s.board.difficulty.tile_size();
    let base_gap = s.board.difficulty.tile_gap();
    // Classic '95 tightens the gutter between beveled tiles while preserving exact board footprint.
    let (tile, gap) = if p.classic_bevel {
        let tight_gap = 1.5f32.min(base_gap);
        let cols = s.board.cols as f32;
        let orig_w = cols * base_tile + (cols - 1.0) * base_gap;
        let new_tile = ((orig_w - (cols - 1.0) * tight_gap) / cols).floor();
        (new_tile, tight_gap)
    } else {
        (base_tile, base_gap)
    };

    let mut grid_col = col().key("board-grid").gap(gap);
    for r in 0..s.board.rows {
        let mut grid_row = row().key(format!("row-{r}")).gap(gap);
        for c in 0..s.board.cols {
            grid_row = grid_row.child(tile_view(s, p, r, c, tile, hair));
        }
        grid_col = grid_col.child(grid_row);
    }

    let mut tray = el()
        .key("board-tray")
        .center()
        .padding(10.0)
        .radius(p.tray_radius)
        .fill(p.tray_bg)
        .border(1.0, p.tray_border)
        .child(grid_col);

    if p.classic_bevel {
        tray = tray
            .box_shadow(Shadow {
                x: 2.5,
                y: 2.5,
                blur: 0.5,
                spread: 0.0,
                color: p.tray_inset_shadow,
                inset: true,
            })
            .box_shadow(Shadow {
                x: -2.5,
                y: -2.5,
                blur: 0.5,
                spread: 0.0,
                color: p.tray_inset_highlight,
                inset: true,
            });
    } else {
        tray = tray
            .continuous_corners()
            .box_shadow(Shadow {
                x: 0.0,
                y: 2.0,
                blur: 8.0,
                spread: 0.0,
                color: p.tray_inset_shadow,
                inset: true,
            })
            .box_shadow(Shadow {
                x: 0.0,
                y: -1.0,
                blur: 1.0,
                spread: 0.0,
                color: p.tray_inset_highlight,
                inset: true,
            });
    }

    tray
}

fn tile_view(s: &AppState, p: &Palette, r: usize, c: usize, size: f32, hair: f32) -> El {
    let cell = s.board.cell(r, c);
    let game_over = matches!(s.board.phase, GamePhase::Won | GamePhase::Lost);
    let is_cursor = s.cursor == Some((r, c));
    let is_chord_preview = s.is_tile_chord_highlighted(r, c);
    let is_pressed_self = matches!(s.pressed, Some(PressTarget::Tile(pr, pc)) if pr == r && pc == c);
    let hint_here = s.hint.filter(|h| h.row == r && h.col == c);

    let radius = if p.classic_bevel {
        p.tile_radius
    } else {
        (p.tile_radius * (size / 38.0)).clamp(4.0, 7.0)
    };

    let a11y_desc = match cell.state {
        CellState::Hidden => format!("Row {}, Column {}, hidden", r + 1, c + 1),
        CellState::Flagged => format!("Row {}, Column {}, flagged", r + 1, c + 1),
        CellState::Question => format!("Row {}, Column {}, questioned", r + 1, c + 1),
        CellState::Revealed if cell.has_bomb => format!("Row {}, Column {}, bomb", r + 1, c + 1),
        CellState::Revealed if cell.adjacent > 0 => {
            format!("Row {}, Column {}, {} adjacent bombs", r + 1, c + 1, cell.adjacent)
        }
        CellState::Revealed => format!("Row {}, Column {}, empty", r + 1, c + 1),
    };

    let mut tile = el()
        .key(format!("cell-{r}-{c}"))
        .role("button")
        .label(a11y_desc)
        .size(size, size)
        .center()
        .radius(radius)
        .motion(TILE_SPRING);

    if !p.classic_bevel {
        tile = tile.continuous_corners();
    }

    match cell.state {
        CellState::Hidden | CellState::Flagged | CellState::Question => {
            let depressed = is_chord_preview || is_pressed_self;

            if cell.misflagged {
                tile = tile
                    .fill(p.misflag_bg)
                    .border(1.0, p.misflag_cross.alpha(0.55));
            } else if depressed {
                tile = tile
                    .fill(p.tile_depressed)
                    .border(hair.max(1.0), p.revealed_border)
                    .box_shadow(Shadow {
                        x: 0.0,
                        y: 1.5,
                        blur: 3.0,
                        spread: 0.0,
                        color: p.tray_inset_shadow.alpha(0.6),
                        inset: true,
                    });
            } else if cell.state == CellState::Flagged && s.board.phase == GamePhase::Won {
                tile = tile
                    .fill(p.victory_flag_bg)
                    .border(1.0, p.success.alpha(0.55))
                    .shadow(1.0, 2.5, p.tile_shadow);
            } else {
                let top = if cell.state == CellState::Flagged {
                    p.tile_top.mix(p.tile_flagged_tint, 0.45)
                } else {
                    p.tile_top
                };
                let bottom = if cell.state == CellState::Flagged {
                    p.tile_bottom.mix(p.tile_flagged_tint, 0.55)
                } else {
                    p.tile_bottom
                };
                tile = tile
                    .fill(bottom)
                    .gradient(top, bottom)
                    .border_gradient(1.0, p.tile_border_top, p.tile_border_bottom)
                    .shadow(1.0, 2.5, p.tile_shadow);

                if p.classic_bevel {
                    tile = tile
                        .box_shadow(Shadow {
                            x: 2.0,
                            y: 2.0,
                            blur: 0.35,
                            spread: 0.0,
                            color: p.tile_bevel_light,
                            inset: true,
                        })
                        .box_shadow(Shadow {
                            x: -2.0,
                            y: -2.0,
                            blur: 0.35,
                            spread: 0.0,
                            color: p.tile_bevel_dark,
                            inset: true,
                        });
                } else {
                    tile = tile.box_shadow(Shadow {
                        x: 0.0,
                        y: 1.0,
                        blur: 0.6,
                        spread: 0.0,
                        color: p.tile_bevel_light,
                        inset: true,
                    });
                }
            }

            if !game_over {
                tile = tile
                    .cursor(Cursor::Pointer)
                    .hover(p.tile_hover)
                    .press(p.tile_press)
                    .press_scale(0.93);
            }

            // Inner artwork for Flagged / Misflagged / Question
            let art_size = (size * 0.68).round();
            if cell.misflagged {
                tile = tile.child(
                    canvas(art::misflag_shape(p, art_size))
                        .key(format!("misflag-{}-{r}-{c}", s.game_id))
                        .size(art_size, art_size)
                        .transition(
                            Transition::scale(0.6)
                                .allow_small_scale()
                                .springs(SpringPair::new(POP_SPRING, EXIT_SPRING)),
                        )
                        .a11y_hidden(),
                );
            } else if cell.state == CellState::Flagged {
                let delay = if cell.auto_flagged {
                    (cell.reveal_wave as f32 * 0.022).min(0.32)
                } else {
                    0.0
                };
                tile = tile.child(
                    canvas(art::flag_shape(p, art_size))
                        .key(format!("flag-{}-{r}-{c}", s.game_id))
                        .size(art_size, art_size)
                        .motion_intent(MotionIntent::BOUNCE)
                        .transition(
                            Transition::scale(0.55)
                                .allow_small_scale()
                                .springs(SpringPair::new(POP_SPRING, EXIT_SPRING)),
                        )
                        .enter_delay(delay)
                        .a11y_hidden(),
                );
            } else if cell.state == CellState::Question {
                tile = tile.child(
                    canvas(art::question_shape(p.question_ink, art_size))
                        .key(format!("q-{}-{r}-{c}", s.game_id))
                        .size(art_size, art_size)
                        .transition(
                            Transition::scale(0.65)
                                .allow_small_scale()
                                .springs(SpringPair::new(POP_SPRING, EXIT_SPRING)),
                        )
                        .a11y_hidden(),
                );
            }
        }
        CellState::Revealed => {
            let checker_bg = if (r + c).is_multiple_of(2) {
                p.revealed_a
            } else {
                p.revealed_b
            };
            let delay = (cell.reveal_wave as f32 * 0.018).min(0.28);

            if cell.detonated {
                let art_size = (size * 0.78).round();
                tile = tile
                    .fill(p.blast_bottom)
                    .gradient(p.blast_top, p.blast_bottom)
                    .border(1.5, p.blast_border)
                    .shadow(0.0, 12.0, p.danger.alpha(0.55))
                    .motion_intent(MotionIntent::BOUNCE)
                    .sequence(Sequence::new(BLAST_PULSE))
                    .child(
                        canvas(art::bomb_shape(p, art_size, true))
                            .key(format!("blast-{}-{r}-{c}", s.game_id))
                            .size(art_size, art_size)
                            .a11y_hidden(),
                    );
            } else if cell.has_bomb {
                let art_size = (size * 0.68).round();
                tile = tile
                    .fill(p.bomb_revealed_bg)
                    .border(hair.max(1.0), p.revealed_border)
                    .child(
                        canvas(art::bomb_shape(p, art_size, false))
                            .key(format!("bomb-{}-{r}-{c}", s.game_id))
                            .size(art_size, art_size)
                            .motion_intent(MotionIntent::BOUNCE)
                            .transition(
                                Transition::scale(0.58)
                                    .allow_small_scale()
                                    .springs(SpringPair::new(POP_SPRING, EXIT_SPRING)),
                            )
                            .enter_delay(delay)
                            .a11y_hidden(),
                    );
            } else {
                tile = tile
                    .fill(checker_bg)
                    .border(hair.max(1.0), p.revealed_border);

                if cell.adjacent > 0 {
                    let can_chord_now = s.board.can_chord(r, c);
                    if !game_over {
                        tile = tile.hover(p.revealed_hover);
                        if can_chord_now {
                            tile = tile.cursor(Cursor::Pointer);
                        }
                    }

                    let num_size = (size * 0.56).round().clamp(14.0, 22.0);
                    let num_color = p.number_color(cell.adjacent);
                    tile = tile.child(
                        text(cell.adjacent.to_string())
                            .key(format!("num-{}-{r}-{c}", s.game_id))
                            .font(TextStyle::new(num_size, 600.0).lh(num_size * 1.05))
                            .color(num_color)
                            .transition(
                                Transition::scale(0.75)
                                    .allow_small_scale()
                                    .springs(SpringPair::new(Spring::new(0.22, 0.14), EXIT_SPRING)),
                            )
                            .enter_delay(delay)
                            .a11y_hidden(),
                    );
                }
            }
        }
    }

    // Hint glowing target ring overlay
    if let Some(hint) = hint_here {
        let ring_color = match hint.kind {
            HintKind::Safe => p.success,
            HintKind::Bomb => p.warning,
        };
        tile = tile.child(
            el().key(format!("hint-ring-{r}-{c}"))
                .pin(Some(0.5), Some(0.5), Some(0.5), Some(0.5))
                .radius((radius - 0.5).max(1.5))
                .border(2.5, ring_color)
                .fill(ring_color.alpha(0.22))
                .shadow(0.0, 10.0, ring_color.alpha(0.55))
                .inert()
                .motion_intent(MotionIntent::PULSE)
                .sequence(Sequence::new(HINT_PULSE))
                .a11y_hidden(),
        );
    }

    // Keyboard navigation focus ring
    if is_cursor {
        tile = tile.outline(2.0, 1.0, p.focus_ring);
    }

    // Wire tactile pointer interactions while game is active
    if !game_over {
        tile = tile
            .on_drag(move |s: &mut AppState, d: &mut Drag, tx: &mut Tx| {
                if s.modal != Modal::None {
                    return;
                }
                let inside = d.x >= 0.0 && d.x <= d.w && d.y >= 0.0 && d.y <= d.h;
                let cell = s.board.cell(r, c);
                let target = match cell.state {
                    CellState::Revealed if cell.adjacent > 0 => Some(PressTarget::Chord(r, c)),
                    CellState::Hidden | CellState::Question => Some(PressTarget::Tile(r, c)),
                    _ => None,
                };
                match d.phase {
                    Phase::Down => {
                        s.cursor = None;
                        if s.pressed != target {
                            s.pressed = target;
                        } else {
                            tx.live_only();
                        }
                    }
                    Phase::Move => {
                        let next = if inside { target } else { None };
                        if s.pressed != next {
                            s.pressed = next;
                        } else {
                            tx.live_only();
                        }
                    }
                    Phase::Up => {
                        s.pressed = None;
                        if !d.cancelled() && inside {
                            s.handle_primary_tile(r, c, tx);
                        }
                    }
                }
            })
            .on_secondary_press(move |s: &mut AppState, _, tx: &mut Tx| {
                s.cursor = None;
                s.pressed = None;
                s.handle_secondary_tile(r, c, tx);
            })
            .on_middle_press(move |s: &mut AppState, tx: &mut Tx| {
                s.cursor = None;
                s.pressed = None;
                s.perform_chord(r, c, tx);
            });
    }

    tile
}

fn footer_bar(s: &AppState, p: &Palette, width: f32) -> El {
    let r = if p.classic_bevel { 4.0 } else { 10.0 };

    match s.board.phase {
        GamePhase::Won => {
            let time_str = format!("{:.2}s", s.exact_secs);
            let bv_str = format!("{:.1} 3BV/s", s.bv_per_sec());
            let eff_str = format!("{}% eff", s.efficiency_pct());

            let mut left = row()
                .center_y()
                .gap(7.0)
                .child(canvas(art::crown_icon(p.warning, 16.0)).size(16.0, 16.0))
                .child(
                    text(format!("Cleared in {time_str}"))
                        .font(TextStyle::new(12.5, 600.0).features("'tnum' 1"))
                        .color(p.text),
                );

            if s.new_record {
                left = left.child(
                    el().key("new-best-badge")
                        .px(6.0)
                        .py(2.0)
                        .radius(4.0)
                        .fill(p.warning.alpha(0.22))
                        .border(1.0, p.warning.alpha(0.55))
                        .child(
                            text("NEW BEST")
                                .font(TextStyle::new(9.5, 600.0).tracking(0.6))
                                .color(p.warning),
                        ),
                );
            } else if width >= 500.0 {
                left = left.child(
                    text(format!("·  {bv_str}  ·  {eff_str}"))
                        .font(TextStyle::new(11.0, 500.0).features("'tnum' 1"))
                        .color(p.text_muted),
                );
            }

            row()
                .key("footer-won")
                .w(width)
                .h(38.0)
                .px(12.0)
                .center_y()
                .justify_between()
                .radius(r)
                .fill(p.success.alpha(if p.kind.is_dark() { 0.16 } else { 0.14 }))
                .border(1.0, p.success.alpha(0.45))
                .transition(Transition::opacity().combined(Transition::offset(0.0, 4.0)))
                .child(left)
                .child(action_pill("play-again", "Play Again", p.success, p))
        }
        GamePhase::Lost => {
            let pct = (s.board.progress() * 100.0).round() as u32;
            row()
                .key("footer-lost")
                .w(width)
                .h(38.0)
                .px(12.0)
                .center_y()
                .justify_between()
                .radius(r)
                .fill(p.danger.alpha(if p.kind.is_dark() { 0.16 } else { 0.14 }))
                .border(1.0, p.danger.alpha(0.45))
                .transition(Transition::opacity().combined(Transition::offset(0.0, 4.0)))
                .child(
                    row()
                        .center_y()
                        .gap(7.0)
                        .child(
                            canvas(art::mini_bomb_icon(p.danger, p.bomb_spark, 15.0))
                                .size(15.0, 15.0),
                        )
                        .child(
                            text(format!("Boom!  {pct}% cleared"))
                                .font(TextStyle::new(12.5, 600.0).features("'tnum' 1"))
                                .color(p.text),
                        ),
                )
                .child(action_pill("try-again", "Try Again", p.danger, p))
        }
        GamePhase::Ready | GamePhase::Playing => {
            if let Some(hint) = s.hint {
                let tint = match hint.kind {
                    HintKind::Safe => p.success,
                    HintKind::Bomb => p.warning,
                };
                row()
                    .key("footer-hint")
                    .w(width)
                    .h(38.0)
                    .px(12.0)
                    .center_y()
                    .justify_between()
                    .radius(r)
                    .fill(tint.alpha(0.14))
                    .border(1.0, tint.alpha(0.42))
                    .child(
                        row()
                            .center_y()
                            .gap(7.0)
                            .child(canvas(art::sparkle_icon(tint, 14.0)).size(14.0, 14.0))
                            .child(
                                text(hint.reason)
                                    .font(TextStyle::new(11.5, 500.0))
                                    .color(p.text),
                            ),
                    )
                    .child(
                        text(format!("R{} C{}", hint.row + 1, hint.col + 1))
                            .font(TextStyle::new(11.0, 600.0).features("'tnum' 1"))
                            .color(tint),
                    )
            } else {
                let left_text = if s.board.phase == GamePhase::Ready {
                    if let Some(best_ms) = s.stats.best_for(s.board.difficulty) {
                        format!("Best: {}  ·  Click any tile to begin", format_ms(best_ms))
                    } else {
                        "Click any tile to begin — first click is always safe".to_owned()
                    }
                } else {
                    format!(
                        "Cleared {} / {}  ·  Click numbers to chord",
                        s.board.revealed_safe,
                        s.board.safe_total()
                    )
                };
                let right_text = if width >= 540.0 {
                    "Right-click flag · H hint · R reset"
                } else {
                    "R reset · H hint"
                };

                row()
                    .key("footer-status")
                    .w(width)
                    .h(38.0)
                    .px(4.0)
                    .center_y()
                    .justify_between()
                    .child(
                        text(left_text)
                            .font(TextStyle::new(11.5, 500.0).features("'tnum' 1"))
                            .color(p.text_muted),
                    )
                    .child(
                        text(right_text)
                            .font(TextStyle::new(11.0, 500.0))
                            .color(p.text_faint),
                    )
            }
        }
    }
}

fn action_pill(key: &str, label: &str, accent: Color, p: &Palette) -> El {
    let r = if p.classic_bevel { 3.5 } else { 7.0 };
    let ink = if accent.luminance() > 0.36 {
        rune::oklch(0.14, 0.01, 260.0)
    } else {
        Color::WHITE
    };
    el().key(key)
        .role("button")
        .label(label)
        .focusable(true)
        .cursor(Cursor::Pointer)
        .h(26.0)
        .px(12.0)
        .center()
        .radius(r)
        .fill(accent)
        .hover(Color::WHITE.alpha(0.14))
        .press(Color::BLACK.alpha(0.14))
        .press_scale(0.95)
        .on_press_tx(|s: &mut AppState, tx| s.restart(tx))
        .child(
            text(label)
                .font(TextStyle::new(11.5, 600.0))
                .color(ink)
                .a11y_hidden(),
        )
}

fn stats_modal(s: &AppState, p: &Palette, hair: f32) -> El {
    let fmt_opt = |ms: Option<u32>| ms.map(format_ms).unwrap_or_else(|| "—".to_owned());

    let record_row = |label: &str, dims: &str, value: String| {
        row()
            .w_full()
            .h(34.0)
            .px(10.0)
            .center_y()
            .justify_between()
            .radius(7.0)
            .fill(p.control_track)
            .child(
                row()
                    .center_y()
                    .gap(6.0)
                    .child(text(label).font(TextStyle::new(12.5, 600.0)).color(p.text))
                    .child(text(dims).font(TextStyle::new(11.0, 500.0)).color(p.text_faint)),
            )
            .child(
                text(value)
                    .font(TextStyle::new(13.0, 600.0).features("'tnum' 1"))
                    .color(p.warning),
            )
    };

    let stat_tile = |label: &str, value: String| {
        col()
            .grow(1.0)
            .py(10.0)
            .center()
            .gap(2.0)
            .radius(8.0)
            .fill(p.control_track)
            .child(
                text(value)
                    .font(TextStyle::new(17.0, 600.0).features("'tnum' 1"))
                    .color(p.text),
            )
            .child(text(label).font(TextStyle::new(10.5, 500.0)).color(p.text_muted))
    };

    let card = col()
        .key("stats-dialog")
        .role("dialog")
        .label("Best Times and Statistics")
        .modal(true, |s: &mut AppState| s.modal = Modal::None)
        .w(340.0)
        .padding(18.0)
        .gap(14.0)
        .radius(if p.classic_bevel { 6.0 } else { 15.0 })
        .fill(p.modal_bg)
        .border(hair.max(1.0), p.control_border)
        .shadow(10.0, 32.0, Color::BLACK.alpha(0.45))
        .transition(Transition::scale(0.94))
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(
                    row()
                        .center_y()
                        .gap(8.0)
                        .child(canvas(art::trophy_icon(p.warning, 18.0)).size(18.0, 18.0))
                        .child(
                            text("Best Times & Stats")
                                .font(TextStyle::new(14.5, 600.0))
                                .color(p.text),
                        ),
                )
                .child(close_chip(p)),
        )
        .child(
            col()
                .w_full()
                .gap(6.0)
                .child(record_row("Beginner", "9×9", fmt_opt(s.stats.best_beginner_ms)))
                .child(record_row("Intermediate", "16×16", fmt_opt(s.stats.best_intermediate_ms)))
                .child(record_row("Expert", "30×16", fmt_opt(s.stats.best_expert_ms))),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .child(stat_tile("Played", s.stats.played.to_string()))
                .child(stat_tile("Win Rate", format!("{}%", s.stats.win_rate_pct())))
                .child(stat_tile("Best Streak", s.stats.best_streak.to_string())),
        )
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(
                    el().key("reset-stats")
                        .role("button")
                        .label("Reset stats")
                        .focusable(true)
                        .cursor(Cursor::Pointer)
                        .px(10.0)
                        .h(28.0)
                        .center()
                        .radius(7.0)
                        .hover(p.danger.alpha(0.14))
                        .on_press(|s: &mut AppState| s.reset_stats())
                        .child(text("Reset Stats").font(TextStyle::new(11.5, 500.0)).color(p.danger)),
                )
                .child(
                    el().key("stats-done")
                        .role("button")
                        .label("Done")
                        .default_button()
                        .focusable(true)
                        .cursor(Cursor::Pointer)
                        .px(14.0)
                        .h(28.0)
                        .center()
                        .radius(7.0)
                        .fill(p.accent)
                        .press_scale(0.95)
                        .on_press(|s: &mut AppState| s.modal = Modal::None)
                        .child(text("Done").font(TextStyle::new(11.5, 600.0)).color(Color::WHITE)),
                ),
        );

    el().key("stats-scrim")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .center()
        .fill(p.modal_scrim)
        .transition(Transition::opacity())
        .child(card)
}

fn custom_modal(s: &AppState, p: &Palette, hair: f32) -> El {
    let total = s.custom_cols * s.custom_rows;
    let density = (s.custom_bombs as f32 / total as f32) * 100.0;

    let stepper_row = |key: &str,
                       label: &str,
                       val: usize,
                       on_dec: fn(&mut AppState),
                       on_inc: fn(&mut AppState)| {
        let step_btn = |k: &str, sym: &str, f: fn(&mut AppState)| {
            el().key(k)
                .role("button")
                .label(format!("{label} {sym}"))
                .focusable(true)
                .cursor(Cursor::Pointer)
                .size(28.0, 26.0)
                .center()
                .radius(6.0)
                .fill(p.control_pill)
                .border(1.0, p.control_border)
                .hover(p.control_hover)
                .press_scale(0.92)
                .on_press(f)
                .child(text(sym).font(TextStyle::new(14.0, 600.0)).color(p.text).a11y_hidden())
        };

        row()
            .w_full()
            .h(38.0)
            .px(10.0)
            .center_y()
            .justify_between()
            .radius(8.0)
            .fill(p.control_track)
            .child(text(label).font(TextStyle::new(12.5, 500.0)).color(p.text))
            .child(
                row()
                    .center_y()
                    .gap(8.0)
                    .child(step_btn(&format!("{key}-dec"), "−", on_dec))
                    .child(
                        text(val.to_string())
                            .w(32.0)
                            .align(Align::Center)
                            .font(TextStyle::new(13.5, 600.0).features("'tnum' 1"))
                            .color(p.text),
                    )
                    .child(step_btn(&format!("{key}-inc"), "+", on_inc)),
            )
    };

    let card = col()
        .key("custom-dialog")
        .role("dialog")
        .label("Custom Board")
        .modal(true, |s: &mut AppState| s.modal = Modal::None)
        .w(330.0)
        .padding(18.0)
        .gap(13.0)
        .radius(if p.classic_bevel { 6.0 } else { 15.0 })
        .fill(p.modal_bg)
        .border(hair.max(1.0), p.control_border)
        .shadow(10.0, 32.0, Color::BLACK.alpha(0.45))
        .transition(Transition::scale(0.94))
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(
                    row()
                        .center_y()
                        .gap(8.0)
                        .child(canvas(art::sliders_icon(p.accent, 16.0)).size(16.0, 16.0))
                        .child(
                            text("Custom Board")
                                .font(TextStyle::new(14.5, 600.0))
                                .color(p.text),
                        ),
                )
                .child(close_chip(p)),
        )
        .child(
            col()
                .w_full()
                .gap(7.0)
                .child(stepper_row(
                    "cols",
                    "Columns",
                    s.custom_cols,
                    |s| s.adjust_custom(-1, 0, 0),
                    |s| s.adjust_custom(1, 0, 0),
                ))
                .child(stepper_row(
                    "rows",
                    "Rows",
                    s.custom_rows,
                    |s| s.adjust_custom(0, -1, 0),
                    |s| s.adjust_custom(0, 1, 0),
                ))
                .child(stepper_row(
                    "bombs",
                    "Bombs",
                    s.custom_bombs,
                    |s| s.adjust_custom(0, 0, -2),
                    |s| s.adjust_custom(0, 0, 2),
                )),
        )
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(
                    text(format!("Density: {density:.1}% ({total} tiles)"))
                        .font(TextStyle::new(11.0, 500.0).features("'tnum' 1"))
                        .color(p.text_muted),
                )
                .child(spacer())
                .child(
                    el().key("apply-custom")
                        .role("button")
                        .label("Start Custom Game")
                        .default_button()
                        .focusable(true)
                        .cursor(Cursor::Pointer)
                        .px(14.0)
                        .h(30.0)
                        .center()
                        .radius(7.5)
                        .fill(p.accent)
                        .hover(Color::WHITE.alpha(0.12))
                        .press_scale(0.95)
                        .on_press_tx(|s: &mut AppState, tx| s.apply_custom(tx))
                        .child(
                            text("Start Game")
                                .font(TextStyle::new(12.0, 600.0))
                                .color(Color::WHITE),
                        ),
                ),
        );

    el().key("custom-scrim")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .center()
        .fill(p.modal_scrim)
        .transition(Transition::opacity())
        .child(card)
}

fn close_chip(p: &Palette) -> El {
    el().key("modal-close")
        .role("button")
        .label("Close")
        .focusable(true)
        .cursor(Cursor::Pointer)
        .size(24.0, 24.0)
        .center()
        .radius(6.0)
        .fill(p.control_track)
        .hover(p.control_hover)
        .on_press(|s: &mut AppState| s.modal = Modal::None)
        .child(text("×").font(TextStyle::new(15.0, 600.0)).color(p.text_muted).a11y_hidden())
}
