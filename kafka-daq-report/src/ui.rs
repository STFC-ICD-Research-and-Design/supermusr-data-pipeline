use ratatui::{prelude::{Backend, Layout, Direction, Constraint, Alignment, Rect}, Frame, widgets::{Paragraph, Block, Borders, TableState, Table, Row}, text::Text, style::{Style, Modifier, Color}};
use crate::app::TableBody;

pub fn ui<B: Backend>(frame: &mut Frame<B>, table_body: &TableBody, table_state: &mut TableState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3)
            ]
            .as_ref()
        )
        .split(frame.size()
    );

    draw_title(frame, chunks[0]);
    draw_table(frame, &table_body, table_state, chunks[1]);
    draw_help(frame, chunks[2]);
}

fn draw_title<B: Backend>(frame: &mut Frame<B>, chunk: Rect) {
    let title = Paragraph::new(
        Text::styled(
            "Kafka DAQ Report",
            Style::default()
        )
    )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()
        )
    );

    frame.render_widget(title, chunk);
}

fn draw_help<B: Backend>(frame: &mut Frame<B>, chunk: Rect) {
    let help = Paragraph::new(
        Text::styled(
            "<UP>: Previous row | <DOWN>: Next row | <q>: Quit",
            Style::default()
        )
    )
    .alignment(Alignment::Center)
    .block(
        Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title("Help")
    );

    frame.render_widget(help, chunk)
}

fn draw_table<B: Backend>(frame: &mut Frame<B>, table_body: &TableBody, table_state: &mut TableState, chunk: Rect) {
    let table = Table::new(table_body.clone())
        .header(Row::new(vec!["Key", "Value"])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
            )
        )
        .block(Block::default().borders(Borders::ALL))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
        .column_spacing(3)
        .highlight_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, chunk, table_state);
}
