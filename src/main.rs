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
const FADE_DURATION: f32 = 0.6;
const KEYFRAME_DURATION: f32 = 0.8;
const FADE_OUT_START: f32 = 2.4;
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
        .add_observer(on_reset)
        .add_systems(
            Update,
            (replay_splash, splash_dispatch, elapsed, fade, keyframe).chain(),
        )
        .run();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Bird {
    index: u8,
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
    start: bool,
    elapsed: f32,

    birds: [Entity; 3],
    built: Entity,
    with: Entity,
    bevy: Entity,

    cutoff: Entity,
    overlay: Entity,
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
            Visibility::Hidden,
            Fade::default(),
        ))
        .id();

    let with = commands
        .spawn((
            Name::new("'With' text"),
            VelloSvg2d(asset_server.load::<VelloSvg>("with.svg")),
            VelloSvgAnchor::Center,
            Transform::from_xyz(180.0, 60.0, 0.0),
            Visibility::Hidden,
            Fade::default(),
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
            Visibility::Hidden,
            Fade::default(),
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
            Visibility::Hidden,
        ))
        .id();

    let overlay_svg = svgs.add(
        load_svg_from_str(&format!(
            r#"<svg><rect width="1" height="1" fill="{BG_FILL}"/></svg>"#
        ))
        .expect("overlay svg failed to parse"),
    );

    let overlay = commands
        .spawn((
            Name::new("Fade-out overlay"),
            VelloSvg2d(overlay_svg),
            VelloSvgAnchor::Center,
            Transform::from_xyz(0.0, 0.0, 100.0).with_scale(Vec3::new(3000.0, 3000.0, 1.0)),
            Visibility::Hidden,
            Fade::default(),
        ))
        .id();

    let mut splash = Splash {
        start: false,
        elapsed: 0.0,

        birds: [Entity::PLACEHOLDER; 3],
        built,
        with,
        bevy,
        cutoff,
        overlay,
    };

    for index in 0u8..3 {
        let z: f32 = match index {
            0 => 3.0,
            1 => 1.0,
            _ => 0.0,
        };

        let mut bird_cmds = commands.spawn((
            Name::new(BIRD_NAMES[index as usize]),
            VelloSvg2d(bodies[index as usize].clone()),
            VelloSvgAnchor::Center,
            Transform::from_xyz(BIRD_ANCHOR.x, BIRD_ANCHOR.y, z)
                .with_scale(Vec3::splat(BIRD_SCALE)),
            Visibility::Hidden,
            Bird { index },
            KeyframeInterp::from_keyframe(bird_keyframes()[index as usize].clone()),
        ));
        if index == 0 {
            bird_cmds.insert(Fade::default());
        }
        splash.birds[index as usize] = bird_cmds.id();
    }

    commands.insert_resource(splash);
}

fn bake_bird(fill_hex: &str) -> VelloSvg {
    let recolored = BIRD_SVG.replace(BIRD_SOURCE_FILL, fill_hex);
    load_svg_from_str(&recolored).expect("bird svg failed to parse")
}

fn replay_splash(
    keys: Res<ButtonInput<KeyCode>>,
    mut splash: ResMut<Splash>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        commands.trigger(ResetSplash);
    }
    if keys.just_pressed(KeyCode::Space) {
        splash.start = true;
    }
}

#[derive(Event)]
pub struct ResetSplash;

