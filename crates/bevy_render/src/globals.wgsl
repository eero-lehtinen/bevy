#define_import_path bevy_render::globals

struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    time: f32,
    // The delta time since the previous frame in seconds
    delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,

    light_color: vec4<f32>,
    ambient_sky_color: vec4<f32>,
    ambient_ground_color: vec4<f32>,
};
