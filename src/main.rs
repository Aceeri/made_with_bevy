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
const FADE_DURATION: f32 = 0.4;
const SLIDE_DURATION: f32 = 0.7;
const BIRD_ANCHOR: Vec2 = Vec2::new(-110.0, 25.0);

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
        .insert_resource(ClearColor(Color::srgb_u8(0x23, 0x23, 0x26)))
        .add_systems(Startup, setup)
        .add_observer(splash_event)
        .add_observer(on_fade)
        .add_observer(on_keyframe)
        .add_systems(
            Update,
            (replay_splash, splash_dispatch, elapsed, keyframe).chain(),
        )
        .run();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Bird {
    index: u8,

    fade_in: bool,
    fade_elapsed: f32,

    started: bool,
    keyframe_elapsed: f32,
}

#[derive(Event, Reflect)]
#[reflect(Event)]
enum BirdEvent {
    FadeIn { index: u8 },
    StartKeyframe { index: u8 },
}

#[derive(Reflect, Clone, Default)]
struct KeyFrame {
    start: Transform,
    end: Transform,
}

// Handwritten keyframes (not const because Quat::from_euler isn't const).
fn bird_keyframes() -> [KeyFrame; 3] {
    [
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-203.0, 0.5, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.8),
            },
            end: Transform {
                translation: Vec3::new(-203.0, 0.5, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.8),
            },
        },
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-200.0, 4.0, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.3),
            },
            end: Transform {
                translation: Vec3::new(-175.7, 3.3, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.5),
                scale: Vec3::splat(3.3),
            },
        },
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-235.0, 0.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.0),
            },
            end: Transform {
                translation: Vec3::new(-156.8, -6.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -1.1),
                scale: Vec3::splat(3.0),
            },
        },
    ]
}

#[derive(Resource)]
struct Splash {
    elapsed: f32,

    birds: [Entity; 3],
    built: Entity,
    with: Entity,
    bevy: Entity,

    cutoff: Entity,
}

fn setup(
    mut commands: Commands,
    mut svgs: ResMut<Assets<VelloSvg>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Name::new("Camera"), Camera2d, VelloView, Msaa::Sample4));

    let built = commands
        .spawn((
            Name::new("'Built' text"),
            VelloSvg2d(asset_server.load::<VelloSvg>("built.svg")),
            VelloSvgAnchor::Center,
            Transform::from_xyz(55.0, 60.0, 0.0),
        ))
        .id();

    let with = commands
        .spawn((
            Name::new("'With' text"),
            VelloSvg2d(asset_server.load::<VelloSvg>("with.svg")),
            VelloSvgAnchor::Center,
            Transform::from_xyz(180.0, 60.0, 0.0),
        ))
        .id();

    // commands.spawn((
    //     Name::new("Bird text"),
    //     VelloSvg2d(asset_server.load::<VelloSvg>("bird_text.svg")),
    //     VelloSvgAnchor::Center,
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));

    let bevy = commands
        .spawn((
            Name::new("Bevy text"),
            VelloSvg2d(asset_server.load::<VelloSvg>("bevy_text.svg")),
            VelloSvgAnchor::Center,
            Transform::from_xyz(90.0, -20.0, 0.0),
        ))
        .id();

    let bodies: [Handle<VelloSvg>; 3] =
        std::array::from_fn(|i| svgs.add(bake_bird(BIRD_COLORS[i])));

    // has to be another vello svg to interleave
    let cutoff_svg = svgs.add(
        load_svg_from_str(&format!(
            r#"<svg><rect width="1" height="1" fill="{BG_FILL}"/></svg>"#
        ))
        .expect("cutoff svg failed to parse"),
    );

    let cutoff = commands
        .spawn((
            Name::new("Tail/wing cutoff"),
            VelloSvg2d(cutoff_svg),
            VelloSvgAnchor::Center,
            Transform {
                translation: Vec3::new(-230.0, 35.0, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -0.3),
                scale: Vec3::new(150.0, 300.0, 1.0),
            },
            // Visibility::Hidden,
        ))
        .id();

    let mut splash = Splash {
        elapsed: 0.0,

        birds: [Entity::PLACEHOLDER; 3],
        built,
        with,
        bevy,
        cutoff,
    };

    for index in 0u8..3 {
        let z: f32 = match index {
            0 => 3.0,
            1 => 1.0,
            _ => 0.0,
        };

        let bird = commands
            .spawn((
                Name::new(BIRD_NAMES[index as usize]),
                VelloSvg2d(bodies[index as usize].clone()),
                VelloSvgAnchor::Center,
                Transform::from_xyz(BIRD_ANCHOR.x, BIRD_ANCHOR.y, z)
                    .with_scale(Vec3::splat(BIRD_SCALE)),
                Bird {
                    index,

                    fade_in: false,
                    fade_elapsed: 0.0,

                    started: false,
                    keyframe_elapsed: 0.0,
                },
                FadeIn::default(),
                KeyframeInterp::from_keyframe(bird_keyframes()[index as usize].clone()),
            ))
            .id();

        splash.birds[index as usize] = bird;
    }

    commands.insert_resource(splash);
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

