use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use crate::{
    block::{Block, Kind},
    blocks::BLOCKS,
    result::ViiruResult,
    spec::{Shape, Spec},
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

pub fn print_size(columns: u16, rows: u16) -> ViiruResult {
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(format!("hi there, {columns}x{rows} terminal!"))
    )?;
    Ok(())
}

pub fn draw_block(block: &Block, x: u16, y: u16) -> ViiruResult<u16> {
    queue!(stdout(), MoveTo(x, y))?;
    let spec = &BLOCKS[&block.opcode];
    let mut dx = 0u16;
    let input_index = 0;
    match &block.kind {
        Kind::Expression(expr) => {
            let start = match spec.shape {
                Shape::Circle => '(',
                Shape::Hexagon => '<',
                Shape::Stack => todo!(),
            };
            queue!(stdout(), Print(start))?;
            dx += 1;
            for frag in &spec.head {
                match frag {
                    crate::spec::Fragment::Text(t) => {
                        queue!(stdout(), Print(t))?;
                        dx += t.chars().count() as u16;
                    }
                    crate::spec::Fragment::StrumberInput(_field, _default) => {
                        match &block.input_ids[input_index] {
                            Some(_input_id) => todo!(),
                            None => todo!(),
                        }
                    }
                    crate::spec::Fragment::BooleanInput(_) => todo!(),
                    crate::spec::Fragment::BlockInput(_) => todo!(),
                    crate::spec::Fragment::Dropdown(_) => todo!(),
                    crate::spec::Fragment::Expander => todo!(),
                    crate::spec::Fragment::Flag => todo!(),
                    crate::spec::Fragment::Clockwise => todo!(),
                    crate::spec::Fragment::Anticlockwise => todo!(),
                    crate::spec::Fragment::CustomBlock(_) => todo!(),
                    crate::spec::Fragment::VariableName(_) => todo!(),
                    crate::spec::Fragment::ListName(_) => todo!(),
                    crate::spec::Fragment::CustomColour(_) => todo!(),
                    crate::spec::Fragment::FieldText(_) => todo!(),
                }
            }
            let end = match spec.shape {
                Shape::Circle => ')',
                Shape::Hexagon => '>',
                Shape::Stack => todo!(),
            };
            queue!(stdout(), Print(end))?;
        }
        Kind::Stack(_stack) => todo!(),
    }

    stdout().flush()?;
    Ok(dx)
}
