use anyhow::Result;
use log::{error, info};
use poise::serenity_prelude::{
    Context, CreateAllowedMentions, CreateAttachment, CreateMessage, Message,
};

use crate::{
    database::{
        models::{Attachment, Macro},
        Database,
    },
    params::ParameterizedString,
};

pub async fn execute_macro(ctx: &Context, message: &Message, database: &Database) -> Result<()> {
    let command = &message.content.split(" ").next().unwrap()[1..];

    let Some((r#macro, attachments)) = database.get_macro(command)? else {
        return Ok(()); // Ignore if macro doesn't exist
    };

    // Attempt to retrieve up-to-date content and attachments from source
    let success = match ctx
        .http
        .get_message(
            r#macro.channel_id.parse().unwrap(),
            r#macro.message_id.parse().unwrap(),
        )
        .await
    {
        Ok(src_message) => execute_macro_with_message(ctx, src_message, message).await?,
        Err(_) => execute_macro_with_database(ctx, message, r#macro, attachments).await?,
    };

    if success {
        info!(
            "Executed macro .{command} (by {})",
            message.author.display_name()
        );
    }

    Ok(())
}

async fn execute_macro_with_message(
    ctx: &Context,
    src_message: Message,
    message: &Message,
) -> Result<bool> {
    // Build macro content, replacing params with our files
    let param_str = ParameterizedString::new(&src_message.content)?;

    if src_message.attachments.len() < param_str.parameters() {
        error!("Message has too many parameters for the attachments it has, cannot send macro!");

        message
            .channel_id
            .send_message(
                ctx,
                CreateMessage::new().content("Macro invocation failed: source message parameters have been changed and are no longer valid!"),
            )
            .await?;

        return Ok(false);
    }

    let macro_content = param_str.to_string(
        src_message.attachments[..param_str.parameters()]
            .iter()
            .map(|att| &att.url)
            .collect(),
    )?;

    let mut builder = CreateMessage::new().content(&macro_content);

    // Any additional files will be attached directly to the message
    for attachment in &src_message.attachments[param_str.parameters()..] {
        builder = builder.add_file(CreateAttachment::url(ctx, &attachment.url).await?);
    }

    if let Some(ref reference) = message.referenced_message {
        builder = builder
            .reference_message(&**reference)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(true))
    }

    message.channel_id.send_message(ctx, builder).await?;
    message.delete(ctx).await?;

    Ok(true)
}

async fn execute_macro_with_database(
    ctx: &Context,
    message: &Message,
    r#macro: Macro,
    attachments: Vec<Attachment>,
) -> Result<bool> {
    // Build macro content, replacing params with our files
    let param_str = ParameterizedString::new(&r#macro.content)?;
    let mut macro_content = param_str.to_string(
        attachments[..param_str.parameters()]
            .iter()
            .map(|att| &att.link)
            .collect(),
    )?;

    // Any additional files will be attached at the end of the message, to fix issues with expiring urls
    if !attachments[param_str.parameters()..].is_empty() {
        macro_content += "\n";

        for attachment in &attachments[param_str.parameters()..] {
            macro_content += &format!("\n{}", attachment.link);
        }
    }

    let mut builder = CreateMessage::new().content(&macro_content);

    if let Some(ref reference) = message.referenced_message {
        builder = builder
            .reference_message(&**reference)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(true))
    }

    message.channel_id.send_message(ctx, builder).await?;
    message.delete(ctx).await?;

    Ok(true)
}
