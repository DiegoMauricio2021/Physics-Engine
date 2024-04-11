use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use Bodys::*;

use bevy::math::primitives;

pub enum Bodys {
    Circle(f32),
    Rec(f32, f32),
    Poly(f32, usize),
}

impl Bodys {
    fn size(&self) -> (f32, f32) {
        let mut minx = f32::MAX;
        let mut maxx = f32::MIN;
        let mut miny = f32::MAX;
        let mut maxy = f32::MIN;
        for i in &self.vertices() {
            maxx = maxx.max(i.x);
            maxy = maxy.max(i.y);
            minx = minx.min(i.x);
            miny = miny.min(i.y);
        }
        (maxx - minx, maxy - miny)
    }

    fn area(&self) -> f32 {
        match *self {
            Circle(r) => r.powi(2) * PI,
            Rec(w, h) => w * h,
            Poly(r, n) => RegularPolygon::new(r, n as usize).area(),
        }
    }
    fn vertices(&self) -> Vec<Vec2> {
        match *self {
            Rec(w, h) => {
                vec![
                    Vec2::new(-w / 2., h / 2.),
                    Vec2::new(w / 2., h / 2.),
                    Vec2::new(w / 2., -h / 2.),
                    Vec2::new(-w / 2., -h / 2.),
                ]
            }
            Poly(r, n) => RegularPolygon::new(r, n as usize)
                .vertices(0.)
                .into_iter()
                .collect(),
            _ => {
                vec![]
            }
        }
    }
}

pub struct AABB {
    pub max: Vec2,
    pub min: Vec2,
}
impl AABB {
    fn new(maxx: f32, maxy: f32, minx: f32, miny: f32) -> AABB {
        AABB {
            max: Vec2::new(maxx, maxy),
            min: Vec2::new(minx, miny),
        }
    }
}

pub fn create_shape(
    meshes: &mut ResMut<Assets<Mesh>>,
    shape: Bodys,
    pos: Vec2,
    movil: bool,
    stat: bool,
) -> (Shape, Mesh2dHandle) {
    let (mut shape, mesh) = match shape {
        Circle(r) => {
            let cir = Circle(r);
            let mass = cir.area();
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
        Rec(w, h) => {
            let rec = Rec(w, h);
            let mass = rec.area();
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
                Mesh2dHandle(meshes.add(Rectangle::new(w, h))),
            )
        }
        Poly(r, n) => {
            let poly = Poly(r, n);
            let vertices = poly.vertices();
            let mass = poly.area();
            let (w, h) = poly.size();
            (
                Shape {
                    area: poly.area(),
                    vertices,
                    kind: poly,
                    pos,
                    movil,
                    mass,
                    is_static: stat,
                    inv_mass: if stat { 0. } else { 1. / mass },
                    w,
                    h,
                    ..default()
                },
                Mesh2dHandle(meshes.add(RegularPolygon::new(r, n))),
            )
        }
    };
    shape.aabb = shape.get_aabb();
    shape.inertia = shape.get_inertia();
    shape.inv_inertia = if stat { 0. } else { 1. / shape.inertia };
    shape.restitution = 0.05;
    (shape, mesh)
}

pub fn spawn_shape(
    commands: &mut Commands,
    color: Color,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    shape: Shape,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh,
            material: materials.add(color),
            transform: Transform::from_xyz(shape.pos.x, shape.pos.y, 0.),
            ..default()
        },
        shape,
    ));
}

pub fn create_body(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    shape: Bodys,
    pos: Vec2,
    color: Color,
    movil: bool,
    stat: bool,
) {
    let (mut shape, mesh) = create_shape(meshes, shape, pos, movil, stat);
    shape.ac = Vec2::new(0., -196.);

    spawn_shape(commands, color, mesh, materials, shape);
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
    pub inertia: f32,
    pub inv_inertia: f32,
    pub static_friction: f32,
    pub dinaminc_friction: f32,
    pub h: f32,
    pub w: f32,
}

impl std::default::Default for Shape {
    fn default() -> Self {
        Self {
            kind: Circle(20.),
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
            inertia: 0.,
            inv_inertia: 0.,
            static_friction: 0.6,
            dinaminc_friction: 0.4,
            h: 0.,
            w: 0.,
        }
    }
}

impl Shape {
    pub fn get_aabb(&self) -> AABB {
        match self.kind {
            Circle(r) => {
                let minx = self.pos.x - r;
                let maxx = self.pos.x + r;
                let miny = self.pos.y - r;
                let maxy = self.pos.y + r;
                AABB::new(maxx, maxy, minx, miny)
            }
            _ => {
                let mut minx = f32::MAX;
                let mut maxx = f32::MIN;
                let mut miny = f32::MAX;
                let mut maxy = f32::MIN;
                for i in &self.vertices {
                    let i = *i + self.pos;
                    maxx = maxx.max(i.x);
                    maxy = maxy.max(i.y);
                    minx = minx.min(i.x);
                    miny = miny.min(i.y);
                }
                AABB::new(maxx, maxy, minx, miny)
            }
        }
    }

    fn get_inertia(&self) -> f32 {
        match self.kind {
            Circle(r) => 1. / 2. * self.mass * r * r,
            Rec(w, h) => 1. / 12. * self.mass * (w * w + h * h),
            Poly(_, _) => {
                let w = self.w;
                let h = self.h;
                1. / 12. * self.mass * (w * w + h * h)
            }
        }
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct Vecs {}
