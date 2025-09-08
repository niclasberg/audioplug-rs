use crate::core::Rect;


pub struct BVHNode {
    bounds: Rect<f32>,
    left_child: u32, 
    
}