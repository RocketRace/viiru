use std::io::stdout;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Color, Colors, Print, ResetColor, SetColors},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use crate::{
    block::{Field, Input},
    opcodes::BLOCKS,
    result::ViiruResult,
    runtime::Runtime,
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
    state: &Runtime,
    x: i32,
    y: i32,
    text: &str,
    colors: Colors,
) -> ViiruResult<()> {
    let screen_x = x - state.scroll_x;
    let screen_y = y - state.scroll_y;
    // no point even trying if our starting point is bad
    if screen_y < state.viewport.y_min
        || screen_y >= state.viewport.y_max
        || screen_x >= state.viewport.x_max
    {
        return Ok(());
    }
    // todo: technically this should use grapheme clusters, same with .chars() everywhere below
    if screen_x >= state.viewport.x_min {
        let visible_chars = state.viewport.x_max - screen_x;
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
        let hidden_chars = state.viewport.x_min - screen_x;
        let chopped: String = text.chars().skip(hidden_chars as usize).collect();
        queue!(
            stdout(),
            MoveTo(state.viewport.x_min as u16, screen_y as u16),
            SetColors(colors),
        )?;
        if !chopped.is_empty() {
            queue!(stdout(), Print(chopped))?;
        }
    }
    Ok(())
}

/// returns either dx or dy depending on the block shape (expression or stack)
pub fn draw_block(state: &Runtime, block_id: &str, x: i32, y: i32) -> ViiruResult<i32> {
    let block = state.blocks.get(block_id).unwrap();
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
        print_in_view(state, x, y + dy, delimeters.0, block_colors)?;
        dx += delimeters.0.chars().count() as i32;
        max_width = max_width.max(dx);
        for frag in line {
            match frag {
                Fragment::Text(t) => {
                    print_in_view(state, x + dx, y + dy, t, block_colors)?;
                    dx += t.chars().count() as i32;
                    max_width = max_width.max(dx);
                }
                Fragment::StrumberInput(input_name, _default) => {
                    let Input {
                        shadow_id,
                        block_id,
                    } = &block.inputs[input_name];
                    let topmost = block_id.clone().or(shadow_id.clone());
                    if let Some(input_id) = topmost {
                        let delta = draw_block(state, &input_id, x + dx, y + dy)?;
                        dx += delta;
                        max_width = max_width.max(dx);
                    } else {
                        print_in_view(state, x + dx, y + dy, "()", block_colors)?;
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                }
                Fragment::BooleanInput(input_name) => {
                    if let Some(child_id) = &block.inputs[input_name].block_id {
                        let delta = draw_block(state, child_id, x + dx, y + dy)?;
                        dx += delta;
                        max_width = max_width.max(dx);
                    } else {
                        print_in_view(state, x + dx, y + dy, "<>", block_colors)?;
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                }
                Fragment::BlockInput(input_name) => {
                    if let Some(child_id) = &block.inputs[input_name].block_id {
                        // - 1 because we already have 1 cell available
                        let stack_height = draw_block(state, child_id, x + 1, y + dy)? - 1;
                        for y_range in 1..=stack_height {
                            print_in_view(state, x, y + dy + y_range, " ", block_colors)?;
                        }
                        dy += stack_height;
                    }
                    skip_padding = true;
                }
                Fragment::Dropdown(field) => {
                    // todo: dropdowns are not implemented
                    print_in_view(state, x + dx, y + dy, &format!("[{field}]"), block_colors)?;
                    dx += 2 + field.chars().count() as i32;
                }
                Fragment::Expander => {
                    print_in_view(
                        state,
                        x + dx,
                        y + dy,
                        // - 1 because we already have the left delimiter
                        &" ".repeat(max_width as usize - 1),
                        block_colors,
                    )?;
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
                    print_in_view(state, x + dx, y + dy, "▸", flag_color)?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Clockwise => {
                    print_in_view(state, x + dx, y + dy, "↻", block_colors)?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Anticlockwise => {
                    print_in_view(state, x + dx, y + dy, "↺", block_colors)?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::FieldText(field) => {
                    let Field { text, .. } = block.fields.get(field).unwrap();
                    print_in_view(state, x + dx, y + dy, text, block_colors)?;
                    dx += text.chars().count() as i32;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomColour(field) => {
                    let Field {
                        text: rgb_string, ..
                    } = block.fields.get(field).unwrap();
                    // #RRGGBB format
                    let (r, g, b) = parse_rgb(&rgb_string[1..]);
                    let custom_colours = Colors::new(Color::Reset, Color::Rgb { r, g, b });
                    print_in_view(state, x + dx, y + dy, "  ", custom_colours)?;
                    dx += 2;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomBlock(custom) => todo!("custom block"),
            }
        }
        if !skip_padding {
            print_in_view(state, x + dx, y + dy, delimeters.1, block_colors)?;
            dx += delimeters.1.chars().count() as i32;
            max_width = max_width.max(dx);
        }
        skip_padding = false;

        if let Shape::Stack = spec.shape {
            dy += 1;
            dx = 0;
        }
    }

    if spec.is_hat {
        print_in_view(state, x, y, &" ".repeat(max_width as usize), block_colors)?;
    }

    if let Some(next_id) = &block.next_id {
        dy += draw_block(state, next_id, x, y + dy)?;
    }
    queue!(stdout(), ResetColor)?;

    if let Shape::Stack = spec.shape {
        Ok(dy)
    } else {
        Ok(max_width)
    }
}
