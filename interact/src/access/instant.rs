use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;

use crate::access::ReflectDirect;
use crate::climber::{ClimbError, Climber};
use crate::node_tree::{NodeInfo, NodeTree};
use crate::reflector::Reflector;

impl ReflectDirect for Instant {
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let meta = try_seen_dyn!(self, reflector);
        NodeInfo::Leaf(Cow::Owned(format!("{:?}", self.elapsed()))).with_meta(meta)
    }

    fn immut_climber<'a>(
        &self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        return Ok(None);
    }

    fn mut_climber<'a>(
        &mut self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        return Ok(None);
    }
}

use interact_derive::derive_interact_opaque;

derive_interact_opaque! {
    struct Instant;
}
