use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};
use ratatui_image::{Resize, StatefulImage, picker::Picker, protocol::StatefulProtocol};
use snapd_rs_artie::SnapConfinement;

use crate::app::App;
use crate::layout::{format_size, truncate_text};

pub(crate) fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Snap Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::uniform(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(snap) = app.selected_snap() else {
        let placeholder = Paragraph::new("Select a snap to see details")
            .style(Style::default().fg(Color::DarkGray).italic());
        frame.render_widget(placeholder, inner);
        return;
    };

    // Layout: header + body
    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(inner);

    // Header: name + version + publisher
    let mut header_lines = vec![
        Line::from(vec![Span::styled(
            snap.title.clone().unwrap_or_else(|| snap.name.clone()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Name:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(snap.name.clone(), Style::default().fg(Color::Cyan)),
        ]),
    ];

    if let Some(v) = &snap.version {
        header_lines.push(Line::from(vec![
            Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
            Span::raw(v.clone()),
        ]));
    }

    if let Some(pub_) = &snap.publisher {
        header_lines.push(Line::from(vec![
            Span::styled("By:      ", Style::default().fg(Color::DarkGray)),
            Span::raw(pub_.clone()),
        ]));
    }

    if let Some(size) = snap.size {
        header_lines.push(Line::from(vec![
            Span::styled("Size:    ", Style::default().fg(Color::DarkGray)),
            Span::raw(format_size(size)),
        ]));
    }

    frame.render_widget(Paragraph::new(header_lines), detail_layout[0]);

    // Body: summary + description + metadata
    let mut body_lines: Vec<Line> = vec![];

    if let Some(summary) = &snap.summary {
        body_lines.push(Line::from(Span::styled(
            summary.clone(),
            Style::default().fg(Color::Yellow),
        )));
        body_lines.push(Line::raw(""));
    }

    if let Some(desc) = &snap.description {
        for line in desc.lines().take(20) {
            body_lines.push(Line::raw(line.to_string()));
        }
        body_lines.push(Line::raw(""));
    }

    // Metadata badges
    let mut badges: Vec<Span> = vec![];

    if snap.installed {
        badges.push(Span::styled(
            " installed ",
            Style::default().bg(Color::Green).fg(Color::Black),
        ));
        badges.push(Span::raw(" "));
    }

    if let Some(conf) = &snap.confinement {
        let (label, color) = match conf {
            SnapConfinement::Strict => ("strict", Color::Blue),
            SnapConfinement::Classic => ("classic", Color::Magenta),
            SnapConfinement::Devmode => ("devmode", Color::Red),
            _ => ("unknown", Color::DarkGray),
        };
        badges.push(Span::styled(
            format!(" {label} "),
            Style::default().bg(color).fg(Color::White),
        ));
        badges.push(Span::raw(" "));
    }

    if let Some(channel) = &snap.channel {
        badges.push(Span::styled(
            format!(" {channel} "),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ));
    }

    if !badges.is_empty() {
        body_lines.push(Line::from(badges));
    }

    let body = Paragraph::new(Text::from(body_lines)).wrap(Wrap { trim: false });
    frame.render_widget(body, detail_layout[1]);

    if let Some(icon_url) = &snap.icon_url
        && let Some(picker) = &app.icon_picker
        && let Some(Some(image)) = app.icon_cache.get(icon_url)
    {
        render_snap_icon(frame, picker, image, inner);
    }
}

pub(crate) fn render_snap_icon(
    frame: &mut Frame,
    picker: &Picker,
    image: &image::DynamicImage,
    area: Rect,
) {
    let icon_width = area.width.min(16);
    let icon_height = area.height.min(8);
    if icon_width < 8 || icon_height < 4 {
        return;
    }

    let icon_area = Rect {
        x: area.x + area.width.saturating_sub(icon_width),
        y: area.y,
        width: icon_width,
        height: icon_height,
    };
    let mut protocol = picker.new_resize_protocol(image.clone());
    let image_widget = StatefulImage::<StatefulProtocol>::default().resize(Resize::Fit(None));
    frame.render_stateful_widget(image_widget, icon_area, &mut protocol);
}

pub(crate) fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let style = Style::default().bg(Color::DarkGray).fg(Color::White);

    let help = if app.search_focused {
        " Enter  confirm   Esc  cancel "
    } else {
        return render_status_bar_with_help(
            frame,
            app,
            area,
            &format!(
                " {}   Esc  back   ?/F1  help   q  quit ",
                crate::symbols::nav_hint()
            ),
            style,
        );
    };
    render_status_bar_with_help(frame, app, area, help, style);
}

fn render_status_bar_with_help(frame: &mut Frame, app: &App, area: Rect, help: &str, style: Style) {
    let indicator = if app.loading {
        Span::styled(
            format!(" Loading{} ", crate::symbols::ellipsis()),
            Style::default().bg(Color::Yellow).fg(Color::Black),
        )
    } else if let Some(err) = &app.error {
        Span::styled(
            format!(" {} {err} ", crate::symbols::error_sym()),
            Style::default().bg(Color::Red).fg(Color::White),
        )
    } else if app.active_change_id.is_some() {
        let working = format!("Working{}", crate::symbols::ellipsis());
        let message = app
            .status_message
            .as_deref()
            .or_else(|| {
                app.active_change
                    .as_ref()
                    .map(|change| change.summary.as_str())
            })
            .unwrap_or(working.as_str());
        Span::styled(
            format!(
                " {} {message} ",
                if crate::symbols::is_unicode() {
                    "⟳"
                } else {
                    "*"
                }
            ),
            Style::default().bg(Color::Blue).fg(Color::White),
        )
    } else if let Some(msg) = &app.status_message {
        Span::styled(
            format!(" {} {msg} ", crate::symbols::ok_sym()),
            Style::default().bg(Color::Green).fg(Color::Black),
        )
    } else {
        Span::raw("")
    };

    let indicator_width = indicator.content.chars().count() as u16;
    let help_width = area.width.saturating_sub(indicator_width) as usize;
    let help_truncated = truncate_text(help, help_width);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(help_truncated, style),
        indicator,
    ]));
    frame.render_widget(bar, area);
}
