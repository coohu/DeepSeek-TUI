use crate::localization::MessageId;
use crate::palette;
use crate::tui::app::App;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

pub const PROVIDER_OPTIONS: &[(char, &str, &str, &str)] = &[
    ('1', "deepseek", "DeepSeek", "(platform.deepseek.com)"),
    (
        '2',
        "shengsuanyun",
        "胜算云 (ShengSuanYun)",
        "(router.shengsuanyun.com)",
    ),
];

pub fn lines(app: &App) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = vec![
        Line::from(Span::styled(
            app.tr(MessageId::OnboardApiKeyProviderTitle).to_string(),
            Style::default()
                .fg(palette::DEEPSEEK_SKY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            app.tr(MessageId::OnboardApiKeyProviderBlurb).to_string(),
            Style::default().fg(palette::TEXT_MUTED),
        )),
        Line::from(""),
    ];

    let selected_id = app.onboarding_api_provider.as_str();

    for (hotkey, provider_id, name, hint) in PROVIDER_OPTIONS {
        let is_selected = selected_id == *provider_id;
        let bullet = if is_selected { "●" } else { "○" };
        let bullet_color = if is_selected {
            palette::DEEPSEEK_BLUE
        } else {
            palette::TEXT_MUTED
        };
        out.push(Line::from(vec![
            Span::styled(format!("  {bullet}  "), Style::default().fg(bullet_color)),
            Span::styled(
                format!("[{hotkey}] "),
                Style::default()
                    .fg(palette::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(name.to_string(), Style::default().fg(palette::TEXT_PRIMARY)),
            Span::styled(
                format!("  {hint}"),
                Style::default().fg(palette::TEXT_MUTED),
            ),
        ]));
    }

    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        app.tr(MessageId::OnboardApiKeyProviderFooter).to_string(),
        Style::default().fg(palette::TEXT_MUTED),
    )));

    out
}
