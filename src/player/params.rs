pub const CAMERA_SPEED: f32 = 0.1;

pub const MAX_VEL: f32 = 300.;
pub const WALL_IMPULSE: f32 = 400.;
pub const WALK_SPEED: f32 = 130.;
pub const AIR_ACCEL: f32 = 0.08;
pub const AIR_DAMPING: f32 = 0.04;
pub const SLIDE_SPEED: f32 = 40.;
pub const WALL_STICK_TIME: f32 = 0.20;

/// The angle (in terms of the dot product)
/// at which the player should break lock-on
/// with a target when hitting a static body.
pub const BREAK_ANGLE: f32 = 0.66;

pub const JUMP_SPEED: f32 = 200.;
pub const JUMP_MAX_DURATION: f32 = 0.2;

pub const DASH_DURATION: f32 = 0.1;
pub const DASH_SPEED: f32 = 1000.;
/// Divides the velocity by this factor _once_ after a dash is completed.
pub const DASH_DECAY: f32 = 2.;

/// Maximum distance for a hook target
pub const TARGET_THRESHOLD: f32 = 256.0;
pub const TERMINAL_VELOCITY2_THRESHOLD: f32 = 60_000.;
