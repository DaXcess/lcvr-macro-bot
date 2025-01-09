use crate::Context;

use anyhow::Result;
use log::{error, info};
use poise::{serenity_prelude::CreateEmbed, CreateReply};

/// Delete a macro
#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The name of the macro to remove"] name: String,
) -> Result<()> {
    match ctx.data.delete_macro(&name) {
        Err(why) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .description(format!("Failed to delete the `.{name}` macro"))
                            .color(0xFC1F28),
                    )
                    .ephemeral(true),
            )
            .await?;

            error!("Failed to delete macro: {why}");
        }
        Ok(false) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .description(format!("No macro with the name `.{name}` exists"))
                            .color(0xFC1F28),
                    )
                    .ephemeral(true),
            )
            .await?;
        }
        Ok(true) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .description(format!("Successfully deleted the `.{name}` macro"))
                            .color(0x3BD65D),
                    )
                    .ephemeral(true),
            )
            .await?;

            info!("Deleted macro .{name}");
        }
    };

    Ok(())
}
