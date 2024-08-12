use crate::{app::App, fetcher::StoryItem};
use ratatui::{prelude::*, widgets::*};

fn render_title(area: Rect, buf: &mut Buffer) {
    Block::new()
        .borders(Borders::NONE)
        .title("Hacker news".bold())
        .title_alignment(Alignment::Center)
        .on_light_blue()
        .fg(Color::LightYellow)
        .render(area, buf);
}

fn render_loading(area: Rect, buf: &mut Buffer) {
    "Loading...".bold().on_blue().render(area, buf);
}
fn render_nothing(area: Rect, buf: &mut Buffer) {
    "          ".on_black().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Block::new()
        .borders(Borders::NONE)
        .title(" Q to quit, Enter to read story")
        .on_light_blue()
        .fg(Color::LightYellow)
        .render(area, buf);
}

fn render_stories(
    area: Rect,
    buf: &mut Buffer,
    stories: &Vec<StoryItem>,
    list_state: &mut ListState,
) {
    let mut items = Vec::new();
    for story in stories {
        items.push(story.title.as_str());
    }
    let widget = List::new(items)
        .block(Block::bordered().title("Stories"))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);
    if list_state.selected() == None {
        list_state.select_first()
    }
    StatefulWidget::render(widget, area, buf, list_state);
}

pub struct Main {
    story_state: ListState,
    show_loading: bool,
    stories: Vec<StoryItem>,
}

impl Widget for &mut Main {
    fn render(self: Self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);
        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);
        if self.show_loading {
            render_loading(tabs_area, buf)
        } else {
            render_nothing(tabs_area, buf);
            render_stories(inner_area, buf, &self.stories, &mut self.story_state);
        }
        render_footer(footer_area, buf);
        render_title(title_area, buf);
    }
}

pub fn clear(f: &mut Frame) {
    let area = f.size();
    f.render_widget(Clear, area);
}

pub fn main(f: &mut Frame, app: &App) {
    let mut main = Main {
        show_loading: app.show_loading,
        stories: app.stories.clone(),
        story_state: app.stories_state.clone(),
    };

    f.render_widget(&mut main, f.size());
}