fn on_reset(
    _: On<ResetSplash>,
    mut splash: ResMut<Splash>,
    mut visibilities: Query<&mut Visibility>,
    mut keyframes: Query<&mut KeyframeInterp>,
    mut commands: Commands,
) {
    splash.start = false;
    splash.elapsed = 0.0;

    let all: [Entity; 8] = [
        splash.birds[0],
        splash.birds[1],
        splash.birds[2],
        splash.built,
        splash.with,
        splash.bevy,
        splash.cutoff,
        splash.overlay,
    ];
    for entity in all {
        if let Ok(mut vis) = visibilities.get_mut(entity) {
            *vis = Visibility::Hidden;
        }
        commands.entity(entity).remove::<Fade>();
    }
    for entity in [
        splash.birds[0],
        splash.built,
        splash.with,
        splash.bevy,
        splash.overlay,
    ] {
        commands.entity(entity).insert(Fade::default());
    }
    for entity in splash.birds {
        if let Ok(mut interp) = keyframes.get_mut(entity) {
            interp.start = false;
            interp.elapsed = 0.0;
        }
    }
}

#[derive(Event)]
pub enum SplashEvent {
    Fade(Entity),
    Keyframe(Entity),
    Show(Entity),
}

fn splash_event(on: On<SplashEvent>, mut commands: Commands) {
    match on.event() {
        SplashEvent::Fade(entity) => commands.trigger(TriggerFade { entity: *entity }),
        SplashEvent::Keyframe(entity) => commands.trigger(TriggerKeyFrame { entity: *entity }),
        SplashEvent::Show(entity) => {
            commands.entity(*entity).insert(Visibility::Visible);
        }
    }
}

#[derive(EntityEvent)]
pub struct TriggerFade {
    entity: Entity,
}

fn on_fade(on: On<TriggerFade>, mut q: Query<(&mut Fade, &mut Visibility)>) {
    let Ok((mut fade, mut visibility)) = q.get_mut(on.entity) else {
        return;
    };

    fade.start = true;
    fade.elapsed = 0.0;
    *visibility = Visibility::Visible;
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
pub struct Fade {
    start: bool,
    elapsed: f32,
    from: f32,
    to: f32,
}

impl Default for Fade {
    fn default() -> Self {
        Self {
            start: false,
            elapsed: 0.0,
            from: 0.0,
            to: 1.0,
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
fn elapsed(time: Res<Time>, mut fade: Query<&mut Fade>, mut key: Query<&mut KeyframeInterp>) {
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
    if !splash.start {
        return;
    }
    let last = splash.elapsed;
    splash.elapsed += time.delta_secs();

    let tasks: [(f32, SplashEvent); 10] = [
        (0.0, SplashEvent::Fade(splash.birds[0])),
        (0.0, SplashEvent::Fade(splash.built)),
        (0.0, SplashEvent::Fade(splash.with)),
        (0.0, SplashEvent::Fade(splash.bevy)),
        (0.8, SplashEvent::Show(splash.cutoff)),
        (0.8, SplashEvent::Show(splash.birds[1])),
        (0.8, SplashEvent::Show(splash.birds[2])),
        (0.8, SplashEvent::Keyframe(splash.birds[1])),
        (0.8, SplashEvent::Keyframe(splash.birds[2])),
        (FADE_OUT_START, SplashEvent::Fade(splash.overlay)),
    ];

    for (t, event) in tasks {
        if t >= last && t < splash.elapsed {
            commands.trigger(event);
        }
    }

    let auto_reset = FADE_OUT_START + FADE_DURATION;
    if auto_reset >= last && auto_reset < splash.elapsed {
        commands.trigger(ResetSplash);
    }
}

fn fade(fades: Query<(&Fade, &VelloSvg2d)>, mut svgs: ResMut<Assets<VelloSvg>>) {
    for (fade, vello) in &fades {
        let alpha = if fade.start {
            let t = (fade.elapsed / FADE_DURATION).clamp(0.0, 1.0);
            fade.from + (fade.to - fade.from) * t
        } else {
            fade.from
        };
        if let Some(svg) = svgs.get_mut(&vello.0) {
            svg.alpha = alpha;
        }
    }
}

fn keyframe(mut keyed: Query<(&mut Transform, &KeyframeInterp)>) {
    for (mut transform, interp) in &mut keyed {
        let eased = EaseFunction::CubicOut.sample_clamped(interp.elapsed / KEYFRAME_DURATION);

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
