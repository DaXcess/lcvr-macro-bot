use anyhow::Result;
use log::info;
use poise::serenity_prelude::{
    Context, CreateAllowedMentions, CreateAttachment, CreateMessage, Message,
};

use crate::{database::Database, params::ParameterizedString};

pub async fn execute_macro(
    ctx: &Context,
    message: &Message,
    content: String,
    database: &Database,
) -> Result<()> {
    let command = &content.split(" ").next().unwrap()[1..];

    let Some((r#macro, attachments)) = database.get_macro(command)? else {
        return Ok(()); // Ignore if macro doesn't exist
    };

    // Build macro content, replacing params with our files
    let param_str = ParameterizedString::new(&r#macro.content)?;
    let macro_content = param_str.to_string(
        attachments[..param_str.parameters()]
            .iter()
            .map(|att| &att.link)
            .collect(),
    )?;

    let mut builder = CreateMessage::new().content(&macro_content);

    // Any additional files will be attached directly to the message
    for attachment in &attachments[param_str.parameters()..] {
        builder = builder.add_file(CreateAttachment::url(ctx, &attachment.link).await?);
    }

    if let Some(ref reference) = message.referenced_message {
        builder = builder
            .reference_message(&**reference)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(true))
    }

    message.channel_id.send_message(ctx, builder).await?;
    message.delete(ctx).await?;

    info!(
        "Executed macro .{command} (by {})",
        message.author.display_name()
    );

    Ok(())
}