#[derive(Event)]
pub enum SplashEvent {
    Fade(Entity),
    Keyframe(Entity),
}

fn splash_event(on: On<SplashEvent>, mut commands: Commands) {
    match on.event() {
        SplashEvent::Fade(entity) => commands.trigger(TriggerFadeIn { entity: *entity }),
        SplashEvent::Keyframe(entity) => commands.trigger(TriggerKeyFrame { entity: *entity }),
    }
}

#[derive(EntityEvent)]
pub struct TriggerFadeIn {
    entity: Entity,
}

fn on_fade(on: On<TriggerFadeIn>, mut fade: Query<&mut FadeIn>) {
    let Ok(mut fade_in) = fade.get_mut(on.entity) else {
        return;
    };

    fade_in.start = true;
    fade_in.elapsed = 0.0;
}

#[derive(EntityEvent)]
pub struct TriggerKeyFrame {
    entity: Entity,
}

fn on_keyframe(on: On<TriggerKeyFrame>, mut keyed: Query<&mut KeyframeInterp>) {
    let Ok(mut keyed) = keyed.get_mut(on.entity) else {
        return;
    };

    keyed.start = true;
    keyed.elapsed = 0.0;
}

#[derive(Component, Reflect)]
pub struct FadeIn {
    start: bool,
    elapsed: f32,
}

impl Default for FadeIn {
    fn default() -> Self {
        Self {
            start: false,
            elapsed: 0.0,
        }
    }
}

#[derive(Component, Reflect)]
pub struct KeyframeInterp {
    start: bool,
    elapsed: f32,
    keyframe: KeyFrame,
}

impl KeyframeInterp {
    fn from_keyframe(key: KeyFrame) -> Self {
        Self {
            start: false,
            elapsed: 0.0,
            keyframe: key,
        }
    }
}

// accumulate time for currently animating things
fn elapsed(time: Res<Time>, mut fade: Query<&mut FadeIn>, mut key: Query<&mut KeyframeInterp>) {
    for mut fade in &mut fade {
        if fade.start {
            fade.elapsed += time.delta_secs();
        }
    }

    for mut key in &mut key {
        if key.start {
            key.elapsed += time.delta_secs();
        }
    }
}

// tasks over time
fn splash_dispatch(time: Res<Time>, mut splash: ResMut<Splash>, mut commands: Commands) {
    let last = splash.elapsed;
    splash.elapsed += time.delta_secs();

    let tasks: [(f32, SplashEvent); 6] = [
        (0.0, SplashEvent::Fade(splash.birds[0])),
        (0.0, SplashEvent::Fade(splash.built)),
        (0.0, SplashEvent::Fade(splash.with)),
        (0.0, SplashEvent::Fade(splash.bevy)),
        (FADE_DURATION, SplashEvent::Keyframe(splash.birds[1])),
        (FADE_DURATION, SplashEvent::Keyframe(splash.birds[2])),
    ];

    for (t, event) in tasks {
        if t >= last && t < splash.elapsed {
            commands.trigger(event);
        }
    }
}

fn keyframe(mut keyed: Query<(&mut Transform, &KeyframeInterp)>) {
    for (mut transform, interp) in &mut keyed {
        let eased = EaseFunction::CubicOut.sample_clamped(interp.elapsed / FADE_DURATION);

        transform.translation = interp
            .keyframe
            .start
            .translation
            .lerp(interp.keyframe.end.translation, eased);
        transform.rotation = interp
            .keyframe
            .start
            .rotation
            .slerp(interp.keyframe.end.rotation, eased);
        transform.scale = interp
            .keyframe
            .start
            .scale
            .lerp(interp.keyframe.end.scale, eased);
    }
}
