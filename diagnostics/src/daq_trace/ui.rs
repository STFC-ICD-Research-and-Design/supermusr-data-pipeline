use super::{app::App, data::DigitiserData};
use ratatui::{
    Frame,
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

/// Draws the ui based on the current app state.
pub(crate) fn ui(frame: &mut Frame, app: &mut App) {
    // Split terminal into different-sized chunks.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(frame.area());

    // Draw all widgets.
    draw_table(frame, app, chunks[0]);
    draw_help(frame, chunks[1]);
}

/// Draws a help box containing key binding information in a given chunk.
fn draw_help(frame: &mut Frame, chunk: Rect) {
    let help = Paragraph::new(Text::styled(
        "<UP>: Previous row | <DOWN>: Next row | <LEFT>: Previous Channel | <RIGHT>: Next Channel | <q>: Quit",
        Style::default().add_modifier(Modifier::DIM),
    ))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title("Help"),
    );

    frame.render_widget(help, chunk)
}

/// Draws the main table in a given chunk.
fn draw_table(frame: &mut Frame, app: &mut App, chunk: Rect) {
    let widths: Vec<Constraint> = DigitiserData::width_percentages()
        .into_iter()
        .map(Constraint::Percentage)
        .collect();
    let table = Table::new(
        // Turn table data into rows with given formatting.
        app.table_body.iter().map(|item| {
            // Calculate height based on line count.
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            // Apply formatting to each cell.
            let cells = item.iter().map(|c| Cell::from(c.clone()));
            Row::new(cells).height(height as u16).bottom_margin(1)
        }),
        &widths,
    )
    // Add table headers with given formatting.
    .header(
        Row::new(
            app.table_headers
                .iter()
                .map(|h| Cell::from(h.clone().replace(' ', "\n"))),
        )
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .height(3)
        .bottom_margin(2),
    )
    // Modify table style.
    .column_spacing(1)
    .row_highlight_style(
        Style::default()
            .fg(Color::LightMagenta)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));

    frame.render_stateful_widget(table, chunk, &mut app.table_state);
}
