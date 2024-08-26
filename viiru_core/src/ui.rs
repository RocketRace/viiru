use std::{collections::HashMap, io::stdout};

use crossterm::{
    cursor::{Hide, MoveDown, MoveLeft, MoveTo, Show},
    queue,
    style::{
        Attribute, Color, Colors, Print, ResetColor, SetAttribute, SetColors, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use crate::{
    block::{Field, Input},
    opcodes::BLOCKS,
    result::ViiruResult,
    runtime::{Runtime, State},
    spec::{Fragment, Shape},
    util::parse_rgb,
};

pub fn in_terminal_scope<F>(f: F) -> ViiruResult
where
    F: FnOnce() -> ViiruResult,
{
    stdout().execute(EnterAlternateScreen)?.execute(Hide)?;
    enable_raw_mode()?;
    f()?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?.execute(Show)?;
    Ok(())
}

pub fn print_in_view(
    runtime: &Runtime,
    x: i32,
    y: i32,
    text: &str,
    colors: Colors,
    fake: bool,
) -> ViiruResult<()> {
    let screen_x = x - runtime.scroll_x;
    let screen_y = y - runtime.scroll_y;
    // bypass check for toolbox, only care about going off screen to the right
    if fake {
        if screen_x >= runtime.window_cols as i32 {
            return Ok(());
        }
        let visible_chars = runtime.window_cols as i32 - screen_x;
        let chopped: String = text.chars().take(visible_chars as usize).collect();
        if !chopped.is_empty() {
            queue!(
                stdout(),
                MoveTo(screen_x as u16, screen_y as u16),
                SetColors(colors),
                Print(chopped)
            )?;
        }
        return Ok(());
    }
    // no point even trying if our starting point is bad
    if screen_y < runtime.viewport.y_min
        || screen_y >= runtime.viewport.y_max
        || screen_x >= runtime.viewport.x_max
    {
        return Ok(());
    }
    // todo: technically this should use grapheme clusters, same with .chars() everywhere below
    if screen_x >= runtime.viewport.x_min {
        let visible_chars = runtime.viewport.x_max - screen_x;
        let chopped: String = text.chars().take(visible_chars as usize).collect();
        queue!(
            stdout(),
            MoveTo(screen_x as u16, screen_y as u16),
            SetColors(colors)
        )?;
        if !chopped.is_empty() {
            queue!(stdout(), Print(chopped))?;
        }
    } else {
        let hidden_chars = runtime.viewport.x_min - screen_x;
        let chopped: String = text.chars().skip(hidden_chars as usize).collect();
        queue!(
            stdout(),
            MoveTo(runtime.viewport.x_min as u16, screen_y as u16),
            SetColors(colors),
        )?;
        if !chopped.is_empty() {
            queue!(stdout(), Print(chopped))?;
        }
    }
    Ok(())
}

fn add_row_to_cache(
    block_id: &str,
    x: i32,
    y: i32,
    width: i32,
    placement_grid: &mut HashMap<(i32, i32), Vec<String>>,
) {
    for i in 0..width {
        placement_grid
            .entry((x + i, y))
            .or_default()
            .push(block_id.to_string())
    }
}

fn mark_block_offset(
    block_id: &str,
    dx: i32,
    dy: i32,
    offset_mapping: &mut HashMap<String, (i32, i32)>,
) {
    offset_mapping.insert(block_id.to_string(), (dx, dy));
}

/// returns either dx or dy depending on the block shape (expression or stack)
pub fn draw_block(
    runtime: &Runtime,
    block_id: &str,
    x: i32,
    y: i32,
    placement_grid: &mut HashMap<(i32, i32), Vec<String>>,
    block_offset_mapping: &mut HashMap<String, (i32, i32)>,
    fake: bool,
) -> ViiruResult<i32> {
    let block = &runtime.blocks[block_id];
    let spec = &BLOCKS[&block.opcode];
    let mut max_width = 0;
    let mut dx = 0;
    let mut dy = 0;

    let block_colors = Colors::new(
        Color::Rgb {
            r: spec.text_color.0,
            g: spec.text_color.1,
            b: spec.text_color.2,
        },
        Color::Rgb {
            r: spec.block_color.0,
            g: spec.block_color.1,
            b: spec.block_color.2,
        },
    );

    let alt_colors = Colors::new(
        Color::Rgb {
            r: spec.text_color.0,
            g: spec.text_color.1,
            b: spec.text_color.2,
        },
        Color::Rgb {
            r: spec.alt_color.0,
            g: spec.alt_color.1,
            b: spec.alt_color.2,
        },
    );

    if spec.is_hat {
        dy += 1;
    }

    let delimeters = match spec.shape {
        Shape::Circle => ("(", ")"),
        Shape::Hexagon => ("<", ">"),
        Shape::Stack => (" ", " "),
    };

    let mut skip_padding = false;
    for line in &spec.lines {
        print_in_view(runtime, x, y + dy, delimeters.0, block_colors, fake)?;
        let d_count = delimeters.0.chars().count();
        add_row_to_cache(block_id, x, y + dy, d_count as i32, placement_grid);
        dx += d_count as i32;
        max_width = max_width.max(dx);
        for frag in line {
            match frag {
                Fragment::Text(t) => {
                    print_in_view(runtime, x + dx, y + dy, t, block_colors, fake)?;
                    let d_count = t.chars().count() as i32;
                    add_row_to_cache(block_id, x + dx, y + dy, d_count, placement_grid);
                    dx += d_count;
                    max_width = max_width.max(dx);
                }
                Fragment::StrumberInput(input_name, _default) => {
                    let Input {
                        shadow_id,
                        block_id: child_id,
                    } = &block.inputs[input_name];
                    let is_not_covered = child_id.is_none();
                    let topmost = child_id.clone().or(shadow_id.clone());
                    if let Some(input_id) = topmost {
                        let delta = draw_block(
                            runtime,
                            &input_id,
                            x + dx,
                            y + dy,
                            placement_grid,
                            block_offset_mapping,
                            fake,
                        )?;
                        block_offset_mapping.insert(input_id, (dx, dy));

                        if is_not_covered {
                            add_row_to_cache(block_id, x + dx, y + dy, delta, placement_grid);
                        }
                        dx += delta;
                        max_width = max_width.max(dx);
                    } else {
                        print_in_view(runtime, x + dx, y + dy, "()", alt_colors, fake)?;
                        add_row_to_cache(block_id, x + dx, y + dy, 2, placement_grid);
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                }
                Fragment::BooleanInput(input_name) => {
                    if let Some(child_id) = &block.inputs[input_name].block_id {
                        let delta = draw_block(
                            runtime,
                            child_id,
                            x + dx,
                            y + dy,
                            placement_grid,
                            block_offset_mapping,
                            fake,
                        )?;
                        block_offset_mapping.insert(child_id.to_string(), (dx, dy));
                        dx += delta;
                        max_width = max_width.max(dx);
                    } else {
                        print_in_view(runtime, x + dx, y + dy, "<>", alt_colors, fake)?;
                        add_row_to_cache(block_id, x + dx, y + dy, 2, placement_grid);
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                }
                Fragment::BlockInput(input_name) => {
                    if let Some(child_id) = &block.inputs[input_name].block_id {
                        // - 1 because we already have 1 cell available
                        let stack_height = draw_block(
                            runtime,
                            child_id,
                            x + 1,
                            y + dy,
                            placement_grid,
                            block_offset_mapping,
                            fake,
                        )? - 1;
                        block_offset_mapping.insert(child_id.to_string(), (1, dy));
                        for y_range in 1..=stack_height {
                            print_in_view(runtime, x, y + dy + y_range, " ", block_colors, fake)?;
                            add_row_to_cache(block_id, x, y + dy + y_range, 1, placement_grid);
                        }
                        dy += stack_height;
                    }
                    skip_padding = true;
                }
                Fragment::Dropdown(field) => {
                    // todo: dropdowns are not implemented
                    print_in_view(
                        runtime,
                        x + dx,
                        y + dy,
                        &format!("[{field}]"),
                        block_colors,
                        fake,
                    )?;
                    let d_count = 2 + field.chars().count() as i32;
                    add_row_to_cache(block_id, x + dx, y + dy, d_count, placement_grid);
                    // TODO: add interactors
                    dx += d_count;
                }
                Fragment::Expander => {
                    print_in_view(
                        runtime,
                        x + dx,
                        y + dy,
                        // - 1 because we already have the left delimiter
                        &" ".repeat(max_width as usize - 1),
                        block_colors,
                        fake,
                    )?;
                    add_row_to_cache(block_id, x + dx, y + dy, max_width - 1, placement_grid);
                    skip_padding = true;
                    dx = max_width;
                }
                Fragment::Flag => {
                    let flag_color = Colors::new(
                        Color::Rgb {
                            r: 0x6d,
                            g: 0xbf,
                            b: 0x63,
                        },
                        Color::Rgb {
                            r: spec.block_color.0,
                            g: spec.block_color.1,
                            b: spec.block_color.2,
                        },
                    );
                    // todo: ascii equivalent?
                    print_in_view(runtime, x + dx, y + dy, "▸", flag_color, fake)?;
                    add_row_to_cache(block_id, x + dx, y + dy, 1, placement_grid);
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Clockwise => {
                    print_in_view(runtime, x + dx, y + dy, "↻", block_colors, fake)?;
                    add_row_to_cache(block_id, x + dx, y + dy, 1, placement_grid);
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Anticlockwise => {
                    print_in_view(runtime, x + dx, y + dy, "↺", block_colors, fake)?;
                    add_row_to_cache(block_id, x + dx, y + dy, 1, placement_grid);
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::FieldText(field) => {
                    let Field { text, .. } = block.fields.get(field).unwrap();
                    print_in_view(runtime, x + dx, y + dy, text, block_colors, fake)?;
                    let d_count = text.chars().count() as i32;
                    add_row_to_cache(block_id, x + dx, y + dy, d_count, placement_grid);
                    dx += d_count;
                    max_width = max_width.max(dx);
                }
                Fragment::WritableFieldText(field) => {
                    let Field { text, .. } = block.fields.get(field).unwrap();
                    print_in_view(runtime, x + dx, y + dy, text, block_colors, fake)?;
                    let d_count = text.chars().count() as i32;
                    add_row_to_cache(block_id, x + dx, y + dy, d_count, placement_grid);
                    // TODO: add interactors
                    dx += d_count;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomColour(field) => {
                    let Field {
                        text: rgb_string, ..
                    } = block.fields.get(field).unwrap();
                    // #RRGGBB format
                    let (r, g, b) = parse_rgb(&rgb_string[1..]);
                    let custom_colours = Colors::new(Color::Reset, Color::Rgb { r, g, b });
                    print_in_view(runtime, x + dx, y + dy, "  ", custom_colours, fake)?;
                    add_row_to_cache(block_id, x + dx, y + dy, 2, placement_grid);
                    // TODO: add interactors
                    dx += 2;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomBlock(custom) => todo!("custom block"),
            }
        }
        if !skip_padding {
            print_in_view(runtime, x + dx, y + dy, delimeters.1, block_colors, fake)?;
            let d_count = delimeters.1.chars().count() as i32;
            add_row_to_cache(block_id, x + dx, y + dy, d_count, placement_grid);
            dx += d_count;
            max_width = max_width.max(dx);
        }
        skip_padding = false;

        if let Shape::Stack = spec.shape {
            dy += 1;
            dx = 0;
        }
    }

    if spec.is_hat {
        print_in_view(
            runtime,
            x,
            y,
            &" ".repeat(max_width as usize),
            block_colors,
            fake,
        )?;
        add_row_to_cache(block_id, x, y, max_width, placement_grid);
    }

    if let Some(next_id) = &block.next_id {
        dy += draw_block(
            runtime,
            next_id,
            x,
            y + dy,
            placement_grid,
            block_offset_mapping,
            fake,
        )?;
        block_offset_mapping.insert(next_id.to_string(), (0, dy));
    }
    queue!(stdout(), ResetColor)?;

    if let Shape::Stack = spec.shape {
        Ok(dy)
    } else {
        Ok(max_width)
    }
}

pub fn draw_viewport_border(runtime: &Runtime) -> ViiruResult<()> {
    let vp = &runtime.viewport;
    let border_color = Color::Rgb {
        r: 0x7f,
        g: 0x3e,
        b: 0xcf,
    };
    queue!(
        stdout(),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(border_color),
        MoveTo(vp.x_min as u16, vp.y_min as u16 - 1),
        Print("-".repeat((vp.x_max - vp.x_min) as usize)),
        MoveTo(vp.x_min as u16, vp.y_max as u16),
        Print("-".repeat((vp.x_max - vp.x_min) as usize)),
        MoveTo(vp.x_min as u16 - 1, vp.y_min as u16 - 1),
    )?;
    for _ in vp.y_min..vp.y_max + 1 {
        queue!(stdout(), Print("|"), MoveDown(1), MoveLeft(1))?;
    }
    queue!(stdout(), MoveTo(vp.x_max as u16, vp.y_min as u16 - 1))?;
    for _ in vp.y_min..vp.y_max + 1 {
        queue!(stdout(), Print("|"), MoveDown(1), MoveLeft(1))?;
    }
    queue!(
        stdout(),
        MoveTo(vp.x_min as u16 - 1, vp.y_min as u16 - 1),
        Print("."),
        MoveTo(vp.x_max as u16, vp.y_min as u16 - 1),
        Print("."),
        MoveTo(vp.x_min as u16 - 1, vp.y_max as u16),
        Print("'"),
        MoveTo(vp.x_max as u16, vp.y_max as u16),
        Print("'"),
        SetAttribute(Attribute::NormalIntensity),
        ResetColor
    )?;
    Ok(())
}

pub fn draw_cursor_lines(runtime: &Runtime) -> ViiruResult<()> {
    let vp = &runtime.viewport;
    let screen_x = runtime.cursor_x - runtime.scroll_x;
    let screen_y = runtime.cursor_y - runtime.scroll_y;
    if vp.y_min <= screen_y && screen_y < vp.y_max {
        queue!(
            stdout(),
            MoveTo(vp.x_min as u16, screen_y as u16),
            Print("-".repeat((vp.x_max - vp.x_min) as usize)),
        )?;
    }
    if vp.x_min <= screen_x && screen_x < vp.x_max {
        queue!(stdout(), MoveTo(screen_x as u16, vp.y_min as u16))?;
        for _ in vp.y_min..vp.y_max {
            queue!(stdout(), Print("|"), MoveDown(1), MoveLeft(1))?;
        }
    }
    Ok(())
}

pub fn draw_cursor(runtime: &Runtime) -> ViiruResult<()> {
    let cursor_color = Colors::new(Color::Black, Color::White);
    queue!(stdout(), SetAttribute(Attribute::Bold))?;
    print_in_view(
        runtime,
        runtime.cursor_x,
        runtime.cursor_y,
        "+",
        cursor_color,
        false,
    )?;
    queue!(
        stdout(),
        ResetColor,
        SetAttribute(Attribute::NormalIntensity)
    )?;
    Ok(())
}

pub fn draw_marker_dots(runtime: &Runtime) -> ViiruResult<()> {
    let x_spacing = 10;
    let y_spacing = 5;
    let x_first = runtime.scroll_x / x_spacing * x_spacing;
    let y_first = runtime.scroll_y / y_spacing * y_spacing;

    let dot_color = Colors::new(Color::DarkGrey, Color::Reset);

    for dx in 0..1 + (runtime.viewport.x_max - runtime.viewport.x_min) / x_spacing {
        for dy in 0..2 + (runtime.viewport.y_max - runtime.viewport.y_min) / y_spacing {
            print_in_view(
                runtime,
                x_first + dx * x_spacing,
                y_first + dy * y_spacing,
                ".",
                dot_color,
                false,
            )?;
        }
    }
    queue!(stdout(), ResetColor)?;

    Ok(())
}

pub fn draw_toolbox(
    runtime: &mut Runtime,
    left_border: i32,
    top_border: i32,
    // HACK: replace this whole nonsense system with a proper toolbox viewport
    recompute: bool,
) -> ViiruResult<()> {
    // + 1 for border, + 1 for blank, + 1 for potential cursor, + 2 for numbers, + 1 for blank
    let offset_x = runtime.scroll_x + left_border + runtime.viewport.width() + 6;
    let offset_y = runtime.scroll_y + top_border;
    let mut dy = 0;

    for (i, id) in runtime
        .toolbox
        .iter()
        .enumerate()
        .skip(runtime.toolbox_scroll)
    {
        let mut unused = HashMap::new();
        let mut unused_2 = HashMap::new();
        let shape = BLOCKS[&runtime.blocks[id].opcode].shape;
        let i_str = if i == runtime.toolbox_cursor {
            if runtime.state == State::Toolbox {
                format!(">{i}")
            } else {
                format!("={i}")
            }
        } else {
            i.to_string()
        };
        let colors = Colors::new(Color::Reset, Color::Reset);
        if !recompute {
            print_in_view(
                runtime,
                offset_x - 1 - i_str.len() as i32,
                offset_y + dy,
                &i_str,
                colors,
                true,
            )?;
        }
        let delta = draw_block(
            runtime,
            id,
            offset_x,
            offset_y + dy,
            &mut unused,
            &mut unused_2,
            !recompute,
        )?;
        if !recompute {
            queue!(stdout(), ResetColor)?;
        }
        if let Shape::Stack = shape {
            dy += delta;
        } else {
            dy += 1;
        }
        if dy >= runtime.viewport.height() {
            runtime.toolbox_visible_max = i;
            break;
        }
    }
    Ok(())
}
