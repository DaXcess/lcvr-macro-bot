use std::sync::LazyLock;

use poise::serenity_prelude::RoleId;

pub static DISCORD_TOKEN: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN environment variable")
});

pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_URL").expect("missing DATABASE_URL environment variable")
});

pub static MACRO_ROLE_ID: LazyLock<RoleId> = LazyLock::new(|| {
    let id = std::env::var("MACRO_ROLE_ID").expect("missing MACRO_ROLE_ID environment variable");

    RoleId::new(id.parse().expect("invalid MACRO_ROLE_ID specified"))
});
