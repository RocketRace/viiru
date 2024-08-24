use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, Show},
    execute, queue,
    style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetColors, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use crate::{
    block::Throption::Given,
    blocks::BLOCKS,
    result::ViiruResult,
    spec::{Fragment, Shape},
    state::State,
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

/// returns either dx or dy depending on the block shape (expression or stack)
pub fn draw_block(state: &State, block_id: &str, x: u16, y: u16) -> ViiruResult<u16> {
    queue!(stdout(), MoveTo(x, y))?;
    let block = state.blocks.get(block_id).unwrap();
    let spec = &BLOCKS[&block.opcode];
    let mut max_width = 0u16;
    let mut dx = 0u16;
    let mut dy = 0u16;
    let mut input_index = 0;

    let color_command = SetColors(Colors::new(
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
    ));

    if spec.is_hat {
        queue!(stdout(), MoveDown(1))?;
        dy += 1;
    }

    queue!(stdout(), color_command)?;

    let delimeters = match spec.shape {
        Shape::Circle => ('(', ')'),
        Shape::Hexagon => ('<', '>'),
        Shape::Stack => (' ', ' '),
    };

    let mut skip_padding = false;
    for line in &spec.lines {
        queue!(stdout(), Print(delimeters.0))?;
        dx += 1;
        max_width = max_width.max(dx);
        for frag in line {
            match frag {
                Fragment::Text(t) => {
                    queue!(stdout(), Print(t))?;
                    dx += t.chars().count() as u16;
                    max_width = max_width.max(dx);
                }
                Fragment::StrumberInput(input_name, default) => {
                    let (shadow, cover) = &block.input_ids[input_index];
                    let topmost = cover.clone().or(shadow.clone());
                    if let Some(input_id) = topmost {
                        let delta = draw_block(state, &input_id, x + dx, y + dy)?;
                        dx += delta;
                        max_width = max_width.max(dx);
                        queue!(stdout(), color_command)?;
                    } else {
                        queue!(stdout(), Print("()"))?;
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                    input_index += 1;
                }
                Fragment::BooleanInput(input_name) => {
                    if let (_, Some(child_id)) = &block.input_ids[input_index] {
                        let delta = draw_block(state, child_id, x + dx, y + dy)?;
                        dx += delta;
                        max_width = max_width.max(dx);
                        queue!(stdout(), color_command)?;
                    } else {
                        queue!(stdout(), Print("<>"))?;
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                    input_index += 1;
                }
                Fragment::BlockInput(input_name) => {
                    if let (_, Some(child_id)) = &block.input_ids[input_index] {
                        dy += draw_block(state, child_id, x + 1, y + dy)? - 1;
                    } else {
                        skip_padding = true;
                    }
                    input_index += 1;
                }
                Fragment::Dropdown(field) => todo!("dropdown"),
                Fragment::Expander => {
                    queue!(stdout(), Print(" ".repeat(max_width as usize - 1)))?;
                    // - 1 because we already have the left delimiter
                    skip_padding = true;
                    dx = max_width;
                }
                Fragment::Flag => {
                    queue!(
                        stdout(),
                        SetForegroundColor(Color::Rgb {
                            r: 0x6d,
                            g: 0xbf,
                            b: 0x63
                        }),
                        Print("▸"),
                        color_command
                    )?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Clockwise => {
                    queue!(stdout(), Print("↻"))?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::Anticlockwise => {
                    queue!(stdout(), Print("↺"))?;
                    dx += 1;
                    max_width = max_width.max(dx);
                }
                Fragment::FieldText(field) => {
                    let text = block.fields.get(field).unwrap();
                    queue!(stdout(), Print(text))?;
                    dx += text.chars().count() as u16;
                    max_width = max_width.max(dx);
                }
                Fragment::VariableName(field) => {
                    let id = block.fields.get(field).unwrap();
                    let name = state.variables.get(id).unwrap();
                    queue!(stdout(), Print(name))?;
                    dx += name.chars().count() as u16;
                    max_width = max_width.max(dx);
                }
                Fragment::ListName(field) => {
                    let id = block.fields.get(field).unwrap();
                    let name = state.lists.get(id).unwrap();
                    queue!(stdout(), Print(name))?;
                    dx += name.chars().count() as u16;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomColour(field) => {
                    let rgb_string = block.fields.get(field).unwrap();
                    let (r, g, b) = parse_rgb(rgb_string);
                    queue!(
                        stdout(),
                        SetBackgroundColor(Color::Rgb { r, g, b }),
                        Print("  "),
                        color_command
                    )?;
                    dx += 2;
                    max_width = max_width.max(dx);
                }
                Fragment::CustomBlock(custom) => todo!("cb"),
            }
        }
        if !skip_padding {
            queue!(stdout(), Print(delimeters.1))?;
            dx += 1;
            max_width = max_width.max(dx);
        }
        skip_padding = false;

        if let Shape::Stack = spec.shape {
            dy += 1;
            dx = 0;
            queue!(stdout(), ResetColor, MoveTo(x, y + dy), color_command)?;
        }
    }

    if spec.is_hat {
        queue!(
            stdout(),
            MoveTo(x, y),
            Print(" ".repeat(max_width as usize))
        )?;
    }

    if let Given(next_id) = &block.next_id {
        dy += draw_block(state, next_id, x, y + dy)?;
    }

    queue!(stdout(), ResetColor)?;

    stdout().flush()?;
    if let Shape::Stack = spec.shape {
        Ok(dy)
    } else {
        Ok(max_width)
    }
}
