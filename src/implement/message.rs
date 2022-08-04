use std::{path::Path, fs::File, io::Write};

use async_trait::async_trait;
use serenity::{prelude::Context, model::prelude::Message};

use crate::{
    data::TTSClientData,
    tts::{
        instance::TTSInstance,
        message::TTSMessage,
        gcp_tts::structs::{
            audio_config::AudioConfig, voice_selection_params::VoiceSelectionParams, synthesis_input::SynthesisInput, synthesize_request::SynthesizeRequest
        }
    },
};

#[async_trait]
impl TTSMessage for Message {
    async fn parse(&self, instance: &mut TTSInstance, _: &Context) -> String {
        let res = if let Some(before_message) = &instance.before_message {
            if before_message.author.id == self.author.id {
                self.content.clone()
            } else {
                format!("<speak>{} さんの発言<break time=\"200ms\"/>{}</speak>", self.author.name, self.content)
            }
        } else {
            self.content.clone()
        };

        instance.before_message = Some(self.clone());

        res
    }

    async fn synthesize(&self, instance: &mut TTSInstance, ctx: &Context) -> String {
        let text = self.parse(instance, ctx).await;

        let data_read = ctx.data.read().await;
        let storage = data_read.get::<TTSClientData>().expect("Cannot get TTSClientStorage").clone();
        let storage = storage.lock().await;

        let audio = storage.synthesize(SynthesizeRequest {
            input: SynthesisInput {
                text: None,
                ssml: Some(text)
            },
            voice: VoiceSelectionParams {
                languageCode: String::from("ja-JP"),
                name: String::from("ja-JP-Wavenet-B"),
                ssmlGender: String::from("neutral")
            },
            audioConfig: AudioConfig {
                audioEncoding: String::from("mp3"),
                speakingRate: 1.2f32,
                pitch: 1.0f32
            }
        }).await.unwrap();

        let uuid = uuid::Uuid::new_v4().to_string();

        let root = option_env!("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(root);
        let file_path = path.join("audio").join(format!("{}.mp3", uuid));

        let mut file = File::create(file_path.clone()).unwrap();
        file.write(&audio).unwrap();

        file_path.into_os_string().into_string().unwrap()
    }
}
