pub struct Vertex{
    position: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
}
pub struct Line{
    position: [f32; 3],
    vertex: [Vertex; 2],
}
pub struct Triangle{
    position: [f32; 3],
    vertex: [Vertex; 3],
}

pub struct Quad{
    position: [f32; 3],
    triangle: [Triangle; 2],
}

