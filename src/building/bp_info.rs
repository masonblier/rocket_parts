use bevy::prelude::*;
use std::collections::HashMap;
use std::iter::FromIterator;
use bevy_rapier3d::prelude::*;

#[derive(Clone,Default,Component)]
pub struct BpSnapPoint {
    pub point: Vec3,
    pub normal: Vec3,
    pub collider: Collider,
    pub filter: BpSnapFilter,
}

#[derive(Clone,Default)]
pub enum BpSnapFilter {
    #[default]
    HalfMeterBlocks,
    // WallBlocks,
}

#[derive(Clone,Default)]
pub struct BpInfo {
    pub bottom: Vec3,
    pub collider: Collider,
    pub snap: Vec<BpSnapPoint>,
}

#[derive(Clone,Resource)]
pub struct BpInfos {
    pub bps: HashMap<String,BpInfo>,
}

impl Default for BpInfos {
    
    fn default() -> BpInfos {
        let hm_all = vec![
            BpSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: Vec3::X/2.,
                normal: Vec3::X,
                collider: Collider::cuboid(0.1, 0.4, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::X/2.,
                normal: -Vec3::X,
                collider: Collider::cuboid(0.1, 0.4, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: Vec3::Z/2.,
                normal: Vec3::Z,
                collider: Collider::cuboid(0.4, 0.4, 0.1),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Z/2.,
                normal: -Vec3::Z,
                collider: Collider::cuboid(0.4, 0.4, 0.1),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
        ];

        let hm_tank = vec![
            BpSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                collider: Collider::cuboid(0.4, 0.1, 0.4),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
        ];
        
        let bps: HashMap<String,BpInfo> = HashMap::<_, _>::from_iter([
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
        ].into_iter());

        BpInfos { bps }
    }
}
