mod commands;
mod database;
mod env;
mod error;
mod params;

use std::sync::Arc;

use anyhow::{Error, Result};
use database::Database;
use log::{error, info};
use poise::serenity_prelude::{self as serenity, FullEvent, ShardManager};

type Context<'a> = poise::ApplicationContext<'a, Database, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    if std::env::var("RUST_LOG").is_err() {
        #[cfg(debug_assertions)]
        std::env::set_var("RUST_LOG", "lcvr_macros");

        #[cfg(not(debug_assertions))]
        std::env::set_var("RUST_LOG", "lcvr_macros=info");
    }

    env_logger::init();
    dotenvy::dotenv().ok();

    let database = Database::connect(&*env::DATABASE_URL)?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::add_macro(),
                commands::delete(),
                commands::macros(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tokio::spawn(shutdown_handler(framework.shard_manager().clone()));

                Ok(database)
            })
        })
        .build();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(&*env::DISCORD_TOKEN, intents)
        .framework(framework)
        .await?;

    if let Err(why) = client.start().await {
        error!("Error during runtime: {why}");
    }

    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Database, Error>,
    database: &Database,
) -> Result<(), Error> {
    if let FullEvent::Message {
        new_message: message,
    } = event
    {
        let Some(ref member) = message.member else {
            return Ok(());
        };

        if !member.roles.contains(&env::MACRO_ROLE_ID) {
            return Ok(());
        }

        if !message.content.starts_with(".") {
            return Ok(());
        }

        if let Err(why) = commands::execute_macro(ctx, message, database).await {
            error!("Error on macro invocation: {why}");
        }
    }

    Ok(())
}

async fn shutdown_handler(shard_manager: Arc<ShardManager>) {
    _ = tokio::signal::ctrl_c().await;

    info!("Received interrupt signal, shutting down...");

    shard_manager.shutdown_all().await;
}
