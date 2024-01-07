use crate::loading::{LoadingUiEvent,LoadingUiEventAction,
    WorldProps};
use crate::game_state::GameState;
use crate::world::{InteractableState,WorldAsset,WorldState,
    WorldSoundState,AnimatableState};
use bevy::prelude::*;
use bevy::scene::InstanceId;
use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use bevy_rapier3d::prelude::*;

#[derive(Clone,Default,Component)]
pub struct HmSnapPoint {
    pub point: Vec3,
    pub normal: Vec3,
    pub collider: Collider,
}

#[derive(Clone,Default)]
pub struct BpInfo {
    pub bottom: Vec3,
    pub collider: Collider,
    pub snap: Vec<HmSnapPoint>,
}

#[derive(Clone,Resource)]
pub struct BpInfos {
    pub bps: HashMap<String,BpInfo>,
}

impl Default for BpInfos {
    
    fn default() -> BpInfos {
        let hm_all = vec![
            HmSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
            },
            HmSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
            },
            HmSnapPoint {
                point: Vec3::X/2.,
                normal: Vec3::X,
                collider: Collider::cuboid(0.1, 0.4, 0.4),
            },
            HmSnapPoint {
                point: -Vec3::X/2.,
                normal: -Vec3::X,
                collider: Collider::cuboid(0.1, 0.4, 0.4),
            },
            HmSnapPoint {
                point: Vec3::Z/2.,
                normal: Vec3::Z,
                collider: Collider::cuboid(0.4, 0.4, 0.1),
            },
            HmSnapPoint {
                point: -Vec3::Z/2.,
                normal: -Vec3::Z,
                collider: Collider::cuboid(0.4, 0.4, 0.1),
            },
        ];
            // down: Some(-Vec3::Y/2.),
            // left: Some(-Vec3::Y/2.),
            // right: Some(-Vec3::Y/2.),
            // forward: Some(-Vec3::Y/2.),
            // backward: Some(-Vec3::Y/2.),

        let hm_tank = vec![
            HmSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
            },
            HmSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
            },
        ];
        
        let bps: HashMap<String,BpInfo> = HashMap::<_, _>::from_iter(IntoIter::new([
            ("metal_frame".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cuboid(0.5, 0.5, 0.5),
                snap: hm_all.clone(),
            }),
            ("fuel_tank".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cylinder(0.5, 0.5),
                snap: hm_tank.clone(),
            }),
            ("nose_cone".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cone(0.5, 0.5),
                snap: vec![],
            }),
            ("thruster".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cylinder(0.5, 0.5),
                snap: hm_tank.clone(),
            }),
        ]));

        BpInfos { bps }
    }
}
