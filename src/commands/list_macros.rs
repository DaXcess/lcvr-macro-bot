use std::time::Duration;

use crate::{database::models::Macro, Context};

use anyhow::Result;
use poise::{
    serenity_prelude::{
        ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
        CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    CreateReply,
};

/// List all macros currently available
#[poise::command(slash_command)]
pub async fn macros(ctx: Context<'_>) -> Result<()> {
    let macros = ctx.data.get_macros()?;
    let pages = macros.chunks(25).len();

    let ctx_id = ctx.id();
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let reply = {
        let components = CreateActionRow::Buttons(vec![
            CreateButton::new(&prev_button_id).label('◀'),
            CreateButton::new(&next_button_id).label('▶'),
        ]);

        CreateReply::default()
            .embed(create_macro_embed(&macros, 0))
            .components(vec![components])
    };

    ctx.send(reply).await?;

    let mut page = 0;
    while let Some(press) = ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(Duration::from_secs(3600 * 24))
        .await
    {
        if press.user.id != ctx.author().id {
            press
                .create_response(
                    ctx.serenity_context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(
                                "Only the person who executed the command may use these buttons",
                            )
                            .ephemeral(true),
                    ),
                )
                .await?;

            continue;
        }

        if press.data.custom_id == next_button_id {
            page += 1;
            if page >= pages {
                page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            page = page.checked_sub(1).unwrap_or(pages - 1);
        } else {
            continue;
        }

        press
            .create_response(
                ctx.serenity_context,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(create_macro_embed(&macros, page)),
                ),
            )
            .await?;
    }

    Ok(())
}

fn create_macro_embed(macros: &[Macro], page: usize) -> CreateEmbed {
    let chunks = macros.chunks(25);

    let mut embed = CreateEmbed::new()
        .title("List of macros")
        .footer(CreateEmbedFooter::new(format!(
            "Page {}/{}",
            page + 1,
            chunks.len()
        )))
        .color(0x0773D6);

    for r#macro in macros
        .chunks(25)
        .nth(page)
        .expect("Page exceeds max page count")
    {
        embed = embed.field(&r#macro.name, &r#macro.description, false);
    }

    embed
}
