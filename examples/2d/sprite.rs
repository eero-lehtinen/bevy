//! Displays a single [`Sprite`], created from an image.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = Color::srgba(0., 0.5, 0., 1.);

    commands.spawn(Camera2d);
    let size = 500.;

    commands.spawn((
        Sprite {
            color: Color::srgba(1., 1., 1., 0.5),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(-size * 0.5, 0., 0.),
    ));

    let color = Color::srgba(0., 0., 0., 0.5);
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(size * 0.5, 0., 0.),
    ));
}
