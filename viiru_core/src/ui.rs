use core::str;
use std::{
    collections::HashMap,
    io::{stdout, Write},
};

use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, SetCursorStyle, Show},
    execute, queue,
    style::{
        Attribute, Color, Colors, Print, ResetColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
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
    execute!(stdout(), EnterAlternateScreen, Hide, SetTitle("viiru"))?;
    enable_raw_mode()?;
    f()?;
    disable_raw_mode()?;
    execute!(
        stdout(),
        LeaveAlternateScreen,
        SetCursorStyle::DefaultUserShape,
        Show,
        SetTitle("")
    )?;
    Ok(())
}

pub struct DropPoint {
    pub shape: Shape,
    pub id: String,
    pub input: Option<String>,
}

#[derive(Default)]
pub struct Accumulators {
    pub block_positions: HashMap<(i32, i32), Vec<String>>,
    pub block_offsets: HashMap<String, (i32, i32)>,
    pub drop_points: HashMap<(i32, i32), DropPoint>,
    pub writable_points: HashMap<(i32, i32), String>,
}

impl Accumulators {
    pub fn add_grab_row(&mut self, block_id: &str, x: i32, y: i32, width: i32) {
        for i in 0..width {
            self.block_positions
                .entry((x + i, y))
                .or_default()
                .push(block_id.to_string())
        }
    }

    pub fn mark_block_offset(&mut self, block_id: &str, dx: i32, dy: i32) {
        self.block_offsets.insert(block_id.to_string(), (dx, dy));
    }

    pub fn add_drop_point(&mut self, x: i32, y: i32, shape: Shape, id: &str, input: Option<&str>) {
        self.drop_points.insert(
            (x, y),
            DropPoint {
                shape,
                id: id.to_string(),
                input: input.map(|s| s.to_string()),
            },
        );
    }

    pub fn add_writable_row(&mut self, block_id: &str, x: i32, y: i32, width: i32) {
        for i in 0..width {
            self.writable_points
                .insert((x + i, y), block_id.to_string());
        }
    }
}

fn alignment_point(top: bool, bottom: bool, is_start: bool, is_end: bool) -> &'static str {
    match (is_start, is_end) {
        (true, true) => match (top, bottom) {
            (true, true) => "| ",
            (true, false) => "' ",
            (false, true) => ". ",
            (false, false) => "  ",
        },
        (true, false) => {
            if top {
                "| "
            } else {
                ". "
            }
        }
        (false, true) => {
            if bottom {
                "| "
            } else {
                "' "
            }
        }
        (false, false) => "| ",
    }
}

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub char: Option<char>,
    pub underline: bool,
    pub colors: Option<Colors>,
}

// todo: smarter logic for better rendering performance
pub struct Screen {
    pub cells: Vec<Vec<Cell>>,
}

impl Screen {
    pub fn new(columns: u16, rows: u16) -> Self {
        let cells = vec![vec![Cell::default(); columns as usize]; rows as usize];
        Screen { cells }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }

    pub fn resize(&mut self, columns: u16, rows: u16) {
        for cell in &mut self.cells {
            cell.resize(columns as usize, Cell::default());
        }
        self.cells
            .resize(rows as usize, vec![Cell::default(); columns as usize]);
    }

    pub fn flush_contents(&self) -> ViiruResult<()> {
        queue!(stdout(), Hide, MoveTo(0, 0))?;
        let mut underline = false;
        let mut foreground = Color::Reset;
        let mut background = Color::Reset;
        for row in &self.cells {
            for cell in row {
                if cell.underline && !underline {
                    queue!(stdout(), SetAttribute(Attribute::Underlined))?;
                    underline = true;
                } else if !cell.underline && underline {
                    queue!(stdout(), SetAttribute(Attribute::NoUnderline))?;
                    underline = false;
                }
                if let Some(colors) = cell.colors {
                    if let Some(fg) = colors.foreground {
                        if fg != foreground {
                            queue!(stdout(), SetForegroundColor(fg))?;
                            foreground = fg;
                        }
                    }
                    if let Some(bg) = colors.background {
                        if bg != background {
                            queue!(stdout(), SetBackgroundColor(bg))?;
                            background = bg;
                        }
                    }
                } else if foreground != Color::Reset || background != Color::Reset {
                    queue!(stdout(), ResetColor)?;
                    foreground = Color::Reset;
                    background = Color::Reset;
                }
                let c = cell.char.unwrap_or(' ');
                queue!(stdout(), Print(c))?;
            }
            queue!(stdout(), MoveToNextLine(1))?;
        }
        stdout().flush()?;
        Ok(())
    }

