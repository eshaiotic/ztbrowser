//! A script the browser injects into pages.

use crate::engine::injection_point::InjectionPoint;
use crate::engine::script_world::ScriptWorld;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageScript {
    pub name: String,
    pub source: String,
    pub injection_point: InjectionPoint,
    pub runs_in_all_frames: bool,
    pub world: ScriptWorld,
}
