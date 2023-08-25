use ratatui::{prelude::{Backend, Layout, Direction, Constraint, Alignment, Rect}, Frame, widgets::{Paragraph, Block, Borders, Table, Row, Cell}, text::Text, style::{Style, Modifier, Color}};
use super::SharedData;

type TableBody<'a> = Vec<Row<'a>>;

pub fn ui<B: Backend>(frame: &mut Frame<B>, shared_data: SharedData) {
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
    draw_table(frame, &generate_table_rows(shared_data), chunks[1]);
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

fn draw_table<B: Backend>(frame: &mut Frame<B>, table_body: &TableBody, chunk: Rect) {
    let table = Table::new(table_body.clone())
        .header(Row::new(vec![
        //--------------------------------------------------+-------+
        //  Heading                                         | Index |
        //--------------------------------------------------+-------+
            Cell::from("Digitiser\nID"          ),  //      |   1   |
            Cell::from("#Msgs\nReceived"        ),  //      |   2   |
            Cell::from("First\nMsg\nTimestamp"  ),  //      |   3   |
            Cell::from("Last\nMsg\nTimestamp"   ),  //      |   4   |
            Cell::from("Last\nMsg\nFrame"       ),  //      |   5   |
            Cell::from("#Present\nChannels"     ),  //      |   6   |
            Cell::from("#Channels\nChanged?"    ),  //      |   7   |
            Cell::from("#Samples\nin\nFirst"    ),  //      |   8   |
            Cell::from("#Samples\nIdentical?"   ),  //      |   9   |
            Cell::from("#Samples\nChanged?"     ),  //      |   10  |
        //--------------------------------------------------+-------+
        ])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
            )
            .height(3)
        )
        .block(Block::default().borders(Borders::ALL))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10), 
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .column_spacing(1)
        .highlight_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
        )
        .highlight_symbol("> ");

    frame.render_widget(table, chunk);
}

fn generate_table_rows(shared_data: SharedData) -> TableBody<'static> {
    let mut body: TableBody = Vec::new();
    let logged_data = shared_data.lock().unwrap();
    for (digitiser_id, digitiser_data) in logged_data.iter() {
        body.push(
            Row::new(vec![
                // 1. Digitiser ID
                digitiser_id.to_string(),
                // 2. Number of messages received
                format!("{}", digitiser_data.num_msg_received).to_string(),
                // 3. First message timestamp
                match digitiser_data.first_msg_timestamp {
                    None => "N/A".to_string(),
                    Some(d) => format!("{:?}", d).to_string(),
                },
                // 4. Last message timestamp
                match digitiser_data.last_msg_timestamp {
                    None => "N/A".to_string(),
                    Some(d) => format!("{:?}", d).to_string(),
                },
                // 5. Last message frame
                match digitiser_data.last_msg_frame {
                    None => "N/A".to_string(),
                    Some(d) => format!("{}", d).to_string(),
                },
                // 6. Number of channels present
                format!("{}", digitiser_data.num_channels_present).to_string(),
                // 7. Has the number of channels changed?
                format!("{}", 
                    match digitiser_data.has_num_channels_changed {
                        true => "Yes",
                        false => "No"
                    }
                ).to_string(),
                // 8. Number of samples in the first channel
                match digitiser_data.num_samples_in_first_channel {
                    None => "N/A".to_string(),
                    Some(d) => format!("{}", d).to_string(),
                },
                // 9. Is the number of samples identical?
                format!("{}",
                    match digitiser_data.is_num_samples_identical {
                        true => "Yes",
                        false => "No"
                    }
                ).to_string(),
                // 10. Has the number of samples changed?
                format!("{}",
                    match digitiser_data.has_num_samples_changed {
                        true => "Yes",
                        false => "No"
                    }
                ).to_string()
            ])
        )
    }
    body
}
