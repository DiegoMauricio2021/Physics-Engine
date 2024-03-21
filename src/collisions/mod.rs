use super::body::Shape;
use super::body::Bodys::{Circle, Rec};
use bevy::prelude::*;


impl Shape {
    pub fn collision(&self, pb: Vec2, shape: &Shape, mut gizmos: &mut Gizmos) -> (Vec2, f32) {
        
        match (&self.kind, &shape.kind) {
            (Circle(ra), Circle(rb)) => self.cir_to_cir(*ra, *rb, pb),
            (Rec(_, _), Rec(_, _)) => self.rec_to_rec(shape),
            (Rec(_, _), Circle(_)) => {
                let (normal, depth) = Shape::rec_to_cir(&self, shape, &mut gizmos);
                (-normal, depth)
            },
            _ => Shape::rec_to_cir(shape, &self, &mut gizmos),
        }
    }

    fn proyect_vecs(vertices: &Vec<Vec2>, axis: Vec2, pos: Vec2) -> (f32, f32) {
        let mut max = std::f32::MIN;
        let mut min = std::f32::MAX;

        for i in vertices {
            let proj = Vec2::dot(*i + pos, axis.normalize());
            if proj > max {
                max = proj
            }
            if proj < min {
                min = proj
            }
        }
        (max, min)
    }
    pub fn rotate(&mut self, angle: f32) {
        match self.kind {
            Rec(_, _) => {
                for i in &mut self.vertices {
                    let cos = angle.cos();
                    let sin = angle.sin();
                    let rx = cos * i.x - sin * i.y;
                    let ry = sin * i.x + cos * i.y;
                    *i = Vec2::new(rx, ry);
                }
            }
            Circle(_) => {}
        }
    }
    fn cir_to_cir(&self, ra: f32, rb: f32, pb: Vec2) -> (Vec2, f32) {
        let distance = self.pos.distance(pb);
        let rad = ra + rb;
        let normal = (pb - self.pos).normalize_or_zero();
        if distance >= rad {
            return (Vec2::ZERO, 0.);
        }

        (normal, rad - distance)
    }

    fn rec_to_rec(&self, shape: &Shape) -> (Vec2, f32) {
        let mut depth = f32::MAX;
        let mut axis;
        let mut normal = Vec2::ZERO;
        for (i, _) in self.vertices.iter().enumerate() {
            let va = &self.vertices[i];
            let vb = &self.vertices[(i + 1) % self.vertices.len()];

            axis = *vb - *va;
            axis = Vec2::new(-axis.y, axis.x);

            let (maxa, mina) = Shape::proyect_vecs(&self.vertices, axis, self.pos);
            let (maxb, minb) = Shape::proyect_vecs(&shape.vertices, axis, shape.pos);
            if mina >= maxb || minb >= maxa {
                return (Vec2::ZERO, 0.);
            }
            let axisdepth = f32::min(maxb - mina, maxa - minb);
            if axisdepth < depth {
                normal = axis;
                depth = axisdepth;
            }
        }

        let centera = self.pos;
        let centerb = shape.pos;
        let direction = centerb - centera;

        if direction.dot(normal) < 0. {
            normal = -normal;
        }
        
        for (i, _) in shape.vertices.iter().enumerate() {
            let va = &shape.vertices[i];
            let vb = &shape.vertices[(i + 1) % shape.vertices.len()];

            axis = *vb - *va;
            axis = Vec2::new(-axis.y, axis.x);

            let (maxa, mina) = Shape::proyect_vecs(&shape.vertices, axis, shape.pos);
            let (maxb, minb) = Shape::proyect_vecs(&self.vertices, axis, self.pos);
            if mina >= maxb || minb >= maxa {
                return (Vec2::ZERO, 0.);
            }
            let axisdepth = f32::min(maxb - mina, maxa - minb);
            if axisdepth < depth {
                normal = axis;
                depth = axisdepth;
            }
        }

        let centera = shape.pos;
        let centerb = self.pos;
        let direction = centera - centerb;

        if direction.dot(normal) < 0. {
            normal = -normal;
        }

        (normal.normalize(), depth)
    }

    fn rec_to_cir(slf: &Shape, shape: &Shape, mut _gizmos: &mut Gizmos) -> (Vec2, f32) {
        let radius = match shape.kind {
            Circle(r) => r,
            _ => 0.,
        };
        let mut depth = f32::MAX;
        let mut normal = Vec2::ZERO;
        let mut axis;
        for (i, _) in slf.vertices.iter().enumerate() {
            let va = &slf.vertices[i];
            let vb = &slf.vertices[(i + 1) % slf.vertices.len()];

            axis = *vb - *va;
            axis = Vec2::new(-axis.y, axis.x);

            let (maxa, mina) = Shape::proyect_vecs(&slf.vertices, axis, slf.pos);
            let (maxb, minb) = Shape::proyect_cir(shape.pos, radius, axis);
            //println!("{}, {}, {}, {}, {}", axis, maxa, mina, maxb, minb);
            if mina >= maxb || minb >= maxa {
                return (Vec2::ZERO, 0.);
            }
            let axisdepth = f32::min(maxb - mina, maxa - minb);
            if axisdepth < depth {
                normal = axis;
                depth = axisdepth;
            }
        }

        let cpindex = Shape::close_point(shape.pos, &slf.vertices, slf.pos);
        let axi = slf.vertices[cpindex];
        let (maxa, mina) = Shape::proyect_vecs(&slf.vertices, axi, slf.pos);
        let (maxb, minb) = Shape::proyect_cir(shape.pos, radius, axi);
        if mina >= maxb || minb >= maxa {
            return (Vec2::ZERO, 0.);
        }
        let axisdepth = f32::min(maxb - mina, maxa - minb);
        if axisdepth < depth {
            depth = axisdepth;
            axis = axi;
            normal = axis;
        }

        let centera = slf.pos;
        let centerb = shape.pos;

        let direction = centera - centerb;
        if direction.dot(normal) <= 0. {
            normal = -normal;
        }

        (normal.normalize(), depth)
    }

    fn close_point(center: Vec2, vertices: &Vec<Vec2>, pos: Vec2) -> usize {
        let mut result = 0;
        let mut min_distance = f32::MAX;

        for (i, v) in vertices.iter().enumerate() {
            let distance = Vec2::distance(*v + pos, center);
            if distance < min_distance {
                min_distance = distance;
                result = i;
            }
        }
        result
    }

    fn proyect_cir(center: Vec2, radius: f32, axis: Vec2) -> (f32, f32) {
        let direction = axis.normalize();
        let dir_rad = direction * radius;

        let p1 = center + dir_rad;
        let p2 = center - dir_rad;

        let mut min = Vec2::dot(p1, axis.normalize());
        let mut max = Vec2::dot(p2, axis.normalize());

        if min > max {
            (min, max) = (max, min);
        }
        (max, min)
    }
}
