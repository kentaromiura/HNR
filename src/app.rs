use crate::fetcher;
use crate::fetcher::StoryItem;
use ratatui::widgets::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
#[derive(PartialEq)]
pub enum Action {
    Quit,
    None,
    NextStory,
    PrevStory,
    ReadStory,
    Stories(Vec<StoryItem>),
    OpenNext(String),
}

pub struct App {
    pub listen_to_key: Arc<Mutex<bool>>,
    pub should_quit: bool,
    pub show_loading: bool,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub stories: Vec<StoryItem>,
    pub stories_state: ListState,
    pub fetcher: Arc<fetcher::HackerNews>,
}

impl App {
    pub fn set_should_quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_stories(&mut self, stories: Vec<StoryItem>) {
        self.stories = stories.clone();
        self.stories_state = ListState::default();
        self.stories_state.select_first();
        self.show_loading = false;
    }

    pub fn update(self: &mut Self, msg: Action) -> Action {
        match msg {
            Action::Stories(stories) => self.load_stories(stories),
            Action::Quit => self.set_should_quit(),
            Action::NextStory => self.stories_state.select_next(),
            Action::PrevStory => self.stories_state.select_previous(),
            Action::ReadStory => {
                if let Some(story_index) = self.stories_state.selected() {
                    if let Some(story) = self.stories.get(story_index) {
                        if let Some(url) = story.url.clone() {
                            return Action::OpenNext(url);
                        }
                    }
                }
            }
            _ => {}
        };
        Action::None
    }

    pub fn handle_events(
        self: &Self,
        tx: mpsc::UnboundedSender<Action>,
    ) -> tokio::task::JoinHandle<()> {
        let tick_rate = std::time::Duration::from_millis(100);

        let copy = Arc::clone(&self.listen_to_key);
        tokio::spawn(async move {
            loop {
                #[allow(unused_assignments)]
                let mut should_listen = false;
                {
                    let listen_lock = copy.lock().unwrap();
                    should_listen = *listen_lock;
                }

                let action = if crossterm::event::poll(tick_rate).unwrap() {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                        if should_listen && key.kind == crossterm::event::KeyEventKind::Press {
                            match key.code {
                                crossterm::event::KeyCode::Char('q') => Action::Quit,
                                crossterm::event::KeyCode::Up => Action::PrevStory,
                                crossterm::event::KeyCode::Down => Action::NextStory,
                                crossterm::event::KeyCode::Enter => Action::ReadStory,
                                _ => Action::None,
                            }
                        } else {
                            Action::None
                        }
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                };
                if let Err(_) = tx.send(action) {
                    break;
                }
            }
        })
    }
}
