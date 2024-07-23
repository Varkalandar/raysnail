use vecmath::Matrix4;
use vecmath::Vector4;
use vecmath::row_mat4_transform;
use vecmath::mat4_id;
use vecmath::mat4_inv;

use crate::prelude::Vec3;

#[derive(Debug)]
pub struct Transform
{
    pub matrix: Matrix4<f64>,
    pub inverse: Matrix4<f64>,
}

impl Transform {
    pub fn translate(t: Vec3) -> Self {
        let mut m = mat4_id();

        m[0][3] = t.x;
        m[1][3] = t.y;
        m[2][3] = t.z;

        let inv = mat4_inv(m);
        /*
        let mut inv = mat4_id();

        inv[0][3] = -t.x;
        inv[1][3] = -t.y;
        inv[2][3] = -t.z;
*/
        let t = Transform {matrix: m, inverse: inv};

        t
    }

    pub fn rotate_by_y_axis(theta: f64) -> Self {

        let sin = theta.sin();
        let cos = theta.cos();

        let mut m = mat4_id();

        m[0][0] = cos;
        m[0][2] = sin;
        m[2][0] = -sin;
        m[2][2] = cos;

        let inv = mat4_inv(m);

        let t = Transform {matrix: m, inverse: inv};

        t
    }

}


#[derive(Debug)]
pub struct TransformStack {
    stack: Vec<Transform>,
}


impl TransformStack {
    pub fn new() -> TransformStack {
        TransformStack {
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, t: Transform) {
        self.stack.push(t);
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn tf_pos(&self, pos: &Vec3) -> Vec3 {
        if self.stack.len() > 0 {
            let v4 = [pos.x, pos.y, pos.z, 1.0];
            let r = row_mat4_transform(self.stack[0].matrix, v4);

            Vec3::new(r[0], r[1], r[2])
        }
        else {
            pos.clone()
        }
    }

    pub fn inv_tf_pos(&self, pos: &Vec3) -> Vec3 {

        if self.stack.len() > 0 {
            let v4 = [pos.x, pos.y, pos.z, 1.0];
            let r = row_mat4_transform(self.stack[0].inverse, v4);

            Vec3::new(r[0], r[1], r[2])
        }
        else {
            pos.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation() {
        let mut tfs = TransformStack::new();

        tfs.push_translation(Vec3::new(20.0, 19.0, 18.0));

        let r = tfs.tf_pos(&Vec3::new(0.0, 0.0, 0.0));

        assert_eq!(r.x, 20.0);
        assert_eq!(r.y, 19.0);
        assert_eq!(r.z, 18.0);


        let r2 = tfs.inv_tf_pos(&r);

        assert_eq!(r2.x, 0.0);
        assert_eq!(r2.y, 0.0);
        assert_eq!(r2.z, 0.0);
    }
}

