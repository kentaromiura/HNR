mod app;
mod cli;
mod ui;
use app::*;
use color_eyre::eyre::Result;
mod fetcher;
use ratatui::{prelude::*, widgets::*};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

async fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut app = App {
        listen_to_key: Arc::new(Mutex::new(true)),
        stories_state: ListState::default(),
        show_loading: true,
        should_quit: false,
        action_tx,
        stories: Vec::new(),
        fetcher: Arc::new(fetcher::HackerNews {
            client: reqwest::Client::new(),
        }),
    };

    let task = app.handle_events(app.action_tx.clone());

    let tx_stories = app.action_tx.clone();
    let fetchy = app.fetcher.clone();
    let fetch_stories = tokio::spawn(async move {
        if let Ok(stories) = fetchy.get_stories_from_to(0, 50).await {
            let msg = Action::Stories(stories);
            let _ = tx_stories.send(msg);
        }
    });
    let mut story_max = 50;
    loop {
        t.draw(|f| {
            ui::main(f, &app);
        })?;

        if let Some(action) = action_rx.recv().await {
            match app.update(action) {
                Action::FetchMore => {
                    story_max += 5;
                    let fetchy = app.fetcher.clone();
                    let tx_stories = app.action_tx.clone();
                    let _fetch_stories = tokio::spawn(async move {
                        if let Ok(stories) = fetchy.get_stories_from_to(0, story_max).await {
                            let msg = Action::Stories(stories);
                            let _ = tx_stories.send(msg);
                        }
                    });
                }
                Action::OpenNext(url) => {
                    {
                        let mut listen_lock = app.listen_to_key.lock().unwrap();
                        *listen_lock = false;
                    }
                    cli::leave_alt()?;

                    open_in_browser(&url);

                    // I think a flush for stdin here would help fixing weirdness with lynx...

                    cli::enter_alt()?;
                    {
                        let mut listen_lock = app.listen_to_key.lock().unwrap();
                        *listen_lock = true;
                    }

                    t.clear()?;
                    t.draw(|f| {
                        ui::clear(f);
                        ui::main(f, &app);
                    })?;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    task.abort();
    fetch_stories.abort();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = cli::initialize_panic_handler();
    cli::startup()?;
    run().await?;
    cli::shutdown()?;
    Ok(())
}

fn open_in_browser(url: &str) {
    // lynx seems to have some trouble when returning back
    // let _ = Command::new("lynx")
    //     .arg(url)
    //     .spawn()
    //     .unwrap()
    //     .wait_with_output();
    let _ = Command::new("carbonyl-0.0.3/carbonyl")
        .arg(url)
        .spawn()
        .unwrap()
        .wait_with_output();
}
