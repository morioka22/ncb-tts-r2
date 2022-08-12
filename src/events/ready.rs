use serenity::{prelude::Context, model::prelude::{Ready, application_command::ApplicationCommand}};

pub async fn ready(ctx: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);

    let _ = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
        commands.create_application_command(|command| {
            command.name("stop")
                .description("Stop tts")
        });
        commands.create_application_command(|command| {
            command.name("setup")
                .description("Setup tts")
        });
        commands.create_application_command(|command| {
            command.name("config")
                .description("Config")
        })
    }).await;
}