use bevy::{
    app::{App, Plugin, Update}, core_pipeline::core_3d::Camera3d, ecs::{
        component::Component, entity::Entity, query::With, system::{Commands, Query, Res, ResMut, Resource}
    }, hierarchy::BuildChildren, input::{keyboard::KeyCode, ButtonInput}, math::Vec3, render::color::Color, sprite::Anchor, text::{Text, Text2dBundle, TextSection, TextStyle}, time::Time, transform::components::Transform
};

pub struct RukaPlugin;

impl Plugin for RukaPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(RukaInput::default())
            .add_systems(Update, toggle_ruka_debug)
            .add_systems(Update, update_ruka_debug)
            .add_systems(Update, update_ruka_cam)
        ;

    
    }
}

#[derive(Resource, Default)]
pub struct RukaInput {
    init: bool,

    fingers: [u16; 5],
    finger_limits: [(u16, u16); 5],

    accel: Vec3,
    gyro: Vec3,
}

impl RukaInput {
    pub fn is_init(&self) -> bool {
        self.init
    }

    pub fn set_init(&mut self, init: bool) {
        self.init = init;
    }

    pub fn get_fingers_float(&self) -> [f32; 5] {
        let mut fingers = [0.0; 5];
        for (i, finger) in self.fingers.iter().enumerate() {
            fingers[i] = *finger as f32 / 16384.0;
        }
        fingers
    }

    pub fn update_fingers(&mut self, new_fingers: [u16; 5]) {
        for (i, finger) in new_fingers.iter().enumerate() {
            if self.finger_limits[i].0 < 100 {
                self.finger_limits[i].0 = 14000;
            }
            if *finger < self.finger_limits[i].0 {
                self.finger_limits[i].0 = *finger;
            }
            if *finger > self.finger_limits[i].1 {
                self.finger_limits[i].1 = *finger;
            }
        }

        println!("Limits: {:?}", self.finger_limits);
        self.fingers = new_fingers;
    }

    pub fn update_imu(&mut self, new_imu: [f32; 6]) {
        self.accel = Vec3::new(new_imu[0], new_imu[1], new_imu[2]);
        self.gyro = Vec3::new(new_imu[3], new_imu[4], new_imu[5]);
    }

    pub fn get_gyro(&self) -> Vec3 {
        self.gyro
    }

    pub fn get_accel(&self) -> Vec3 {
        self.accel
    }

    pub fn get_all_for_debug(&self) -> [f32; 12] {
        let all = [
            self.fingers[0] as f32,
            self.fingers[1] as f32,
            self.fingers[2] as f32,
            self.fingers[3] as f32,
            self.fingers[4] as f32,
            self.accel.x,
            self.accel.y,
            self.accel.z,
            self.gyro.x,
            self.gyro.y,
            self.gyro.z,
            self.get_gesture().to_float()
        ];
        all
    }

    pub fn get_gesture(&self) -> RukaGesture {
        // Naive implementation of checking whether the hand is making a fist
        
        let mut is_fist: bool = true;

        for i in 0..self.fingers.len(){
            // Threshold for making a fist is 0.3 along the limit for that finger
            let from_straight = self.fingers[i].abs_diff(self.finger_limits[i].1);
            let from_flexed = self.fingers[i].abs_diff(self.finger_limits[i].0);

            let deny_fist = from_straight as f32 <= from_flexed as f32;
            is_fist = !deny_fist;

            println!("Finger {}: | ll{} c{} ul{} | s{} f{}, denying fist: {}", 
                i, 
                self.finger_limits[i].0, self.fingers[i], self.finger_limits[i].1, 
                from_straight, from_flexed, deny_fist);
        }

        if !is_fist {
            RukaGesture::Fist
        } else {
            RukaGesture::Idle
        }
    }
}

#[derive(PartialEq)]
pub enum RukaGesture {
    Idle,
    Fist, 
    ThumbsUp,
}

impl RukaGesture {
    pub fn to_string(&self) -> String {
        match self {
            RukaGesture::Idle => "Idle".to_string(),
            RukaGesture::Fist => "Fist".to_string(),
            RukaGesture::ThumbsUp => "ThumbsUp".to_string(),
        }
    }

    pub fn to_float(&self) -> f32 {
        match self {
            RukaGesture::Idle => 0.0,
            RukaGesture::Fist => 1.0,
            RukaGesture::ThumbsUp => 2.0,
        }
    }

    pub fn from_float(val: f32) -> RukaGesture {
        match val {
            0.0 => RukaGesture::Idle,
            1.0 => RukaGesture::Fist,
            2.0 => RukaGesture::ThumbsUp,
            _ => RukaGesture::Idle,
        }
    }

    pub fn float_to_string(&self) -> String {
        match self {
            RukaGesture::Idle => "Idle".to_string(),
            RukaGesture::Fist => "Fist".to_string(),
            RukaGesture::ThumbsUp => "ThumbsUp".to_string(),
        }
    }
}

#[derive(Component)]
struct RukaDebugLabel;

#[derive(Component)]
struct RukaDebugFinger;

fn toggle_ruka_debug(
    mut commands: Commands, 
    ruka: Res<RukaInput>,
    keys: Res<ButtonInput<KeyCode>>,
    labels: Query<Entity, With<RukaDebugLabel>>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    if labels.iter().count() == 0 {
        let mut i = 0;
        for lbl in ruka.get_all_for_debug().iter(){
            commands.spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("{}: {}", i + 1, lbl),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                               ..Default::default()
                            },
                           ..Default::default()
                        }],
                        ..Default::default()
                    },
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_xyz(20.0, 20.0 + i as f32 * 20.0, 1000.0),
                    ..Default::default()
                },
                RukaDebugLabel,
            ));

            i += 1;
        }
    }

    else {
        labels.iter().for_each(|label| commands.entity(label).despawn());
    }
}

fn update_ruka_debug(
    ruka: Res<RukaInput>,
    mut labels: Query<&mut Text, With<RukaDebugLabel>>,
) {
    let mut i = 0;
    let fist: bool = ruka.get_gesture() == RukaGesture::Fist;
    for mut lbl in labels.iter_mut() {
        lbl.sections[0].value = format!("{:.2}", ruka.get_all_for_debug()[i]);
        lbl.sections[0].style.color = match fist {
            true => Color::GREEN,
            false => Color::WHITE
        };
        i += 1;
    }
}

fn update_ruka_cam(
    ruka: Res<RukaInput>,
    time: Res<Time>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
){
    if ruka.get_gesture() != RukaGesture::Fist {
        return;
    }

    let mut cam_transform = cam.single_mut();
    let gyro = ruka.get_gyro() * time.delta_seconds() * 0.01;
    // Rotate the camera based on gyro values
    cam_transform.rotate_local_x(-gyro.x);
    cam_transform.rotate_local_y(gyro.z * 0.6);
    cam_transform.rotate_z(- gyro.y * 1.2);

    // cam_transform.translation.x += ruka.get_accel().x * time.delta_seconds() * 0.01;
    // cam_transform.translation.z += ruka.get_accel().y * time.delta_seconds() * 0.01;
    // cam_transform.translation.y += (ruka.get_accel().z + 9.7) * time.delta_seconds() * 0.01;
}
