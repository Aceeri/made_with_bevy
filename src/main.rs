use bevy::{asset::AssetMetaCheck, input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_vello::{VelloPlugin, integrations::svg::load_svg_from_str, prelude::*};

const BIRD_SVG: &str = include_str!("../assets/bird-0.svg");
/// What color to replace
const BIRD_SOURCE_FILL: &str = "#ececec";
/// outline/clear color
const BG_FILL: &str = "#232326";
const BIRD_COLORS: [&str; 3] = ["#ececec", "#b2b2b2", "#787878"];
const BIRD_NAMES: [&str; 3] = ["Birb 0 (front)", "Birb 1 (middle)", "Birb 2 (back)"];

const BIRD_SCALE: f32 = 6.0;
const OUTLINE_THICKNESS: f32 = 0.10;
const FADE_DURATION: f32 = 0.4;
const SLIDE_DURATION: f32 = 0.7;
const BIRD_ANCHOR: Vec2 = Vec2::new(-110.0, 25.0);
const SLIDE_OFFSET: Vec2 = Vec2::new(110.0, -25.0);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Bird(u8);

#[derive(Reflect, Clone)]
struct BirdKeyFrame {
    start: Transform,
    end: Transform,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct BirdAnimation {
    birds: [BirdKeyFrame; 3],
}

// Handwritten keyframes (not const because Quat::from_euler isn't const).
fn bird_keyframes() -> [BirdKeyFrame; 3] {
    [
        BirdKeyFrame {
            start: Transform {
                translation: Vec3::new(-110.0, 25.0, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(6.0),
            },
            end: Transform {
                translation: Vec3::new(-110.0, 25.0, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(6.0),
            },
        },
        BirdKeyFrame {
            start: Transform {
                translation: Vec3::new(-110.0, 25.0, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(5.0),
            },
            end: Transform {
                translation: Vec3::new(-65.9, 35.6, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.5),
                scale: Vec3::splat(5.0),
            },
        },
        BirdKeyFrame {
            start: Transform {
                translation: Vec3::new(-110.5, 21.8, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(4.25),
            },
            end: Transform {
                translation: Vec3::new(-30.5, 21.8, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -1.0),
                scale: Vec3::splat(4.25),
            },
        },
    ]
}

impl Default for BirdAnimation {
    fn default() -> Self {
        Self {
            birds: bird_keyframes(),
        }
    }
}

#[derive(Resource)]
struct Splash {
    elapsed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(VelloPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .register_type::<Bird>()
        .register_type::<BirdKeyFrame>()
        .register_type::<BirdAnimation>()
        .init_resource::<BirdAnimation>()
        .insert_resource(ClearColor(Color::srgb_u8(0x23, 0x23, 0x26)))
        .add_systems(Startup, setup)
        .add_systems(Update, (replay_splash, animate_splash).chain())
        .run();
}

fn setup(mut commands: Commands, mut svgs: ResMut<Assets<VelloSvg>>) {
    commands.spawn((Name::new("Camera"), Camera2d, VelloView));

    let bodies: [Handle<VelloSvg>; 3] =
        std::array::from_fn(|i| svgs.add(bake_bird(BIRD_COLORS[i])));

    // has to be another vello svg to interleave
    let cutoff_svg = svgs.add(
        load_svg_from_str(&format!(
            r#"<svg><rect width="1" height="1" fill="{BG_FILL}"/></svg>"#
        ))
        .expect("cutoff svg failed to parse"),
    );

    commands.spawn((
        Name::new("Tail/wing cutoff"),
        VelloSvg2d(cutoff_svg),
        VelloSvgAnchor::Center,
        Transform {
            translation: Vec3::new(-130.0, 35.0, 2.0),
            rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -0.3),
            scale: Vec3::new(150.0, 300.0, 1.0),
        }
    ));

    for index in 0u8..3 {
        let z: f32 = match index {
            0 => 3.0,
            1 => 1.0,
            _ => 0.0,
        };

        commands.spawn((
            Name::new(BIRD_NAMES[index as usize]),
            VelloSvg2d(bodies[index as usize].clone()),
            VelloSvgAnchor::Center,
            Transform::from_xyz(BIRD_ANCHOR.x, BIRD_ANCHOR.y, z)
                .with_scale(Vec3::splat(BIRD_SCALE)),
            Bird(index),
        ));
    }

    commands.insert_resource(Splash { elapsed: 0.0 });
}

fn bake_bird(fill_hex: &str) -> VelloSvg {
    let recolored = BIRD_SVG.replace(BIRD_SOURCE_FILL, fill_hex);
    load_svg_from_str(&recolored).expect("bird svg failed to parse")
}

fn replay_splash(keys: Res<ButtonInput<KeyCode>>, mut splash: ResMut<Splash>) {
    if keys.just_pressed(KeyCode::KeyR) {
        splash.elapsed = 0.0;
    }
}

fn animate_splash(
    time: Res<Time>,
    mut splash: ResMut<Splash>,
    anim: Res<BirdAnimation>,
    mut birds: Query<(&Bird, &mut Transform)>,
) {
    splash.elapsed += time.delta_secs();
    let slide_t = (splash.elapsed - FADE_DURATION) / SLIDE_DURATION;
    let eased = EaseFunction::CubicOut.sample_clamped(slide_t);

    for (bird, mut transform) in &mut birds {
        let kf = &anim.birds[bird.0 as usize];
        transform.translation = kf.start.translation.lerp(kf.end.translation, eased);
        transform.rotation = kf.start.rotation.slerp(kf.end.rotation, eased);
        transform.scale = kf.start.scale.lerp(kf.end.scale, eased);
    }
}
