//! Simple animation support

use crate::easing::{Easing, EasingKind};
use std::time::Instant;

pub trait AnimationAction {
    fn apply(&mut self, t: f32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Once,
    Repeat,
    Wiggle,
}

pub struct Animation {
    action: Box<dyn AnimationAction>,
    duration: f32,
    kind: AnimationType,
    easing: EasingKind,
    start_time: Option<f32>,
    reverse: bool,
}

impl Animation {
    pub fn new(
        action: Box<dyn AnimationAction>,
        duration_secs: f32,
        kind: AnimationType,
        easing: EasingKind,
    ) -> Self {
        Self {
            action,
            duration: duration_secs,
            kind,
            easing,
            start_time: None,
            reverse: false,
        }
    }

    pub fn end(&mut self) {
        self.action.apply(1.0);
    }

    pub fn animate(&mut self, current_time: f32) -> bool {
        if self.start_time.is_none() {
            self.action.apply(0.0);
            self.start_time = Some(current_time);
            return true;
        }

        let start = self.start_time.unwrap_or(0.0);
        let elapsed = current_time - start;

        if elapsed >= self.duration {
            self.action.apply(1.0);

            if self.kind == AnimationType::Once {
                return false;
            }

            if self.kind == AnimationType::Wiggle {
                self.reverse = !self.reverse;
            }

            if elapsed > self.duration {
                self.start_time = Some(start + self.duration);
            }

            return true;
        }

        let mut pos = elapsed / self.duration;
        if self.reverse {
            pos = 1.0 - pos;
        }

        let eased = Easing::easing_function(pos, self.easing);
        self.action.apply(eased);
        true
    }
}

pub struct AnimationQueue {
    start: Instant,
    last_action_time: f32,
    idle_time: f32,
    animations: Vec<Animation>,
}

impl AnimationQueue {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_action_time: 0.0,
            idle_time: 5.0,
            animations: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        for anim in &mut self.animations {
            anim.end();
        }
        self.animations.clear();
    }

    pub fn pulse(&mut self) -> bool {
        let current = self.start.elapsed().as_secs_f32();
        let mut update_needed = false;
        let mut to_remove = Vec::new();

        for (index, anim) in self.animations.iter_mut().enumerate() {
            update_needed = true;
            if !anim.animate(current) {
                to_remove.push(index);
            }
            self.last_action_time = current;
        }

        for index in to_remove.into_iter().rev() {
            self.animations.remove(index);
        }

        update_needed
    }

    pub fn is_idle(&self) -> bool {
        let current = self.start.elapsed().as_secs_f32();
        current - self.last_action_time > self.idle_time
    }

    pub fn add(&mut self, anim: Animation) {
        self.animations.push(anim);
    }
}

impl Default for AnimationQueue {
    fn default() -> Self {
        Self::new()
    }
}
