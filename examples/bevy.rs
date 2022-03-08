use bevy::prelude::*;
use bevy_spicy_aseprite::{
    AsepriteAnimation, AsepriteAnimationState, AsepriteBundle, AsepritePlugin,
};

mod sprites {
    use bevy_spicy_aseprite::aseprite;
    // https://meitdev.itch.io/crow
    aseprite!(pub Crow, "assets/crow.aseprite");
    // https://shubibubi.itch.io/cozy-people
    aseprite!(pub Player, "assets/player.ase");
}

#[derive(Debug, Component, Clone, Copy)]
pub struct PlayerTag;

#[derive(Debug, Component, Clone, Copy)]
pub struct CrowTag;

enum Action {
    MoveTo(Entity, Vec2),
    SetAnim(Entity, AsepriteAnimation),
}

fn process_sequencer(sequencer: Sequencer) {
    sequencer.for_each_active(|key, action| {
        match action {
            Action::MoveTo(entity, dest) => todo!(),
            Action::SetAnim(entity, anim) => todo!(),
        }
    });
}

type Sequencer = sequencer::Sequencer<Action>;

fn main() {
    App::new()
        .init_resource::<Sequencer>()
        .add_plugins(DefaultPlugins)
        .add_plugin(AsepritePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let font = asset_server.load("Share-Regular.ttf");

    let text_style = TextStyle {
        font,
        font_size: 30.,
        ..Default::default()
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(AsepriteBundle {
            aseprite: sprites::Crow::sprite(),
            animation: AsepriteAnimation::from(sprites::Crow::tags::FLAP_WINGS),
            transform: Transform {
                scale: Vec3::splat(4.),
                translation: Vec3::new(0., 150., 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CrowTag);
    commands
        .spawn_bundle(AsepriteBundle {
            aseprite: sprites::Player::sprite(),
            animation: AsepriteAnimation::from(sprites::Player::tags::LEFT_WALK),
            transform: Transform {
                scale: Vec3::splat(4.),
                translation: Vec3::new(0., -200., 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerTag);
}
