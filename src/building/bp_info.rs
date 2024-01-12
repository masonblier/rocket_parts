use bevy::prelude::*;
use std::collections::HashMap;
use std::iter::FromIterator;
use bevy_rapier3d::prelude::*;

use crate::building::GridSolidity;

#[derive(Clone,Default,Component)]
pub struct BpSnapPoint {
    pub point: Vec3,
    pub normal: Vec3,
    pub cuboid_dims: Vec3,
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
    pub solidity: GridSolidity,
}

#[derive(Clone,Resource)]
pub struct BpInfos {
    pub bps: HashMap<String,BpInfo>,
    pub toolbar_order: Vec<String>,
}

impl Default for BpInfos {
    
    fn default() -> BpInfos {
        let hm_all = vec![
            BpSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                cuboid_dims: Vec3::new(0.8, 0.2, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                cuboid_dims: Vec3::new(0.8, 0.2, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: Vec3::X/2.,
                normal: Vec3::X,
                cuboid_dims: Vec3::new(0.2, 0.8, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::X/2.,
                normal: -Vec3::X,
                cuboid_dims: Vec3::new(0.2, 0.8, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: Vec3::Z/2.,
                normal: Vec3::Z,
                cuboid_dims: Vec3::new(0.8, 0.8, 0.2),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Z/2.,
                normal: -Vec3::Z,
                cuboid_dims: Vec3::new(0.8, 0.8, 0.2),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
        ];

        let hm_tank = vec![
            BpSnapPoint {
                point: Vec3::Y/2.,
                normal: Vec3::Y,
                cuboid_dims: Vec3::new(0.8, 0.2, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
            BpSnapPoint {
                point: -Vec3::Y/2.,
                normal: -Vec3::Y,
                cuboid_dims: Vec3::new(0.8, 0.2, 0.8),
                filter: BpSnapFilter::HalfMeterBlocks,
            },
        ];
        
        let bps: HashMap<String,BpInfo> = HashMap::<_, _>::from_iter([
            ("metal_frame".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cuboid(0.5, 0.5, 0.5),
                snap: hm_all.clone(),
                solidity: GridSolidity::Leaky,
            }),
            ("fuel_tank".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cylinder(0.5, 0.5),
                snap: hm_tank.clone(),
                solidity: GridSolidity::Leaky,
            }),
            ("thruster".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cylinder(0.5, 0.5),
                snap: vec![hm_tank[0].clone()],
                solidity: GridSolidity::Leaky,
            }),
            ("nose_cone".to_string(), BpInfo {
                bottom: -Vec3::Y/2.,
                collider: Collider::cone(0.5, 0.5),
                snap: vec![],
                solidity: GridSolidity::Leaky,
            }),
        ].into_iter());

        let toolbar_order = vec![
            "metal_frame".to_string(),
            "fuel_tank".to_string(),
            "thruster".to_string(),
            "nose_cone".to_string(),
        ];

        BpInfos { bps, toolbar_order }
    }
}