    pub fn print(
        &mut self,
        x: i32,
        y: i32,
        s: &str,
        underline: bool,
        colors: Option<Colors>,
    ) -> Option<()> {
        for (dx, c) in s.chars().enumerate() {
            let cell = self.cells.get_mut(y as usize)?.get_mut(x as usize + dx)?;
            cell.char = Some(c);
            cell.underline = underline;
            cell.colors = colors;
        }
        Some(())
    }

    // todo: refactor to clean up
    #[allow(clippy::too_many_arguments)]
    pub fn print_in_view(
        &mut self,
        runtime: &Runtime,
        x: i32,
        y: i32,
        text: &str,
        colors: Colors,
        underline: bool,
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
            self.print(screen_x, screen_y, &chopped, underline, Some(colors));
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
            self.print(screen_x, screen_y, &chopped, underline, Some(colors));
        } else {
            let hidden_chars = runtime.viewport.x_min - screen_x;
            let chopped: String = text.chars().skip(hidden_chars as usize).collect();
            self.print(screen_x, screen_y, &chopped, underline, Some(colors));
        }
        Ok(())
    }

    /// returns either dx or dy depending on the block shape (expression or stack)
    pub fn draw_block(
        &mut self,
        runtime: &Runtime,
        block_id: &str,
        x: i32,
        y: i32,
        // We pass explicit accumulators instead of taking a &mut Runtime, because
        // there are a lot of simultaneous accesses that would have to be majorly
        // refactored otherwise. This is not too costly to do, but culling would
        // definitely be nice here.
        accumulators: &mut Accumulators,
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
            Shape::Circle => Ok(("(", ")")),
            Shape::Hexagon => Ok(("<", ">")),
            Shape::Stack => Err((block.parent_id.is_some(), block.next_id.is_some())),
        };

        let mut skip_padding = false;
        for (line_number, line) in spec.lines.iter().enumerate() {
            let is_start = line_number == 0;
            let is_end = line_number == spec.lines.len() - 1;
            match delimeters {
                Ok((d, _)) => {
                    self.print_in_view(runtime, x, y + dy, d, block_colors, true, fake)?;
                    let d_count = d.chars().count();
                    if BLOCKS[&block.opcode].is_shadow {
                        accumulators.add_writable_row(block_id, x, y + dy, d_count as i32);
                    } else {
                        accumulators.add_grab_row(block_id, x, y + dy, d_count as i32);
                    }
                    dx += d_count as i32;
                }
                Err((top, bottom)) => {
                    self.print_in_view(
                        runtime,
                        x,
                        y + dy,
                        alignment_point(top, bottom, is_start, is_end),
                        block_colors,
                        is_end,
                        fake,
                    )?;
                    accumulators.add_grab_row(block_id, x, y + dy, 2);
                    dx += 2;
                }
            }
            max_width = max_width.max(dx);
            for frag in line {
                match frag {
                    Fragment::Text(text) => {
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            text,
                            block_colors,
                            true,
                            fake,
                        )?;
                        let count = text.chars().count() as i32;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, count);
                        dx += count;
                        max_width = max_width.max(dx);
                    }
                    Fragment::StrumberInput(input_name, _default) => {
                        let Input {
                            shadow_id,
                            block_id: child_id,
                        } = &block.inputs[input_name];
                        let topmost = child_id.clone().or(shadow_id.clone());
                        accumulators.add_drop_point(
                            x + dx,
                            y + dy,
                            Shape::Circle,
                            block_id,
                            Some(input_name),
                        );
                        if let Some(input_id) = topmost {
                            let delta = self.draw_block(
                                runtime,
                                &input_id,
                                x + dx,
                                y + dy,
                                accumulators,
                                fake,
                            )?;
                            accumulators.mark_block_offset(&input_id, dx, dy);
                            dx += delta;
                            max_width = max_width.max(dx);
                        } else {
                            self.print_in_view(
                                runtime,
                                x + dx,
                                y + dy,
                                "()",
                                alt_colors,
                                true,
                                fake,
                            )?;
                            accumulators.add_grab_row(block_id, x + dx, y + dy, 2);
                            dx += 2;
                            max_width = max_width.max(dx);
                        }
                    }
                    Fragment::BooleanInput(input_name) => {
                        accumulators.add_drop_point(
                            x + dx,
                            y + dy,
                            Shape::Hexagon,
                            block_id,
                            Some(input_name),
                        );
                        if let Some(child_id) = &block.inputs[input_name].block_id {
                            let delta = self.draw_block(
                                runtime,
                                child_id,
                                x + dx,
                                y + dy,
                                accumulators,
                                fake,
                            )?;
                            accumulators.mark_block_offset(child_id, dx, dy);
                            dx += delta;
                            max_width = max_width.max(dx);
                        } else {
                            self.print_in_view(
                                runtime,
                                x + dx,
                                y + dy,
                                "<>",
                                alt_colors,
                                true,
                                fake,
                            )?;
                            accumulators.add_grab_row(block_id, x + dx, y + dy, 2);
                            dx += 2;
                            max_width = max_width.max(dx);
                        }
                    }
                    Fragment::BlockInput(input_name) => {
                        accumulators.add_drop_point(
                            x + 2,
                            y + dy,
                            Shape::Stack,
                            block_id,
                            Some(input_name),
                        );
                        if let Some(child_id) = &block.inputs[input_name].block_id {
                            // - 1 because we already have 1 cell available
                            let stack_height = self.draw_block(
                                runtime,
                                child_id,
                                x + 2,
                                y + dy,
                                accumulators,
                                fake,
                            )? - 1;
                            accumulators.mark_block_offset(child_id, 2, dy);
                            for y_range in 1..=stack_height {
                                self.print_in_view(
                                    runtime,
                                    x,
                                    y + dy + y_range,
                                    "| ",
                                    block_colors,
                                    false,
                                    fake,
                                )?;
                                accumulators.add_grab_row(block_id, x, y + dy + y_range, 2);
                            }
                            dy += stack_height;
                        }
                        skip_padding = true;
                    }
                    Fragment::AlignmentPoint(substack_name) => {
                        let connected = block.inputs[substack_name].block_id.is_some();
                        let s = alignment_point(false, connected, true, true);
                        self.print_in_view(runtime, x + dx, y + dy, s, block_colors, true, fake)?;
                        accumulators.add_grab_row(block_id, x, y + dy, 2);
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                    Fragment::Dropdown(field, _static_options) => {
                        // todo: dropdowns are not implemented
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            &format!("[{field}]"),
                            block_colors,
                            true,
                            fake,
                        )?;
                        let count = 2 + field.chars().count() as i32;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, count);
                        // TODO: add interactors
                        dx += count;
                    }
                    Fragment::Expander => {
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            // - 1 because we already have the left delimiter
                            &" ".repeat(max_width as usize - 1),
                            block_colors,
                            true,
                            fake,
                        )?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, max_width - 1);
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
                        self.print_in_view(runtime, x + dx, y + dy, "▸", flag_color, true, fake)?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, 1);
                        dx += 1;
                        max_width = max_width.max(dx);
                    }
                    Fragment::Clockwise => {
                        self.print_in_view(runtime, x + dx, y + dy, "↻", block_colors, true, fake)?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, 1);
                        dx += 1;
                        max_width = max_width.max(dx);
                    }
                    Fragment::Anticlockwise => {
                        self.print_in_view(runtime, x + dx, y + dy, "↺", block_colors, true, fake)?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, 1);
                        dx += 1;
                        max_width = max_width.max(dx);
                    }
                    Fragment::FieldText(field) => {
                        let Field { value, .. } = block.fields.get(field).unwrap();
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            value,
                            block_colors,
                            true,
                            fake,
                        )?;
                        let count = value.chars().count() as i32;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, count);
                        dx += count;
                        max_width = max_width.max(dx);
                    }
                    Fragment::WritableFieldText(field) => {
                        let Field { value, .. } = block.fields.get(field).unwrap();
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            value,
                            block_colors,
                            true,
                            fake,
                        )?;
                        let count = value.chars().count() as i32;
                        accumulators.add_writable_row(block_id, x + dx, y + dy, count);
                        // TODO: add interactors
                        dx += count;
                        max_width = max_width.max(dx);
                    }
                    Fragment::CustomColour(field) => {
                        let Field {
                            value: rgb_string, ..
                        } = block.fields.get(field).unwrap();
                        // #RRGGBB format
                        let (r, g, b) = parse_rgb(&rgb_string[1..]);
                        let custom_colours = Colors::new(Color::Reset, Color::Rgb { r, g, b });
                        self.print_in_view(
                            runtime,
                            x + dx,
                            y + dy,
                            "  ",
                            custom_colours,
                            true,
                            fake,
                        )?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, 2);
                        // TODO: add interactors
                        dx += 2;
                        max_width = max_width.max(dx);
                    }
                }
            }
            if !skip_padding {
                match delimeters {
                    Ok((_, d)) => {
                        self.print_in_view(runtime, x + dx, y + dy, d, block_colors, true, fake)?;
                        let d_count = d.chars().count() as i32;
                        if BLOCKS[&block.opcode].is_shadow {
                            accumulators.add_writable_row(block_id, x + dx, y + dy, d_count);
                        } else {
                            accumulators.add_grab_row(block_id, x + dx, y + dy, d_count);
                        }
                        dx += d_count;
                    }
                    Err(_) => {
                        self.print_in_view(runtime, x + dx, y + dy, " ", block_colors, true, fake)?;
                        accumulators.add_grab_row(block_id, x + dx, y + dy, 1);
                        dx += 1;
                    }
                }
                max_width = max_width.max(dx)
            }
            skip_padding = false;

            if let Shape::Stack = spec.shape {
                dy += 1;
                dx = 0;
            }
        }

        if spec.is_hat {
            self.print_in_view(
                runtime,
                x,
                y,
                &" ".repeat(max_width as usize),
                block_colors,
                false,
                fake,
            )?;
            accumulators.add_grab_row(block_id, x, y, max_width);
        }

        if let Shape::Stack = spec.shape {
            accumulators.add_drop_point(x, y + dy, Shape::Stack, block_id, None);
            if let Some(next_id) = &block.next_id {
                accumulators.mark_block_offset(next_id, 0, dy);
                dy += self.draw_block(runtime, next_id, x, y + dy, accumulators, fake)?;
            }
        }

        if let Shape::Stack = spec.shape {
            Ok(dy)
        } else {
            Ok(max_width)
        }
    }

    pub fn draw_viewport_border(&mut self, runtime: &Runtime) -> ViiruResult<()> {
        let vp = &runtime.viewport;
        let border_color = Color::Rgb {
            r: 0x7f,
            g: 0x3e,
            b: 0xcf,
        };
        let colors = Some(Colors::new(border_color, Color::Reset));
        self.print(
            vp.x_min,
            vp.y_min - 1,
            &"-".repeat(vp.width() as usize),
            false,
            colors,
        );
        self.print(
            vp.x_min,
            vp.y_max,
            &"-".repeat(vp.width() as usize),
            false,
            colors,
        );
        for y in vp.y_min..vp.y_max + 1 {
            self.print(vp.x_min - 1, y, "|", false, colors);
            self.print(vp.x_max, y, "|", false, colors);
        }
        self.print(vp.x_min - 1, vp.y_min - 1, ".", false, colors);
        self.print(vp.x_max, vp.y_min - 1, ".", false, colors);
        self.print(vp.x_min - 1, vp.y_max, "'", false, colors);
        self.print(vp.x_max, vp.y_max, "'", false, colors);
        Ok(())
    }

    pub fn draw_cursor_lines(&mut self, runtime: &Runtime) -> ViiruResult<()> {
        let vp = &runtime.viewport;
        let screen_x = runtime.cursor_x - runtime.scroll_x;
        let screen_y = runtime.cursor_y - runtime.scroll_y;
        if vp.y_min <= screen_y && screen_y < vp.y_max {
            self.print(
                vp.x_min,
                screen_y,
                &"-".repeat((screen_x - vp.x_min) as usize),
                false,
                None,
            );
            self.print(
                screen_x + 1,
                screen_y,
                &"-".repeat((vp.x_max - screen_x - 1) as usize),
                false,
                None,
            );
        }
        if vp.x_min <= screen_x && screen_x < vp.x_max {
            for y in vp.y_min..vp.y_max {
                if y != screen_y {
                    self.print(screen_x, y, "|", false, None);
                }
            }
        }
        Ok(())
    }

    pub fn draw_cursor(&mut self, runtime: &Runtime) -> ViiruResult<()> {
        execute!(
            stdout(),
            MoveTo(
                (runtime.cursor_x - runtime.scroll_x) as u16,
                (runtime.cursor_y - runtime.scroll_y) as u16
            ),
            Show,
            SetCursorStyle::SteadyBlock,
        )?;
        Ok(())
    }

    pub fn draw_marker_dots(&mut self, runtime: &Runtime) -> ViiruResult<()> {
        let x_spacing = 10;
        let y_spacing = 5;
        let x_first = runtime.scroll_x / x_spacing * x_spacing;
        let y_first = runtime.scroll_y / y_spacing * y_spacing;

        let dot_color = Colors::new(Color::DarkGrey, Color::Reset);

        for dx in 0..1 + runtime.viewport.width() / x_spacing {
            for dy in 0..2 + runtime.viewport.height() / y_spacing {
                self.print_in_view(
                    runtime,
                    x_first + dx * x_spacing,
                    y_first + dy * y_spacing,
                    ".",
                    dot_color,
                    false,
                    false,
                )?;
            }
        }

        Ok(())
    }

    pub fn draw_toolbox(
        &mut self,
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
            let mut unused = Accumulators::default();
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
                self.print_in_view(
                    runtime,
                    offset_x - 1 - i_str.len() as i32,
                    offset_y + dy,
                    &i_str,
                    colors,
                    false,
                    true,
                )?;
            }
            let delta = self.draw_block(
                runtime,
                id,
                offset_x,
                offset_y + dy,
                &mut unused,
                !recompute,
            )?;
            if let Shape::Stack = shape {
                dy += delta + 1;
            } else {
                dy += 2;
            }
            if dy >= runtime.viewport.height() {
                runtime.toolbox_visible_max = i;
                break;
            }
        }
        Ok(())
    }

    pub fn refresh_screen(&mut self, runtime: &mut Runtime) -> ViiruResult<()> {
        self.draw_viewport_border(runtime)?;
        self.draw_marker_dots(runtime)?;
        self.draw_cursor_lines(runtime)?;
        let mut accumulators = Accumulators::default();
        for top_id in &runtime.top_level {
            // draw the cursor block last so it's always on top
            let is_cursor = if let Some(cursor_id) = &runtime.cursor_block {
                cursor_id == top_id
            } else {
                false
            };
            if !is_cursor {
                self.draw_block(
                    runtime,
                    top_id,
                    runtime.blocks[top_id].x,
                    runtime.blocks[top_id].y,
                    &mut accumulators,
                    false,
                )?;
            }
        }
        if let Some(cursor_id) = &runtime.cursor_block {
            self.draw_block(
                runtime,
                cursor_id,
                runtime.blocks[cursor_id].x,
                runtime.blocks[cursor_id].y,
                &mut accumulators,
                false,
            )?;
        }
        runtime.process_accumulators(accumulators);
        let position = format!(
            "{},{} {}",
            runtime.cursor_x, runtime.cursor_y, runtime.last_command
        );
        self.print(
            runtime.viewport.x_max - position.len() as i32,
            runtime.viewport.y_max + 1,
            &position,
            false,
            None,
        );
        if let State::Command = runtime.state {
            let command_prefix = match runtime.last_command {
                'o' => "file path: ",
                'w' => "output path: ",
                _ => "",
            };
            self.print(
                runtime.viewport.x_min,
                runtime.viewport.y_max + 1,
                command_prefix,
                false,
                None,
            );
            self.print(
                runtime.viewport.x_min + command_prefix.len() as i32,
                runtime.viewport.y_max + 1,
                &runtime.command_buffer,
                false,
                None,
            );
        } else {
            self.print(
                runtime.viewport.x_min,
                runtime.viewport.y_max + 1,
                &runtime.status_message,
                false,
                None,
            );
        }
        let vox = runtime.viewport_offset_x;
        let voy = runtime.viewport_offset_y;
        self.draw_toolbox(runtime, vox, voy, false)?;
        Ok(())
    }
}
