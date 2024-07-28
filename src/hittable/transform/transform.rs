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

    pub fn rotate_by_x_axis(theta: f64) -> Self {

        let sin = theta.sin();
        let cos = theta.cos();

        let mut m = mat4_id();

        m[1][1] = cos;
        m[1][2] = sin;
        m[2][1] = -sin;
        m[2][2] = cos;

        let inv = mat4_inv(m);

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

    pub fn rotate_by_z_axis(theta: f64) -> Self {

        let sin = theta.sin();
        let cos = theta.cos();

        let mut m = mat4_id();

        m[0][0] = cos;
        m[0][1] = sin;
        m[1][0] = -sin;
        m[1][1] = cos;

        let inv = mat4_inv(m);

        let t = Transform {matrix: m, inverse: inv};

        t
    }


    pub fn scale(t: Vec3) -> Self {
        let mut m = mat4_id();

        m[0][0] = t.x;
        m[1][1] = t.y;
        m[2][2] = t.z;
        m[3][3] = 1.0;

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

    pub fn forward(&self, pos: &Vec3, w: f64) -> Vec3 {
        let mut result = pos.clone();

        for transform in &self.stack {
            let v4 = [result.x, result.y, result.z, w];
            let r = row_mat4_transform(transform.matrix, v4);

            result = Vec3::new(r[0], r[1], r[2])
        }

        result
    }

    pub fn inverse(&self, pos: &Vec3, w: f64) -> Vec3 {
        let mut result = pos.clone();

        for transform in self.stack.iter().rev() {
            let v4 = [result.x, result.y, result.z, w];
            let r = row_mat4_transform(transform.inverse, v4);

            result = Vec3::new(r[0], r[1], r[2])
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::PI;

    /*
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
    */

    #[test]
    fn test_y_rotation() {
        let mut tfs = TransformStack::new();

        let tf = Transform::rotate_by_y_axis(PI/2.0);

        tfs.push(tf);

        let r = tfs.tf_pos(&Vec3::new(0.0, 0.0, 1.0));

        assert!((r.x - 1.0).abs() < 1e-10);
        assert!((r.y - 0.0).abs() < 1e-10);
        assert!((r.z - 0.0).abs() < 1e-10);

        let r2 = tfs.inv_tf_pos(&r);

        assert!((r2.x - 0.0).abs() < 1e-10);
        assert!((r2.y - 0.0).abs() < 1e-10);
        assert!((r2.z - 1.0).abs() < 1e-10);
    }

}

