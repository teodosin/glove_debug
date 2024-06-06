mod asyncs;
mod ble;
mod particles;
mod ruka;

use bevy::{math::{Affine3A, Mat3A}, prelude::*};
use bevy_gaussian_splatting::{GaussianCloudSettings, GaussianSplattingBundle, GaussianSplattingPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use ble::BLEPlugin;
use ruka::RukaPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BLEPlugin)

        .add_plugins(GaussianSplattingPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(RukaPlugin)
        //.add_plugins(WorldInspectorPlugin::new())
        

        .add_systems(Startup, setup_gaussian)
        .add_systems(Update, listen)
    .run();
}

fn setup_gaussian(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
    commands.spawn((
        GaussianSplattingBundle {
            cloud: asset_server.load("Spir.gcloud"),
            settings: GaussianCloudSettings {
                global_transform: GlobalTransform::from(Mat4 {
                    x_axis: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    y_axis: Vec4::new(0.0, -1.0, 0.0, 0.0),
                    z_axis: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    w_axis: Vec4::new(0.0, 0.0, 0.0, 1.0),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    commands.spawn((
        Camera3dBundle::default(),
        PanOrbitCamera::default(),
    ));

    commands.spawn(
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..Default::default()
            },
            ..Default::default()
        },
    );
}





fn listen() {
    // nothing right now
}

