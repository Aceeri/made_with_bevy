use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_vello::{VelloPlugin, integrations::svg::load_svg_from_str, prelude::*};

const BIRD_SVG: &str = include_str!("../assets/bird-0.svg");
/// What color to replace
const BIRD_SOURCE_FILL: &str = "#ececec";
/// outline/clear color
const BG_FILL: &str = "#232326";
const BIRD_COLORS: [&str; 3] = ["#ececec", "#b2b2b2", "#787878"];

const BIRD_SCALE: f32 = 6.0;
const OUTLINE_THICKNESS: f32 = 0.10;
const FADE_DURATION: f32 = 0.4;
const SLIDE_DURATION: f32 = 0.7;
const BIRD_ANCHOR: Vec2 = Vec2::new(-110.0, 25.0);
const SLIDE_OFFSET: Vec2 = Vec2::new(110.0, -25.0);

#[derive(Component)]
struct Bird(u8);

#[derive(Resource)]
struct Splash {
    elapsed: f32,
    front_body: Handle<VelloSvg>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(VelloPlugin::default())
        .insert_resource(ClearColor(Color::srgb_u8(0x23, 0x23, 0x26)))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_splash)
        .run();
}

fn setup(mut commands: Commands, mut svgs: ResMut<Assets<VelloSvg>>) {
    commands.spawn((Camera2d, VelloView));

    let bodies: [Handle<VelloSvg>; 3] = std::array::from_fn(|i| {
        let initial_alpha = if i == 0 { 0.0 } else { 1.0 };
        svgs.add(bake_bird(BIRD_COLORS[i], initial_alpha))
    });

    for index in 0..3 {
        let z = (2 - index) as f32;

        commands
            .spawn((
                VelloSvg2d(bodies[index as usize].clone()),
                VelloSvgAnchor::Center,
                Transform::from_xyz(BIRD_ANCHOR.x, BIRD_ANCHOR.y, z)
                    .with_scale(Vec3::splat(BIRD_SCALE)),
                Bird(index),
            ));
    }

    commands.insert_resource(Splash {
        elapsed: 0.0,
        front_body: bodies[0].clone(),
    });
}

fn bake_bird(fill_hex: &str, alpha: f32) -> VelloSvg {
    let recolored = BIRD_SVG.replace(BIRD_SOURCE_FILL, fill_hex);
    let mut asset = load_svg_from_str(&recolored).expect("bird svg failed to parse");
    asset.alpha = alpha;
    asset
}

fn animate_splash(
    time: Res<Time>,
    mut splash: ResMut<Splash>,
    mut svgs: ResMut<Assets<VelloSvg>>,
    mut birds: Query<(&Bird, &mut Transform, &mut Visibility)>,
) {
    splash.elapsed += time.delta_secs();
    let t = splash.elapsed;

    if t < FADE_DURATION {
        if let Some(svg) = svgs.get_mut(&splash.front_body) {
            svg.alpha = (t / FADE_DURATION).clamp(0.0, 1.0);
        }
        return;
    }

    if let Some(svg) = svgs.get_mut(&splash.front_body)
        && svg.alpha < 1.0
    {
        svg.alpha = 1.0;
    }

    let slide_t = ((t - FADE_DURATION) / SLIDE_DURATION).clamp(0.0, 1.0);
    let eased = ease_out_cubic(slide_t);

    for (bird, mut transform, mut visibility) in &mut birds {
        if bird.index == 0 {
            continue;
        }
        if matches!(*visibility, Visibility::Hidden) {
            *visibility = Visibility::Visible;
        }
        let mult = bird.index as f32;
        transform.translation.x = BIRD_ANCHOR.x + SLIDE_OFFSET.x * mult * eased;
        transform.translation.y = BIRD_ANCHOR.y + SLIDE_OFFSET.y * mult * eased;
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let p = 1.0 - t;
    1.0 - p * p * p
}
