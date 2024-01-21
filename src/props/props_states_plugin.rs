use bevy::prelude::*;

use crate::props::ThrustersStatePlugin;
use crate::props::InteractablesStatePlugin;


#[derive(Default)]
pub struct PropsStatesPlugin;

impl Plugin for PropsStatesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((InteractablesStatePlugin::default(),))
        .add_plugins((ThrustersStatePlugin::default(),));
    }
}
