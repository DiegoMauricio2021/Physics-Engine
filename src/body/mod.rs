use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use bevy::math::primitives;

pub enum Bodys {
    Circle(f32),
    Rec(f32, f32),
}

impl Bodys {
    fn area(&self) -> f32 {
        match *self {
            Bodys::Circle(r) => r.powi(2) * PI,
            Bodys::Rec(w, h) => w * h,
        }
    }
    fn vertices(&self) -> Vec<Vec2> {
        match *self {
            Bodys::Rec(w, h) => {
                vec![
                    Vec2::new(-w / 2., h / 2.),
                    Vec2::new(w / 2., h / 2.),
                    Vec2::new(w / 2., -h / 2.),
                    Vec2::new(-w / 2., -h / 2.),
                ]
            }
            _ => {
                vec![]
            }
        }
    }
}

pub struct AABB{
    max: Vec2,
    min: Vec2,
} 
impl AABB{
    fn new(maxx: f32, maxy: f32, minx: f32, miny: f32) -> AABB{
        AABB{
            max: Vec2::new(maxx, maxy),
            min: Vec2::new(minx, miny)
        }
    }
}

pub fn create_body(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    shape: Bodys,
    pos: Vec2,
    color: Color,
    movil: bool,
    mass: f32,
    stat: bool,
) {
    let (mut shape, mesh) = match shape {
        Bodys::Circle(r) => {
            let cir = Bodys::Circle(r);
            (
                Shape {
                    area: cir.area(),
                    kind: cir,
                    pos,
                    movil,
                    mass,
                    is_static: stat,
                    inv_mass: if stat { 0. } else { 1. / mass },
                    ..default()
                },
                Mesh2dHandle(meshes.add(primitives::Circle::new(r))),
            )
        }
        Bodys::Rec(w, h) => {
            let rec = Bodys::Rec(w, h);

            (
                Shape {
                    area: rec.area(),
                    vertices: rec.vertices(),
                    kind: rec,
                    pos,
                    movil,
                    mass,
                    is_static: stat,
                    inv_mass: if stat { 0. } else { 1. / mass },
                    ..default()
                },
                Mesh2dHandle(meshes.add(primitives::Rectangle::new(w, h))),
            )
        }
    };
    shape.aabb = shape.get_aabb();
    shape.restitution = 0.5;
    //shape.rot_vel = f32::to_radians(2.);
    shape.ac = Vec2::new(0., -98.);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xyz(pos.x, pos.y, 0.),
            ..default()
        },
        shape,
    ));
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Shape {
    pub kind: Bodys,
    pub pos: Vec2,
    pub vel: Vec2,
    pub ac: Vec2,
    pub rot: f32,
    pub rot_vel: f32,
    pub mass: f32,
    pub restitution: f32,
    pub area: f32,
    pub is_static: bool,
    pub vec: bool,
    pub movil: bool,
    pub vertices: Vec<Vec2>,
    pub force: Vec2,
    pub inv_mass: f32,
    pub aabb: AABB,
}

impl std::default::Default for Shape {
    fn default() -> Self {
        Self {
            kind: Bodys::Circle(20.),
            pos: Vec2::default(),
            vel: Vec2::default(),
            ac: Vec2::default(),
            rot: 0.,
            rot_vel: 0.,
            mass: 1.,
            restitution: 1.,
            area: 0.,
            is_static: false,
            vec: false,
            movil: true,
            vertices: vec![],
            force: Vec2::default(),
            inv_mass: 0.,
            aabb: AABB::new(0., 0., 0., 0.),
        }
    }
}

impl Shape{
    fn get_aabb(&self) -> AABB{
        match self.kind{
            Bodys::Circle(r) => {
                let minx = self.pos.x - r;
                let maxx = self.pos.x + r;
                let miny = self.pos.y - r;
                let maxy = self.pos.y + r;
                AABB::new(maxx, maxy, minx, miny)
            },
            Bodys::Rec(_, _) => {
                let mut minx = f32::MAX;
                let mut maxx = f32::MIN;
                let mut miny = f32::MAX;
                let mut maxy = f32::MIN;
                for i in &self.vertices{
                    maxx = maxx.max(i.x);
                    maxy = maxx.max(i.y);
                    minx = minx.min(i.x);
                    miny = minx.min(i.y);
                }
                AABB::new(maxx, maxy, minx, miny)
            },
        }
    }
}

#[allow(dead_code)]
fn draw_vecs(gizmos: &mut Gizmos, pos: Vec2, vec: Vec2, color: Color) {
    gizmos.ray_2d(pos, vec, color);
}


#[derive(Default, Reflect, GizmoConfigGroup)]
struct Vecs {}
