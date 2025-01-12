use crate::{params::ParameterizedString, Context};
use anyhow::{anyhow, Result};
use log::info;
use poise::{
    serenity_prelude::{self as serenity, CreateEmbed},
    CreateReply,
};

#[derive(Debug, poise::Modal)]
#[name = "Create new macro"]
struct NewMacroModal {
    #[name = "Macro name"]
    name: String,

    #[name = "Macro description"]
    description: String,
}

/// Create a new macro
#[poise::command(context_menu_command = "Create macro")]
pub async fn add_macro(
    ctx: Context<'_>,
    #[description = "Message source to base the macro on"] msg: serenity::Message,
) -> Result<()> {
    use poise::Modal as _;

    let content = msg.content;
    let attachments = msg.attachments;

    // Check content for formatting errors
    let pstring = match ParameterizedString::new(&content).and_then(|pstring| {
        if pstring.parameters() > attachments.len() {
            Err(anyhow!("Macro contains more parameters than attachments"))
        } else {
            Ok(pstring)
        }
    }) {
        Ok(pstring) => pstring,
        Err(why) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("Failed to parse macro content")
                            .description(format!("Your macro contains formatting errors:\n`{why}`"))
                            .color(0xFC1F28),
                    )
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };

    // Prevent bandwidth abuse by blocking "raw" attachments larger than 10 MiB
    if attachments[pstring.parameters()..]
        .iter()
        .any(|attachment| attachment.size > 1024 * 1024 * 10)
    {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Attachment size exceeds limit")
                        .description("Attachments that are not embedded as a URL may not exceed 10 MiB in file size")
                        .color(0xFC1F28),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let Some(NewMacroModal { name, description }) = NewMacroModal::execute(ctx).await? else {
        return Ok(());
    };

    // Check name for errors
    if name.len() > 32
        || name
            .chars()
            .any(|c| !c.is_ascii_lowercase() && c != '-' && c != '_')
    {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Macro name is invalid")
                        .description(
                            "Macro name must be lowercase, only contain characters `a-z`, `-` or `_`, and must not exceed 32 characters in length."
                        )
                        .color(0xFC1F28),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let attachments = attachments
        .into_iter()
        .map(|att| att.url)
        .collect::<Vec<_>>();

    ctx.data.create_macro(
        &name,
        &description,
        &msg.channel_id.to_string(),
        &msg.id.to_string(),
        &content,
        &attachments,
    )?;

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .description(format!("Successfully created the `.{name}` macro"))
                    .color(0x3BD65D),
            )
            .ephemeral(true),
    )
    .await?;

    info!("Macro .{name} has been created");

    Ok(())
}
