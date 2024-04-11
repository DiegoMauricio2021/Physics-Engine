use bevy::{prelude::*, window::close_on_esc};
use bevy_pancam::{PanCam, PanCamPlugin};

use super::body::*;

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
const ITERATIONS: usize = 1;

pub struct PhsyicsEngine;

impl Plugin for PhsyicsEngine{
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.add_plugins(PanCamPlugin);
        app.insert_resource(ClearColor(Color::rgb_u8(11, 187, 202)));
        app.add_systems(Update, close_on_esc);
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, (run, collisions, draw).chain());
        app.add_systems(FixedUpdate, (moving, creating));
    }
}

fn draw(mut query: Query<(&Shape, &mut Transform)>,){
    for (ent,mut  transform) in query.iter_mut(){
        transform.translation = Vec3::new(ent.pos.x, ent.pos.y, 0.);
    }
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
            if key.pressed(KeyCode::KeyA) {
                i.rot += f32::to_radians(2.);
            }
            if key.pressed(KeyCode::KeyD) {
                i.rot -= f32::to_radians(2.);
            }
        }
    }
}

fn creating(
    query: Query<&Transform>,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    window: Query<&Window>,
    camera_transform: Query<&Transform, With<Camera>>,
    query_camera: Query<&OrthographicProjection>,
) {
    let window = window.single();
    let camera = query_camera.single();
    let res = || {
        let camera_pos = camera_transform
            .get_single()
            .unwrap()
            .translation
            .truncate();
        let window_size =
            Vec2::new(-window.resolution.width(), window.resolution.height()) * camera.scale;
        let pos = window.cursor_position().unwrap() * camera.scale;
        let pos =
            Vec2::new((window_size / 2.).x + pos.x, (window_size / 2.).y - pos.y) + camera_pos;
        let w = thread_rng().gen_range(10..=50) as f32;
        let h = thread_rng().gen_range(10..=50) as f32;
        (pos, w, h)
    };
    if input.just_pressed(MouseButton::Left) {
        let (pos, r, _) = res();
        let color = COLORS[thread_rng().gen_range(0..=7)];
        create_body(
            &mut commands,
            &mut meshes,
            &mut materials,
            Bodys::Circle(r),
            pos,
            color,
            false,
            false,
        );
    }
    if input.just_pressed(MouseButton::Right) {
        let (pos, w, h) = res();
        let color = COLORS[thread_rng().gen_range(0..=7)];
        if thread_rng().gen_bool(0.5) {
            create_body(
                &mut commands,
                &mut meshes,
                &mut materials,
                Bodys::Rec(w, h),
                pos,
                color,
                false,
                false,
            );
        }else{
            let r = thread_rng().gen_range(3..=10);
            create_body(
                &mut commands,
                &mut meshes,
                &mut materials,
                Bodys::Poly(w, r),
                pos,
                color,
                false,
                false,
            );
        }
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
            let mass = ent.mass;
            let vel_rot = ent.rot_vel;
            ent.rot += vel_rot * time;
            let force = ent.force;
            ent.ac += force / mass * time;
            let ac = ent.ac;
            ent.vel += ac * time;
            let vel = ent.vel;
            ent.pos += vel * time;
            ent.force = Vec2::ZERO;
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
        let mut normal = Vec2::ZERO;
        for _ in 0..ITERATIONS {
            if a.is_static && b.is_static {
                continue;
            }
            a.aabb = a.get_aabb();
            b.aabb = b.get_aabb();
            if a.checkaabb(&b) {
                continue;
            }

            let (norma, distance) = a.collision(b.pos, &b, &mut gizmos);
            normal = norma;
            if a.is_static {
                b.pos += norma * distance;
            } else if b.is_static {
                a.pos += -(norma * distance);
            } else {
                a.pos += -(norma * distance / 2.);
                b.pos += norma * distance / 2.;
            }
        }
        let sf = (a.static_friction + b.static_friction) * 0.5;
        let df = (a.dinaminc_friction + b.dinaminc_friction) * 0.5;
        let (contact1, contact2, cc) = a.contactpoint(&b);

        let rel_vel = a.vel - b.vel;

        if rel_vel.dot(normal) == 0. {
            continue;
        }

        let e = f32::min(a.restitution, b.restitution);

        let mainv = a.inv_mass;
        let mbinv = b.inv_mass;
        let iainv = a.inv_inertia;
        let ibinv = b.inv_inertia;

        let contactl = [contact1, contact2];
        let mut impulsel = vec![];
        let mut jl = vec![0., 0.];
        let mut ral = vec![];
        let mut rbl = vec![];
        for (r, i) in contactl.iter().enumerate() {
            let i = *i;
            let ra = i - a.pos;
            let rb = i - b.pos;
            ral.push(ra);
            rbl.push(rb);
            let raper = Vec2::new(-ra.y, ra.x);
            let rbper = Vec2::new(-rb.y, rb.x);

            let alv_a = raper * a.rot_vel;
            let alv_b = rbper * b.rot_vel;

            let rel_vel = (b.vel + alv_b) - (a.vel + alv_a);

            let contacvel = rel_vel.dot(normal);

            if contacvel > 0. {
                continue;
            }

            let raperdot = raper.dot(normal);
            let rbperdot = rbper.dot(normal);

            let denom = a.inv_mass
                + b.inv_mass
                + (raperdot * raperdot) * a.inv_inertia
                + (rbperdot * rbperdot) * b.inv_inertia;

            let j = -(1. + e) * contacvel;
            let j = j / denom;
            let j = j / cc as f32;

            let impulse = j * normal;
            impulsel.push(impulse);
            jl[r] = j;
        }
        for (i, impulse) in impulsel.iter().enumerate() {
            let impulse = *impulse;
            let ra = ral[i];
            let rb = rbl[i];

            a.vel += -impulse * mainv;
            a.rot_vel += -ra.perp_dot(impulse) * iainv;
            b.vel += impulse * mbinv;
            b.rot_vel += rb.perp_dot(impulse) * ibinv;
        }
        //-------------------------------------------------------------------Friction-----------------------------------------------------------------------------------------
        /* let mut impulsefrictionl = vec![];
        for (j, i) in contactl.iter().enumerate() {
            let j = jl[j];
            let i = *i;
            let ra = i - a.pos;
            let rb = i - b.pos;
            ral.push(ra);
            rbl.push(rb);
            let raper = Vec2::new(-ra.y, ra.x);
            let rbper = Vec2::new(-rb.y, rb.x);

            let alv_a = raper * a.rot_vel;
            let alv_b = rbper * b.rot_vel;

            let rel_vel = (b.vel + alv_b) - (a.vel + alv_a);

            let mut tangent = rel_vel - rel_vel.dot(normal) * normal;

            if tangent == Vec2::ZERO {
                continue;
            } else {
                tangent = tangent.normalize()
            }

            let raperdot = raper.dot(tangent);
            let rbperdot = rbper.dot(tangent);

            let denom = a.inv_mass
                + b.inv_mass
                + (raperdot * raperdot) * a.inv_inertia
                + (rbperdot * rbperdot) * b.inv_inertia;

            let jt = -rel_vel.dot(tangent);
            let jt = jt / denom;
            let jt = jt / cc as f32;

            let impulsefriction = if jt.abs() <= j * sf {
                jt * tangent
            } else {
                -j * tangent * df
            };
            impulsefrictionl.push(impulsefriction);
        }
        for impulse in impulsefrictionl {

            a.vel += -impulse * mainv;
            b.vel += impulse * mbinv;
        } */
        
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
    create_body(
        &mut commands,
        &mut meshes,
        &mut materials,
        Bodys::Rec(r, r),
        Vec2::new(0., 0.),
        Color::GREEN,
        true,
        false,
    );
    create_body(
        &mut commands,
        &mut meshes,
        &mut materials,
        Bodys::Rec(1000., 100.),
        Vec2::new(500., 50.),
        Color::DARK_GREEN,
        false,
        true,
    );
    let (mut red, mesh) = create_shape(
        &mut meshes,
        Bodys::Rec(500., 50.),
        Vec2::new(50., 500.),
        false,
        true,
    );
    red.rot = f32::to_radians(-20.);
    spawn_shape(&mut commands, Color::RED, mesh, &mut materials, red);
}
