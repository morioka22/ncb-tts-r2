use serenity::{client::{EventHandler, Context}, async_trait, model::{gateway::Ready, interactions::{Interaction, application_command::ApplicationCommandInteraction, InteractionApplicationCommandCallbackDataFlags}, id::{GuildId, UserId}, channel::Message}, framework::standard::macros::group};
use crate::{data::TTSData, tts::instance::TTSInstance};

#[group]
struct Test;

pub struct Handler;

async fn setup_command(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), Box<dyn std::error::Error>> {
    if let None = command.guild_id {
        command.create_interaction_response(&ctx.http, |f| {
            f.interaction_response_data(|d| {
                d.content("このコマンドはサーバーでのみ使用可能です．").flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        }).await?;
        return Ok(());
    }

    let guild = command.guild_id.unwrap().to_guild_cached(&ctx.cache).await;
    if let None = guild {
        command.create_interaction_response(&ctx.http, |f| {
            f.interaction_response_data(|d| {
                d.content("ギルドキャッシュを取得できませんでした．").flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        }).await?;
        return Ok(());
    }
    let guild = guild.unwrap();

    let channel_id = guild
        .voice_states
        .get(&UserId(command.user.id.0))
        .and_then(|state| state.channel_id);

    if let None = channel_id {
        command.create_interaction_response(&ctx.http, |f| {
            f.interaction_response_data(|d| {
                d.content("ボイスチャンネルに参加してから実行してください．").flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        }).await?;
        return Ok(());
    }

    let channel_id = channel_id.unwrap();

    let manager = songbird::get(ctx).await.expect("Cannot get songbird client.").clone();

    let storage_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<TTSData>().expect("Cannot get TTSStorage").clone()
    };

    {
        let mut storage = storage_lock.write().await;
        if storage.contains_key(&guild.id) {
            command.create_interaction_response(&ctx.http, |f| {
                f.interaction_response_data(|d| {
                    d.content("すでにセットアップしています．").flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            }).await?;
            return Ok(());
        }

        storage.insert(guild.id, TTSInstance {
            before_message: None,
            guild: guild.id,
            text_channel: command.channel_id,
            voice_channel: channel_id
        });
    }

    let _handler = manager.join(guild.id.0, channel_id.0).await;

    command.create_interaction_response(&ctx.http, |f| {
        f.interaction_response_data(|d| {
            d.content("セットアップ完了").flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
    }).await?;

    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let name = &*command.data.name;
            match name {
                "setup" => setup_command(&ctx, &command).await.unwrap(),
                _ => {}
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        let guild_id = message.guild(&ctx.cache).await.unwrap().id;

        let storage_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<TTSData>().expect("Cannot get TTSStorage").clone()
        };

        {
            let mut storage = storage_lock.write().await;
            if !storage.contains_key(&guild_id) {
                return;
            }

            let instance = storage.get_mut(&guild_id).unwrap();

            instance.read(message, &ctx).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(696782998799909024);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("setup")
                    .description("Setup tts")
            })
        }).await;
        println!("{:?}", commands);
    }
}
