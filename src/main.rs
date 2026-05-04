use bevy::{
    asset::AssetMetaCheck,
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::common_conditions::input_toggle_active,
    prelude::*,
    window::PresentMode,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_vello::VelloPlugin;
use built_with_bevy::{
    BevySplashscreenEnded, BevySplashscreenPlugin, SkipBevySplashscreen, StartBevySplashscreen,
};
use iyes_perf_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(VelloPlugin::default())
        .add_plugins(EguiPlugin::default())
        // .add_plugins(
        //     WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        // )
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(EntityCountDiagnosticsPlugin::default())
        // .add_plugins(SystemInformationDiagnosticsPlugin)
        // .add_plugins(PerfUiPlugin)
        .add_plugins(BevySplashscreenPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, debug_keys)
        .add_observer(on_ended)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot {
            display_labels: true,
            fontsize_label: 10.0,
            fontsize_value: 10.0,
            values_col_width: 50.0,
            inner_margin: -2.0,
            inner_padding: -2.0,
            ..default()
        },
        PerfUiEntryFPS::default(),
        PerfUiEntryFPSAverage::default(),
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFrameTimeWorst::default(),
        PerfUiEntryEntityCount::default(),
        PerfUiEntryCpuUsage::default(),
        PerfUiEntryMemUsage::default(),
    ));

    commands.trigger(StartBevySplashscreen);
}

fn debug_keys(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.trigger(StartBevySplashscreen);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        commands.trigger(SkipBevySplashscreen);
    }
}

fn on_ended(_: On<BevySplashscreenEnded>) {
    info!("splashscreen ended");
}
