use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetColors},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use crate::{blocks::BLOCKS, result::ViiruResult, spec::Shape, state::State};

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
            crate::spec::Fragment::StrumberInput(input, default) => {
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
            crate::spec::Fragment::BooleanInput(input) => todo!("boi"),
            crate::spec::Fragment::BlockInput(_) => todo!("bi"),
            crate::spec::Fragment::Dropdown(_) => todo!("dd"),
            crate::spec::Fragment::Expander => todo!("e"),
            crate::spec::Fragment::Flag => todo!("f"),
            crate::spec::Fragment::Clockwise => todo!("cw"),
            crate::spec::Fragment::Anticlockwise => todo!("acw"),
            crate::spec::Fragment::FieldText(field) => {
                let text = block.fields.get(field).unwrap();
                queue!(stdout(), Print(text))?;
            }
            crate::spec::Fragment::VariableName(_) => todo!("vn"),
            crate::spec::Fragment::ListName(_) => todo!("ln"),
            crate::spec::Fragment::CustomColour(_) => todo!("cc"),
            crate::spec::Fragment::CustomBlock(_) => todo!("cb"),
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
