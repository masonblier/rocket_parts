use bevy::prelude::*;

use crate::props::ThrustersStatePlugin;


#[derive(Default)]
pub struct PropsStatesPlugin;

impl Plugin for PropsStatesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((ThrustersStatePlugin::default(),));
    }
}
