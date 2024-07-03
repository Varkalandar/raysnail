use vecmath::Matrix4


struct Transform
{
    matrix: Matrix4<64>,
    inverse: Matrix4<64>,
}