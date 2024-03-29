use bevy::{prelude::*, window::close_on_esc};
use bevy_pancam::{PanCam, PanCamPlugin};

extern crate physics_engine;
use physics_engine::body::*;

use rand::{thread_rng, Rng};

const COLORS: [Color; 8] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::CYAN,
    Color::DARK_GREEN,
    Color::AQUAMARINE,
    Color::AZURE,
    Color::YELLOW,
];
const ITERATIONS: usize = 16;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin)
        .insert_resource(ClearColor(Color::rgb_u8(11, 187, 202)))
        .add_systems(Update, close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (run, collisions).chain())
        .add_systems(FixedUpdate, (moving, creating))
        .run();
}

fn draw_vecs(gizmos: &mut Gizmos, pos: Vec2, vec: Vec2, color: Color) {
    gizmos.ray_2d(pos, vec, color);
}

fn moving(mut query: Query<&mut Shape>, key: Res<ButtonInput<KeyCode>>) {
    for mut i in query.iter_mut() {
        if i.movil {
            i.vel = Vec2::ZERO;
            if key.pressed(KeyCode::ArrowLeft) {
                i.vel.x = -250.;
            }
            if key.pressed(KeyCode::ArrowRight) {
                i.vel.x = 250.;
            }
            if key.pressed(KeyCode::ArrowUp) {
                i.vel.y = 250.;
            }
            if key.pressed(KeyCode::ArrowDown) {
                i.vel.y = -250.;
            }
            if key.pressed(KeyCode::KeyA){
                i.rot += f32::to_radians(2.);
            }
            if key.pressed(KeyCode::KeyD){
                i.rot -= f32::to_radians(2.);
            }
        }
    }
}

fn creating(query: Query<&Transform>,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands, 
    window: Query<&Window>,
    camera_transform: Query<&Transform, With<Camera>>,
    query_camera: Query<&OrthographicProjection>,
){
    let window = window.single();
    let camera = query_camera.single();
    let res = || {
        let camera_pos = camera_transform.get_single().unwrap().translation.truncate();
        let window_size = Vec2::new(-window.resolution.width(), window.resolution.height()) * camera.scale;
        let pos = window.cursor_position().unwrap() * camera.scale;
        let pos =  Vec2::new((window_size/2.).x + pos.x, (window_size/2.).y - pos.y) + camera_pos;
        let w = thread_rng().gen_range(10..=50) as f32;
        let h = thread_rng().gen_range(10..=50) as f32;
        (pos, w, h)
    };
    if input.just_pressed(MouseButton::Left){
        let (pos, r, _) = res();
        let color = COLORS[thread_rng().gen_range(0..=7)];
        create_body( &mut commands, &mut meshes, &mut materials, Bodys::Circle(r), pos , color, false, 1., false,);
    }
    if input.just_pressed(MouseButton::Right){
        let (pos, w, h) = res();
        let color = COLORS[thread_rng().gen_range(0..=7)];
        create_body( &mut commands, &mut meshes, &mut materials, Bodys::Rec(w, h), pos , color, false, 1., false,);
    }
}

fn run(
    mut query: Query<(&mut Shape, &mut Transform)>,
    fixed_time: Res<Time<Fixed>>,
    mut gizmos: Gizmos,
) {
    let time = fixed_time.timestep().as_millis() as f32 / 1000.;

    for (mut ent, mut transform) in query.iter_mut() {
        
        if !ent.is_static {
            let vel_rot = ent.rot_vel;
            ent.rot += vel_rot * time;
            let mass = ent.mass;
            let force = ent.force;
            ent.ac += force / mass * time;
            let ac = ent.ac;
            ent.vel += ac * time;
            let vel = ent.vel;
            ent.pos += vel * time;
            transform.translation = Vec3::new(ent.pos.x, ent.pos.y, 0.);
        }
        let rot = ent.rot;
        ent.rotate(rot);
        transform.rotate_z(rot);
        ent.rot = 0.;
        
        

        if ent.vec {
            draw_vecs(&mut gizmos, ent.pos, Vec2::new(ent.pos.x, 0.), Color::GREEN);
            draw_vecs(&mut gizmos, ent.pos, Vec2::new(0., ent.pos.y), Color::RED);
        }
    }
}

fn collisions(mut query: Query<&mut Shape>, mut gizmos: Gizmos) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        for _ in 0..ITERATIONS {
            if a.is_static && b.is_static{
                continue;
            }
            a.aabb = a.get_aabb();
            b.aabb = b.get_aabb();
            if a.checkaabb(&b){
                continue;
            }
            

            let (normal, distance) = a.collision(b.pos, &b, &mut gizmos);

            if a.is_static {
                b.pos += normal * distance;
            } else if b.is_static {
                a.pos += -(normal * distance);
            } else {
                a.pos += -(normal * distance / 2.);
                b.pos += normal * distance / 2.;
            }
           
            let (contact1, contact2, cc) = a.contactpoint(&b, &mut gizmos);
            
            let rel_vel = a.vel - b.vel;
    
            if rel_vel.dot(normal) == 0. {
                continue;
            }
    
            let e = f32::min(a.restitution, b.restitution);
            let j = -(1. + e) * rel_vel.dot(normal);
            let j = j / (a.inv_mass + b.inv_mass);
    
            let impulse = j * normal;
    
            let ainv = a.inv_mass;
            let binv = b.inv_mass;
    
            a.vel += impulse * ainv;
            b.vel -= impulse * binv;

            if cc > 0 {
                gizmos.rect_2d(contact1, 0., Vec2::ONE, Color::ORANGE); 
                if cc > 1 {
                    gizmos.rect_2d(contact2, 0., Vec2::ONE, Color::ORANGE); 
                }
            }
            
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());
    let r = 50.;
    let mut rng = thread_rng();
    create_body( &mut commands, &mut meshes, &mut materials, Bodys::Rec(r, r), Vec2::new(0., 0.), Color::GREEN, true, 1., false,);
    create_body( &mut commands, &mut meshes, &mut materials, Bodys::Rec(1000., 100.), Vec2::new(500., 50.), Color::DARK_GREEN, false, 1., true,);
    /* for i in 0..10 {
        let shape = if i % 2 == 0 {
            Bodys::Rec(r, r)
        } else {
            Bodys::Circle(r)
        };

        let x = rng.gen_range(0..=2000) as f32;
        let y = rng.gen_range(0..=2000) as f32;
        let movil = rng.gen_bool(0.5);
        let color = if !movil {
            COLORS[rng.gen_range(0..=7)]
        } else {
            Color::DARK_GRAY
        };

        create_body( &mut commands, &mut meshes, &mut materials, shape, Vec2::new(x, y), color, false, 1., movil,);
    } */
}
