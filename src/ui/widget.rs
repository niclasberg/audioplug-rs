use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
};

use crate::{AnimationFrame, KeyEvent, MouseEvent};

use super::{EventContext, RenderContext, animation::AnimationContext};
