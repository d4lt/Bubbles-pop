use bevy::{app::App, prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::{collide, Collision}}, window::PresentMode};
use rand::Rng;

pub const BG_COLOR: Color = Color::rgb(0.144, 0.144, 0.144);

pub const NUM_BUBBLES: u32 = 50;

pub const MAX_SCALE: f32 = 80.0;
pub const MAX_VELOCITY: f32 = 30.0;

pub const SCALE_RANGE: std::ops::Range<f32> = 35.0..MAX_SCALE;
pub const VEL_RANGE: std::ops::RangeInclusive<f32> = -MAX_VELOCITY..=MAX_VELOCITY;

pub const COL_BOX_RESCALE_FACTOR: f32 = 0.15;

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
            CoreStage::PostUpdate,
            InteractBodies,
            SystemStage::parallel()
            .with_system(collide_bubbles)
            .with_system(move_bubbles
                .after(border_collision)
                .after(collide_bubbles))
            .with_system(border_collision)
        )
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

    for _ in 0..NUM_BUBBLES {
        let position = Vec3 {
            x: rng.gen_range(-(win_w / 2.0)..=(win_w / 2.0)),
            y: rng.gen_range(-(win_h / 2.0)..=(win_h / 2.0)),
            z: 0.0,
        };

        let circle = meshes.add(
            shape::Circle::default().into(),
            // shape::Circle::new(MAX_RADIUS).into()
        );

        commands.spawn(BubbleBundle {
            velocity: Velocity {
                x: rng.gen_range(VEL_RANGE),
                y: rng.gen_range(VEL_RANGE),
            },


            material: MaterialMesh2dBundle::<ColorMaterial> {
                mesh: circle.into(),
                material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::splat(rng.gen_range(SCALE_RANGE))),
                ..default()
            },
            
        });
    }
}

// Basic move mechanic ( to upgrade )
fn move_bubbles(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, vel) in query.iter_mut() {
        transform.translation.x += vel.x * time.delta_seconds();
        transform.translation.y += vel.y * time.delta_seconds();
    }
}


fn collide_bubbles(
    query: Query<(Entity, &Transform), With<Velocity>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {

    let window = windows.get_primary().unwrap();
    let (win_w, win_h) = (window.width(), window.height());

    let mut rng = rand::thread_rng();

    let mut iter = query.iter_combinations();

    while let Some([(entity_1, transform_1), (_, transform_2)]) = iter.fetch_next() {
        let p1 = transform_1.translation;
        let p2 = transform_2.translation;

        let (scale_1, scale_2) = (transform_1.scale, transform_2.scale);

        // the 0.3 rescaling is because for the collision box to be inside the circle
        let size_1 = scale_1.truncate() - scale_1.x * COL_BOX_RESCALE_FACTOR;
        let size_2 = scale_2.truncate() - scale_2.x * COL_BOX_RESCALE_FACTOR;

        let collision = collide(
            p1,
            size_1 ,

            p2,
            size_2
        );


        if collision.is_some() {

            commands.entity(entity_1).remove::<BubbleBundle>();

            let radius = scale_1.x/2.0;

            let new_pos_rngx = -win_w/2.0 + radius..=win_w/2.0 - radius;
            let new_pos_rngy = -win_h/2.0 + radius..=win_h/2.0 - radius;

            let new_pos = Vec3{
                x: rng.gen_range( new_pos_rngx ),
                y: rng.gen_range( new_pos_rngy ),
                z: 0.0 
            };

            //TODO: ABSTRACT THIS MESS
            commands.entity(entity_1).insert(BubbleBundle {
                velocity: Velocity {
                    x: rng.gen_range(VEL_RANGE),
                    y: rng.gen_range(VEL_RANGE),
                },


                material: MaterialMesh2dBundle::<ColorMaterial> {
                    mesh: meshes
                        .add(shape::Circle::default().into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                    transform: Transform::from_translation(new_pos)
                        .with_scale(Vec3::splat(rng.gen_range(SCALE_RANGE))),
                    ..default()
                },
            });
        }
    }
}

fn border_collision(windows: Res<Windows>, mut query: Query<(&Transform, &mut Velocity)>) {
    let window = windows.get_primary().unwrap();
    let (w_height, w_width) = (window.height(), window.width());
    let center_pos = Vec3::ZERO;


    for (transform, mut vel) in query.iter_mut() {
        let pos = transform.translation;
        let size = transform.scale.truncate();

       let collision = collide(
        pos,
        size,
        center_pos,
        Vec2::from((w_width, w_height)),
       ); 

       if let Some(collision) = collision{
            match collision{
                Collision::Right | Collision::Left => vel.x = -vel.x,
                Collision::Top | Collision::Bottom => vel.y = -vel.y,
                _ => ()
            }
       }
    }
}
