use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetColors, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use crate::{blocks::BLOCKS, result::ViiruResult, spec::Shape, state::State, util::parse_rgb};

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

pub fn print_size(columns: u16, rows: u16) -> ViiruResult {
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(format!("hi there, {columns}x{rows} terminal!"))
    )?;
    Ok(())
}

/// returns either dx or dy depending on the block shape (expression or stack)
pub fn draw_block(state: &State, block_id: &str, x: u16, y: u16) -> ViiruResult<u16> {
    queue!(stdout(), MoveTo(x, y))?;
    let block = state.blocks.get(block_id).unwrap();
    let spec = &BLOCKS[&block.opcode];
    let mut dx = 0u16;
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
    queue!(stdout(), color_command)?;

    let delimeters = match spec.shape {
        Shape::Circle => Some(('(', ')')),
        Shape::Hexagon => Some(('<', '>')),
        Shape::Stack => None,
    };
    if let Some((start, _)) = delimeters {
        queue!(stdout(), Print(start))?;
        dx += 1;
    }

    for frag in &spec.head {
        match frag {
            crate::spec::Fragment::Text(t) => {
                queue!(stdout(), Print(t))?;
                dx += t.chars().count() as u16;
            }
            crate::spec::Fragment::StrumberInput(input_name, default) => {
                let (shadow, cover) = &block.input_ids[input_index];
                let topmost = cover.clone().or(shadow.clone());
                if let Some(input_id) = topmost {
                    dx += draw_block(state, &input_id, x + dx, y)?;
                    queue!(stdout(), color_command)?;
                } else {
                    queue!(stdout(), Print("()"))?;
                    dx += 2;
                }
                input_index += 1;
            }
            crate::spec::Fragment::BooleanInput(input_name) => {
                if let (_, Some(child_id)) = &block.input_ids[input_index] {
                    dx += draw_block(state, child_id, x + dx, y)?;
                    queue!(stdout(), color_command)?;
                } else {
                    queue!(stdout(), Print("<>"))?;
                    dx += 2;
                }
                input_index += 1;
            }
            crate::spec::Fragment::BlockInput(input_name) => {
                unreachable!("could have avoided this with a different design")
            }
            crate::spec::Fragment::Dropdown(field) => todo!("dd"),
            crate::spec::Fragment::Expander => {
                unreachable!("could have avoided this with a different design")
            }
            crate::spec::Fragment::Flag => {
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
            }
            crate::spec::Fragment::Clockwise => {
                queue!(stdout(), Print("↻"))?;
                dx += 1;
            }
            crate::spec::Fragment::Anticlockwise => {
                queue!(stdout(), Print("↺"))?;
                dx += 1;
            }
            crate::spec::Fragment::FieldText(field) => {
                let text = block.fields.get(field).unwrap();
                queue!(stdout(), Print(text))?;
                dx += text.chars().count() as u16;
            }
            crate::spec::Fragment::VariableName(field) => {
                let id = block.fields.get(field).unwrap();
                let name = state.variables.get(id).unwrap();
                queue!(stdout(), Print(name))?;
                dx += name.chars().count() as u16;
            }
            crate::spec::Fragment::ListName(field) => {
                let id = block.fields.get(field).unwrap();
                let name = state.lists.get(id).unwrap();
                queue!(stdout(), Print(name))?;
                dx += name.chars().count() as u16;
            }
            crate::spec::Fragment::CustomColour(field) => {
                let rgb_string = block.fields.get(field).unwrap();
                let (r, g, b) = parse_rgb(rgb_string);
                queue!(
                    stdout(),
                    SetBackgroundColor(Color::Rgb { r, g, b }),
                    Print(" "),
                    color_command
                )?;
                dx += 1;
            }
            crate::spec::Fragment::CustomBlock(custom) => todo!("cb"),
        }
    }
    if let Some((_, end)) = delimeters {
        queue!(stdout(), Print(end))?;
        dx += 1;
    }

    queue!(stdout(), ResetColor)?;

    stdout().flush()?;
    Ok(dx)
}
