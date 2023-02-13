use bevy::{
    app::App, prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep, window::PresentMode,
};
use rand::Rng;

const DELTA_TIME: f64 = 0.01;

pub const BG_COLOR: Color = Color::rgb(0.144, 0.144, 0.144);

pub const MAX_BUBBLES: u32 = 15;

pub const MAX_RADIUS: f32 = 40.0;
pub const MAX_VELOCITY: f32 = 0.75;

pub const RADIUS_RANGE: std::ops::Range<f32> = 3.0..MAX_RADIUS;
pub const VEL_RANGE: std::ops::RangeInclusive<f32> = -MAX_VELOCITY..=MAX_VELOCITY;

#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
struct InteractBodies;

fn main() {
    App::new()
        .insert_resource(ClearColor(BG_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Path Walker".to_string(),
                present_mode: PresentMode::AutoVsync,
                width: 1200.,
                height: 650.,

                ..default()
            },
            ..default()
        }))
        .add_startup_system(setup)
        .add_stage_after(
            CoreStage::Update,
            InteractBodies,
            SystemStage::single_threaded()
                .with_run_criteria(FixedTimestep::step(DELTA_TIME))
                .with_system(move_bubbles)
                .with_system(interact_bubbles)
        )
        .add_system(border_collision)
        .run();
}

#[derive(Component, Default)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component, Clone, Copy)]
struct Radius(f32);

#[derive(Bundle)]
pub struct BubbleBundle {
    velocity: Velocity,
    radius: Radius,
    material: MaterialMesh2dBundle<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut windows: ResMut<Windows>,
) {
    commands.spawn(Camera2dBundle::default());

    let window = windows.get_primary_mut().unwrap();
    window.center_window(MonitorSelection::Primary);

    let (win_w, win_h) = (window.width(), window.height());

    let mut rng = rand::thread_rng();

    for _ in 0..MAX_BUBBLES {
        let position = Vec3 {
            x: rng.gen_range(-(win_w / 2.0)..=(win_w / 2.0)),
            y: rng.gen_range(-(win_h / 2.0)..=(win_h / 2.0)),
            z: 0.0,
        };

        let circle = meshes.add(
            shape::Circle::new(rng.gen_range(RADIUS_RANGE)).into(),
            // shape::Circle::new(MAX_RADIUS).into()
        );

        commands.spawn(BubbleBundle {
            velocity: Velocity {
                x: rng.gen_range(VEL_RANGE),
                y: rng.gen_range(VEL_RANGE),
            },

            radius: Radius(rng.gen_range(RADIUS_RANGE)),

            material: MaterialMesh2dBundle::<ColorMaterial> {
                mesh: circle.into(),
                material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                transform: Transform::from_translation(position),
                ..default()
            },
        });
    }
}


// Basic move mechanic ( to upgrade )
fn move_bubbles(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, vel) in query.iter_mut() {
        transform.translation.x += vel.x;
        transform.translation.y += vel.y;
    }
}

fn interact_bubbles(
    mut query: Query<(Entity, &mut Transform, &Radius), With<Velocity>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (win_w, win_h) = (window.width(), window.height());

    let mut rng = rand::thread_rng();

    let mut iter = query.iter_combinations_mut();

    while let Some([(entity_1, transform_1, r1), (_, transform_2, r2)]) = iter.fetch_next() {
        let p1 = transform_1.translation;
        let p2 = transform_2.translation;

        let dst = p1.distance(p2) - r1.0 - r2.0;

        if dst <= 5.0 {
            commands.entity(entity_1).remove::<BubbleBundle>();

            //TODO: ABSTRACT THIS MESS
            commands.entity(entity_1).insert(BubbleBundle {
                velocity: Velocity {
                    x: rng.gen_range(VEL_RANGE),
                    y: rng.gen_range(VEL_RANGE),
                },

                radius: Radius(rng.gen_range(RADIUS_RANGE)),

                material: MaterialMesh2dBundle::<ColorMaterial> {
                    mesh: meshes
                        .add(shape::Circle::new(rng.gen_range(RADIUS_RANGE)).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                    transform: Transform::from_translation(Vec3 {
                        x: rng.gen_range(-(win_w / 2.0)..=(win_w / 2.0)),
                        y: rng.gen_range(-(win_h / 2.0)..=(win_h / 2.0)),
                        z: 0.0,
                    }),
                    ..default()
                },
            });
        }
    }
}

fn border_collision(
    windows: Res<Windows>,
    mut query: Query<(&Transform, &mut Velocity, &Radius)>
)
{
    let window = windows.get_primary().unwrap();
    let (w_height, w_width) = (window.height(),window.width());

    for (transform, mut vel, r) in query.iter_mut() {
        let pos = transform.translation;
        let radius = r.0;

        let (d_right, d_left) = (pos.x + radius, pos.x - radius);
        let (d_top, d_bottom) = (pos.y + radius, pos.y - radius);

        if d_right >= w_width/2.0 || d_left <= -(w_width/2.0) {
            vel.x = -vel.x;
        }

        if d_top >= w_height/2.0 || d_bottom <= -(w_height/2.0) {
            vel.y = -vel.y;
        }

    }
}
