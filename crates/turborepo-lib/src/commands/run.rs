use std::sync::Arc;

use tracing::error;
use turborepo_signals::{listeners::get_signal, SignalHandler};
use turborepo_telemetry::events::command::CommandEventBuilder;
use turborepo_ui::sender::UISender;

use crate::{commands::CommandBase, run, run::builder::RunBuilder};

pub async fn run(base: CommandBase, telemetry: CommandEventBuilder) -> Result<i32, run::Error> {
    let signal = get_signal()?;
    let handler = SignalHandler::new(signal);

    let run_builder = RunBuilder::new(base)?;

    let run_fut = async {
        let (analytics_sender, analytics_handle) = run_builder.start_analytics();
        let run = Arc::new(
            run_builder
                .with_analytics_sender(analytics_sender)
                .build(&handler, telemetry)
                .await?,
        );

        let (sender, handle) = run.start_ui()?.unzip();

        let result = run.run(sender.clone(), false).await;

        if let Some(analytics_handle) = analytics_handle {
            analytics_handle.close_with_timeout().await;
        }

        // We only stop if it's the TUI, for the web UI we don't need to stop
        if let Some(UISender::Tui(sender)) = sender {
            sender.stop().await;
        }

        if let Some(handle) = handle {
            if let Err(e) = handle.await.expect("render thread panicked") {
                error!("error encountered rendering tui: {e}");
            }
        }

        result
    };

    let handler_fut = handler.done();
    tokio::select! {
        biased;
        // If we get a handler exit at the same time as a run finishes we choose that
        // future to display that we're respecting user input
        _ = handler_fut => {
            // We caught a signal, which already notified the subscribers
            Ok(1)
        }
        result = run_fut => {
            // Run finished so close the signal handler
            handler.close().await;
            result
        },
    }
}
