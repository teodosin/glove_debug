mod asyncs;
mod ble;
mod particles;

use bevy::{math::{Affine3A, Mat3A}, prelude::*};
use bevy_gaussian_splatting::{GaussianCloudSettings, GaussianSplattingBundle, GaussianSplattingPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use ble::BLEPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BLEPlugin)

        .add_plugins(GaussianSplattingPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        
        .insert_resource(RukaInput::default())

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
            cloud: asset_server.load("Trii.gcloud"),
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
}

#[derive(Resource, Default)]
pub struct RukaInput {
    init: bool,
    fingers: [u16; 5]
}

impl RukaInput {
    pub fn init(&self) -> bool {
        self.init
    }

    pub fn set_init(&mut self, init: bool) {
        self.init = init;
    }

    pub fn get_fingers(&self) -> [f32; 5] {
        let mut fingers = [0.0; 5];
        for (i, finger) in self.fingers.iter().enumerate() {
            fingers[i] = *finger as f32 / 16384.0;
        }
        fingers
    }

    pub fn update_fingers(&mut self, new_fingers: [u16; 5]) {
        self.fingers = new_fingers;
    }

    pub fn update_finger(&mut self, finger: usize, value: u16) {
        if finger > 4 {
            panic!("Finger index out of bounds");
        }
        self.fingers[finger] = value;
    }
}



fn listen() {
    // nothing right now
}

